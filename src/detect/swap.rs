//! Swap: used / total (percent). Skipped entirely if no swap is configured
//! (macOS with no swap files yet reports a 0 total the same way).
use crate::detect::{Row, Rows};
use crate::util::{human_iec, percent};

#[cfg(target_os = "linux")]
pub fn detect() -> Rows {
    let Ok(info) = std::fs::read_to_string("/proc/meminfo") else {
        return Vec::new();
    };
    let mut total = 0u64;
    let mut free = 0u64;
    for line in info.lines() {
        if let Some(v) = line.strip_prefix("SwapTotal:") {
            total = parse_kb(v);
        } else if let Some(v) = line.strip_prefix("SwapFree:") {
            free = parse_kb(v);
        }
    }
    row(total.saturating_sub(free), total)
}

#[cfg(target_os = "macos")]
pub fn detect() -> Rows {
    let Some((total, used)) = crate::sys::swap_usage() else {
        return Vec::new();
    };
    row(used, total)
}

fn row(used: u64, total: u64) -> Rows {
    if total == 0 {
        return Vec::new();
    }
    vec![Row::val(format!(
        "{} / {} ({}%)",
        human_iec(used),
        human_iec(total),
        percent(used, total)
    ))]
}

#[cfg(target_os = "linux")]
fn parse_kb(s: &str) -> u64 {
    s.split_whitespace()
        .next()
        .and_then(|n| n.parse::<u64>().ok())
        .map(|kb| kb * 1024)
        .unwrap_or(0)
}
