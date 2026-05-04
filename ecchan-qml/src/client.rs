use std::{
    io::{self, Cursor, ErrorKind, Read, Write},
    os::unix::net::UnixStream,
};

use ecchan_ipc::{
    method::Method,
    ret::{Ret, RetVal},
};
use serde::Deserialize;
use snafu::{ResultExt as _, Snafu};

use crate::q_warning;

#[derive(Debug, Snafu)]
pub enum ClientError {
    #[snafu(display("{msg}"))]
    Call { msg: String },

    #[snafu(display("{source}"))]
    Json { source: serde_json::Error },

    #[snafu(display("{source}"))]
    Io { source: io::Error },

    #[snafu(display("Read reached EOF"))]
    Eof,
}

pub struct Client {
    conn: UnixStream,
    buf: Vec<u8>,
}

impl Client {
    pub fn new(path: &str) -> Result<Self, ClientError> {
        let conn = UnixStream::connect(path).context(IoSnafu)?;
        let this = Self {
            conn,
            buf: vec![0; 1024],
        };

        Ok(this)
    }

    pub fn call(&mut self, method: &Method) -> Result<RetVal<'static>, ClientError> {
        self.buf.clear();

        let mut data = serde_json::to_string(method).context(JsonSnafu)?;
        data.push('\n');

        self.conn.write_all(data.as_bytes()).context(IoSnafu)?;

        let mut buf = [0; 1024];
        let mut res = Ok(RetVal::Unit);
        let mut should_empty_buffer = false;
        loop {
            match self.conn.read(&mut buf) {
                Ok(0) if should_empty_buffer => break,
                Ok(0) => return Err(ClientError::Eof),

                Ok(n) => {
                    if should_empty_buffer {
                        continue;
                    }

                    let msg = &buf[..n];

                    self.buf.extend_from_slice(msg);

                    // accumulate full message
                    let Some(newline_pos) = self.buf.iter().position(|b| *b == b'\n') else {
                        continue;
                    };

                    res = {
                        let data = Cursor::new(self.buf.drain(..=newline_pos));
                        // work around lifetime is not general enough error
                        let mut deserializer = serde_json::Deserializer::from_reader(data);
                        Ret::<'static>::deserialize(&mut deserializer).context(JsonSnafu)?
                    };

                    if !self.buf.is_empty() {
                        q_warning!("call: buf should not have extra data!!: {buf:?}");
                        should_empty_buffer = true;
                        continue;
                    }
                }

                Err(e) => match e.kind() {
                    ErrorKind::WouldBlock if should_empty_buffer => break,
                    ErrorKind::WouldBlock => continue,
                    _ => return Err(e).context(IoSnafu),
                },
            }
        }

        res.map_err(|e| ClientError::Call { msg: e })
    }
}
