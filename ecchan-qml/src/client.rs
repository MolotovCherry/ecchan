use std::{
    io::{self, Cursor, ErrorKind, Read, Write},
    os::unix::net::UnixStream,
    pin::Pin,
    sync::mpsc::{Sender, channel},
    thread,
};

use cxx_qt::{CxxQtThread, ThreadingQueueError};
use ecchan_ipc::{
    method::Method,
    ret::{Ret, RetVal},
};
use serde::Deserialize;
use snafu::{ResultExt as _, Snafu};

use crate::{q_warning, qml::qobject::EcchanClient};

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

#[allow(clippy::type_complexity)]
enum Cb {
    Normal {
        method: Method<'static>,
        cb: Box<
            dyn FnOnce(Pin<&mut EcchanClient>, Result<RetVal<'static>, ClientError>)
                + Send
                + 'static,
        >,
    },

    Queued {
        cb: Box<dyn FnOnce(Pin<&mut EcchanClient>) + Send + 'static>,
    },
}

pub struct Client {
    tx: Sender<Cb>,
}

impl Client {
    pub fn new(path: &str, qthread: CxxQtThread<EcchanClient>) -> Result<Self, ClientError> {
        let conn = UnixStream::connect(path).context(IoSnafu)?;
        let (tx, rx) = channel::<Cb>();

        thread::spawn(move || {
            let mut inner = ClientInner {
                conn,
                buf: vec![0; 1024],
            };

            loop {
                let Ok(cb) = rx.recv() else {
                    break;
                };

                let mut should_exit = false;

                let res = match cb {
                    Cb::Queued { cb } => qthread.queue(|ctx| {
                        cb(ctx);
                    }),

                    Cb::Normal { method, cb } => {
                        let res = inner.call(method);

                        should_exit = matches!(res, Err(ClientError::Io { .. } | ClientError::Eof));

                        qthread.queue(|ctx| {
                            cb(ctx, res);
                        })
                    }
                };

                match res {
                    Ok(_) => (),
                    Err(ThreadingQueueError::ObjectDestroyed) => should_exit = true,
                    Err(e @ ThreadingQueueError::InvokeMethodFailed) => {
                        q_warning!("QThread failed to execute cb: {e}")
                    }
                    Err(e @ ThreadingQueueError::Unknown) => {
                        q_warning!("QThread failed to execute cb: {e}");
                        should_exit = true;
                    }
                    Err(e) => q_warning!("QThread failed to execute cb: {e}"),
                }

                if should_exit {
                    break;
                }
            }
        });

        let this = Self { tx };
        Ok(this)
    }

    pub fn call(
        &self,
        method: Method<'static>,
        cb: impl FnOnce(Pin<&mut EcchanClient>, Result<RetVal<'static>, ClientError>) + Send + 'static,
    ) {
        let cb = Cb::Normal {
            method,
            cb: Box::new(cb),
        };

        self.tx.send(cb).expect("rx should not be dropped");
    }

    /// A dummy call whose sole purpose is to run the callback; however it runs it in queued order
    pub fn queued_call(&self, cb: impl FnOnce(Pin<&mut EcchanClient>) + Send + 'static) {
        let cb = Cb::Queued { cb: Box::new(cb) };
        self.tx.send(cb).expect("rx should not be dropped");
    }
}

struct ClientInner {
    conn: UnixStream,
    buf: Vec<u8>,
}

impl ClientInner {
    fn call(&mut self, method: Method) -> Result<RetVal<'static>, ClientError> {
        self.buf.clear();

        let mut data = serde_json::to_string(&method).context(JsonSnafu)?;
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

                    break;
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
