//! Kernel: release string, e.g. "6.12.94+deb13-amd64".
use crate::detect::{Row, Rows};

pub fn detect() -> Rows {
    match crate::util::read_trim("/proc/sys/kernel/osrelease") {
        Some(v) => vec![Row::val(v)],
        None => Vec::new(),
    }
}
