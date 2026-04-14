// Copied from below under MIT license
// https://github.com/WLBF/single-instance

use std::os::fd::{AsRawFd, OwnedFd};

use nix::{
    Result,
    sys::socket::{self, Backlog, UnixAddr},
};

/// A struct representing one running instance.
pub struct SingleInstance {
    maybe_sock: Option<OwnedFd>,
}

impl SingleInstance {
    /// Returns a new SingleInstance object.
    pub fn new(name: &str) -> Result<Self> {
        let addr = UnixAddr::new_abstract(name.as_bytes())?;
        let sock = socket::socket(
            socket::AddressFamily::Unix,
            socket::SockType::Stream,
            // If we fork and exec, then make sure the child process doesn't
            // hang on to this file descriptor.
            socket::SockFlag::SOCK_CLOEXEC,
            None,
        )?;

        let maybe_sock = match socket::bind(sock.as_raw_fd(), &addr) {
            Ok(()) => {
                socket::listen(&sock, Backlog::new(1).unwrap()).unwrap();
                Some(sock)
            }
            Err(nix::errno::Errno::EADDRINUSE) => None,
            Err(e) => return Err(e),
        };

        Ok(Self { maybe_sock })
    }

    /// Returns whether this instance is single.
    pub fn is_single(&self) -> bool {
        self.maybe_sock.is_some()
    }
}
