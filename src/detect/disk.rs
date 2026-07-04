//! Disk: used / total (percent). On a normal filesystem this is statfs(2) on
//! `/`. On a ZFS root, statfs only sees the root DATASET (a few GiB of the
//! pool), so we report the whole pool instead — as `zpool list` does — labelled
//! by the pool name.
use crate::detect::{Row, Rows};
use crate::util::{cmd, human_iec, percent};

pub fn detect() -> Rows {
    if let Some(row) = zfs_pool_row() {
        return vec![row];
    }
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

/// When `/` is ZFS, report its pool's allocated/size via `zpool list`.
fn zfs_pool_row() -> Option<Row> {
    let pool = zfs_root_pool()?;
    // `zpool list -Hp -o size,alloc <pool>` -> "<size>\t<alloc>" in bytes.
    let out = cmd("zpool", &["list", "-Hp", "-o", "size,alloc", &pool])?;
    let mut f = out.split_whitespace();
    let size: u64 = f.next()?.parse().ok()?;
    let alloc: u64 = f.next()?.parse().ok()?;
    if size == 0 {
        return None;
    }
    Some(Row::keyed(
        format!("Disk ({pool})"),
        format!(
            "{} / {} ({}%)",
            human_iec(alloc),
            human_iec(size),
            percent(alloc, size)
        ),
    ))
}

/// The pool backing `/` when it is ZFS: the first path component of the root
/// dataset name in /proc/mounts (e.g. `zroot/ROOT/debian` -> `zroot`).
fn zfs_root_pool() -> Option<String> {
    let mounts = std::fs::read_to_string("/proc/mounts").ok()?;
    for line in mounts.lines() {
        let mut f = line.split_whitespace();
        let dev = f.next()?;
        let mnt = f.next()?;
        let fstype = f.next()?;
        if mnt == "/" && fstype == "zfs" {
            return dev.split('/').next().map(str::to_string);
        }
    }
    None
}
