//! macOS platform layer — `extern "C"` bindings to libSystem, no libc crate.
//!
//! Raw syscalls are not a stable ABI on macOS: the only supported kernel
//! interface is libSystem, which Rust's std already links unconditionally on
//! darwin targets. Binding the handful of functions we need directly keeps the
//! crate at zero external dependencies here too.
#![allow(dead_code)]

use super::DiskUsage;

/// Disk usage of the filesystem containing `path` (placeholder — implemented
/// with the port).
pub fn disk_usage(_path: &str) -> Option<DiskUsage> {
    None
}

/// Terminal width in columns (placeholder).
pub fn term_width() -> usize {
    80
}

/// Whether stdout is a terminal (placeholder).
pub fn stdout_is_tty() -> bool {
    false
}

/// The node name (placeholder).
pub fn hostname() -> Option<String> {
    None
}

/// Parent pid and short command name of `pid` (placeholder).
pub fn ppid_comm(_pid: u32) -> Option<(u32, String)> {
    None
}
