#[cfg(feature = "tokio-server")]
pub mod tokio;
#[cfg(feature = "compio-server")]
pub mod compio;

pub const DEFAULT_LIMIT_HANDLE_NUM: usize = 2048;
pub const DEFAULT_TCP_BACKLOG: u32 = 4096;