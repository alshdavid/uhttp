//! Filesystem abstraction used by the file server.
//!
//! Re-exported so consumers can supply their own [`AsyncFileSystem`]
//! implementation without depending on `dyn_fs` directly.

pub use dyn_fs::asynch::*;
