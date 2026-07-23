//! Kernel: release string, e.g. "6.12.94+deb13-amd64" or "Darwin 25.4.0".
use crate::detect::{Row, Rows};

#[cfg(target_os = "linux")]
pub fn detect() -> Rows {
    match crate::util::read_trim("/proc/sys/kernel/osrelease") {
        Some(v) => vec![Row::val(v)],
        None => Vec::new(),
    }
}

/// macOS: a bare Darwin release number would read like nonsense next to the
/// macOS version, so prefix the kernel name (matching fastfetch).
#[cfg(target_os = "macos")]
pub fn detect() -> Rows {
    let Some(release) = crate::sys::sysctl_string("kern.osrelease") else {
        return Vec::new();
    };
    let name = crate::sys::sysctl_string("kern.ostype").unwrap_or_else(|| "Darwin".to_string());
    vec![Row::val(format!("{name} {release}"))]
}
