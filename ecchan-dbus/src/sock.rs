use std::{
    cmp::max,
    io::{self, ErrorKind, Read, Write},
    iter,
    os::unix::net::UnixStream,
    sync::Arc,
};

use ecchan_ipc::{
    method::Method,
    ret::{Ret, RetVal},
};
use polonius_the_crab::prelude::*;
use sayuri::sync::Mutex;
use snafu::{ResultExt as _, Snafu};

#[derive(Debug, Snafu)]
pub enum ClientError {
    #[snafu(display("{msg}"))]
    Call { msg: String },

    #[snafu(display("{source}"))]
    Json { source: serde_json::Error },

    #[snafu(display("{source}"))]
    Io { source: io::Error },
}

pub struct Client {
    conn: Arc<Mutex<Option<UnixStream>>>,
    buf: Vec<u8>,
    encode_buf: Vec<u8>,
    sentinel_pos: usize,
}

impl Client {
    pub fn new() -> io::Result<Self> {
        let conn = Self::connect().ok();
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            buf: vec![0; 1024],
            encode_buf: vec![0; 1024],
            sentinel_pos: 0,
        })
    }

    fn connect() -> io::Result<UnixStream> {
        UnixStream::connect(ecchan_ipc::get_socket_path())
    }

    pub fn call(&mut self, call: Method) -> Result<RetVal<'_>, ClientError> {
        let mut this = self; // to make it work with polonius macro
        // we need to use/set the conn, but we can't call a mutable method at the same time,
        // so we'll grab an owned copy of this and reset it back as needed
        let conn = this.conn.clone(); // get free of self lifetime
        let mut lock = conn.lock();

        // We have an unfortunate issue here; due to Ok case returning a borrowed value of self,
        // the borrow checker extends the borrow to the entire function, disallowing us to call
        // a mutable method again, despite the fact that it is sound to call it again in the Err
        // case because it doesn't borrow from self.
        //
        // The lifetime infects the function. This works around the infected lifetime in the Ok case
        // and lets us call mutable methods again
        let error = polonius!(|this| -> Result<RetVal<'polonius>, ClientError> {
            match lock.as_mut() {
                Some(conn) => match this._call(conn, &call) {
                    // return value containing infected lifetime
                    v @ Ok(_) => polonius_return!(v),
                    Err(e) => Some(e),
                },

                None => None,
            }
        });

        // handle the errors free of the infected lifetime

        match error {
            Some(e) => match e {
                // io failure
                ClientError::Io { source } => {
                    log::error!("server conn failed: {source}");

                    match Self::connect() {
                        // reconnected successfully, so we can retry the call
                        Ok(mut s) => {
                            log::info!("reconnected to server");

                            match this._call(&mut s, &call) {
                                Ok(v) => {
                                    *lock = Some(s);
                                    Ok(v)
                                }

                                e @ Err(ClientError::Io { .. }) => {
                                    *lock = None;
                                    e
                                }

                                e @ Err(_) => {
                                    *lock = Some(s);
                                    e
                                }
                            }
                        }

                        // can't connect for some reason; return same error as before
                        Err(e) => {
                            log::error!("reconnection failed: {e}");
                            // socket is useless since we couldn't reconnect
                            *lock = None;
                            Err(ClientError::Io { source })
                        }
                    }
                }

                // all other errors, just return them
                e => {
                    log::error!("call error: {e}");
                    Err(e)
                }
            },

            None => match Self::connect() {
                // reconnected successfully, so we can retry the call
                Ok(mut s) => {
                    log::info!("reconnected to server");
                    match this._call(&mut s, &call) {
                        Ok(v) => {
                            *lock = Some(s);
                            Ok(v)
                        }

                        Err(e @ ClientError::Io { .. }) => {
                            log::error!("server conn failed: {e}");
                            *lock = None;
                            Err(e)
                        }

                        Err(e) => {
                            log::error!("server conn failed: {e}");
                            *lock = Some(s);
                            Err(e)
                        }
                    }
                }

                // can't connect for some reason; return same error as before
                Err(e) => {
                    log::error!("reconnection failed: {e}");
                    Err(ClientError::Io { source: e })
                }
            },
        }
    }

    fn _call<'a>(
        &'a mut self,
        conn: &mut UnixStream,
        call: &Method,
    ) -> Result<RetVal<'a>, ClientError> {
        self.buf.clear();

        let data = serde_json::to_string(call).context(JsonSnafu)?;

        log::debug!("sending req: {data}");

        let cap = max(self.encode_buf.len(), cobs::max_encoding_length(data.len()));
        if cap > self.encode_buf.len() {
            let count = cap - self.encode_buf.len();
            self.encode_buf.extend(iter::repeat_n(0, count));
        }

        let size = cobs::encode_including_sentinels(data.as_bytes(), &mut self.encode_buf);
        let encoded = &self.encode_buf[..size];

        conn.write_all(encoded).context(IoSnafu)?;

        let mut buf = [0; 1024];

        let mut zeroes = 0;
        let data = loop {
            match conn.read(&mut buf) {
                Ok(n) => match n {
                    0 => {
                        return Err(ClientError::Io {
                            source: io::Error::new(ErrorKind::BrokenPipe, "no data left in socket"),
                        });
                    }

                    n => {
                        let msg = &buf[..n];

                        // accumulate full message
                        self.buf.extend_from_slice(msg);

                        // count zeroes
                        zeroes += msg.iter().filter(|b| **b == 0).count();

                        // ensure we have 2 zeroes (begin and end sentinel)
                        if zeroes < 2 {
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

            let t = serde_json::from_slice::<Ret>(data).context(JsonSnafu)?;
            log::debug!("got server reply: {}", str::from_utf8(data).unwrap());
            break t;
        };

        let val = match data {
            Ret::Ok(val) => val,
            Ret::Err(e) => return Err(ClientError::Call { msg: e }),
        };

        Ok(val)
    }
}
