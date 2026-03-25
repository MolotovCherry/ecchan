#[cfg(not(target_os = "linux"))]
compile_error!("Only Linux is supported");

mod ec;
mod fw;

pub use ec::*;
