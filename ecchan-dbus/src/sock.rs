use std::{
    io::{self, ErrorKind, Read, Write},
    os::unix::net::UnixStream,
};

use ecchan_ipc::{
    method::Method,
    ret::{Ret, RetVal},
};
use snafu::{ResultExt as _, Snafu};

#[derive(Debug, Snafu)]
pub enum ClientError {
    #[snafu(display("{msg}"))]
    Call {
        msg: String,
    },

    Json {
        source: serde_json::Error,
    },

    Io {
        source: io::Error,
    },
}

pub struct Client {
    conn: UnixStream,
    buf: Vec<u8>,
    sentinel_pos: usize,
}

impl Client {
    pub fn new() -> io::Result<Self> {
        let conn = UnixStream::connect(ecchan_ipc::get_socket_path())?;
        Ok(Self {
            conn,
            buf: vec![0; 1024],
            sentinel_pos: 0,
        })
    }

    pub fn call<'a>(&'a mut self, call: Method) -> Result<RetVal<'a>, ClientError> {
        self.buf.clear();

        let data = serde_json::to_string(&call).context(JsonSnafu)?;
        let encoded = cobs::encode_vec_including_sentinels(data.as_bytes());

        self.conn.write_all(&encoded).context(IoSnafu)?;

        let mut buf = [0; 1024];

        let mut z = 0;
        let data = loop {
            match self.conn.read(&mut buf) {
                Ok(n) => match n {
                    0 => continue,
                    n => {
                        let msg = &buf[..n];

                        // accumulate full message
                        self.buf.extend_from_slice(msg);

                        // count zeroes
                        for &b in msg {
                            if b == 0 {
                                z += 1;
                            }

                            if z >= 2 {
                                break;
                            }
                        }

                        // ensure we have 2 zeroes (begin and end sentinel)
                        if z < 2 {
                            continue;
                        }

                        self.sentinel_pos = self
                            .buf
                            .iter()
                            .enumerate()
                            .filter(|(_, b)| **b == 0)
                            .map(|(pos, _)| pos)
                            .nth(1)
                            .unwrap();
                    }
                },

                Err(e) => match e.kind() {
                    ErrorKind::WouldBlock => continue,
                    _ => return Err(e).context(IoSnafu),
                },
            }

            let data = match cobs::decode_in_place(&mut self.buf[1..self.sentinel_pos]) {
                Ok(s) => &self.buf[1..=s],
                Err(e) => {
                    log::error!("Server COBs decode error: {e}");
                    continue;
                }
            };

            break serde_json::from_slice::<Ret>(data).context(JsonSnafu)?;
        };

        let val = match data {
            Ret::Ok(val) => val,
            Ret::Err(e) => return Err(ClientError::Call { msg: e }),
        };

        Ok(val)
    }
}
