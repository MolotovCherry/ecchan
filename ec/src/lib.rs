#[cfg(not(target_os = "linux"))]
compile_error!("Only Linux is supported");

mod ec;
mod fw;
mod models;

pub use ec::*;
