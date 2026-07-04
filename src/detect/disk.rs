//! Disk (/): used / total (percent) via statfs(2) on the root mount.
//! Works on ZFS-on-root (unicorn) as on any other filesystem.
use crate::detect::{Row, Rows};
use crate::util::{human_iec, percent};

pub fn detect() -> Rows {
    let Some(d) = crate::sys::disk_usage("/") else {
        return Vec::new();
    };
    vec![Row::val(format!(
        "{} / {} ({}%)",
        human_iec(d.used),
        human_iec(d.total),
        percent(d.used, d.total)
    ))]
}
