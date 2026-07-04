//! Memory: used / total (percent), e.g. "14.98 GiB / 62.61 GiB (24%)".
//! Used = MemTotal - MemAvailable, minus the reclaimable ZFS ARC. The ARC is
//! kernel-slab cache that MemAvailable does not count as available, so without
//! this correction a ZFS-on-root box massively over-reports used memory. This
//! matches fastfetch/htop-with-ZFS.
use crate::detect::{Row, Rows};
use crate::util::{human_iec, percent};

pub fn detect() -> Rows {
    let Ok(info) = std::fs::read_to_string("/proc/meminfo") else {
        return Vec::new();
    };
    let mut total = 0u64;
    let mut available = 0u64;
    for line in info.lines() {
        if let Some(v) = line.strip_prefix("MemTotal:") {
            total = parse_kb(v);
        } else if let Some(v) = line.strip_prefix("MemAvailable:") {
            available = parse_kb(v);
        }
    }
    if total == 0 {
        return Vec::new();
    }

    let mut used = total.saturating_sub(available);
    if let Some(reclaimable) = zfs_arc_reclaimable() {
        used = used.saturating_sub(reclaimable);
    }

    vec![Row::val(format!(
        "{} / {} ({}%)",
        human_iec(used),
        human_iec(total),
        percent(used, total)
    ))]
}

/// Reclaimable ZFS ARC bytes: the ARC `size` above its `c_min` floor (the ARC
/// cannot shrink below `c_min`). `None` when there is no ZFS ARC or it is
/// already at/below the floor.
fn zfs_arc_reclaimable() -> Option<u64> {
    let stats = std::fs::read_to_string("/proc/spl/kstat/zfs/arcstats").ok()?;
    let mut size = 0u64;
    let mut c_min = 0u64;
    for line in stats.lines() {
        // Each line is: "<name> <type> <data>"; we want <data>.
        let mut fields = line.split_whitespace();
        match fields.next() {
            Some("size") => size = fields.nth(1).and_then(|d| d.parse().ok()).unwrap_or(0),
            Some("c_min") => c_min = fields.nth(1).and_then(|d| d.parse().ok()).unwrap_or(0),
            _ => {}
        }
    }
    (size > c_min).then_some(size - c_min)
}

fn parse_kb(s: &str) -> u64 {
    s.split_whitespace()
        .next()
        .and_then(|n| n.parse::<u64>().ok())
        .map(|kb| kb * 1024)
        .unwrap_or(0)
}
