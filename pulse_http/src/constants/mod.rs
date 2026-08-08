pub const DEFAULT_MAX_BODY_SIZE: usize = 1024 * 1024;
pub const MAX_REQUESTS_PER_CONNECTION: u32 = 100;
pub const DEFAULT_READ_TIMEOUT_SECS: u64 = 30;

pub const MAX_HEADER_SIZE: usize = 32 * 1024;
pub const HEADER_END: &[u8] = b"\r\n\r\n";
pub const SHUTDOWN_GRACE_SECS: u64 = 30;
pub const DEFAULT_MAX_CONNECTIONS: u16 = 2;
