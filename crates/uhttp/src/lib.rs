#![allow(clippy::unused_io_amount)]
#![allow(dead_code)]
pub mod body;
pub mod constants;
pub mod http1;
mod result;

#[cfg(feature = "router")]
pub mod router;
pub use http::StatusCode;
pub use result::*;
pub use tokio::io::AsyncReadExt;
pub use tokio::io::AsyncWriteExt;
