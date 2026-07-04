//! Swap: used / total (percent). Skipped entirely if no swap is configured.
use crate::detect::{Row, Rows};
use crate::util::{human_iec, percent};

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
    if total == 0 {
        return Vec::new();
    }
    let used = total.saturating_sub(free);
    vec![Row::val(format!(
        "{} / {} ({}%)",
        human_iec(used),
        human_iec(total),
        percent(used, total)
    ))]
}

fn parse_kb(s: &str) -> u64 {
    s.split_whitespace()
        .next()
        .and_then(|n| n.parse::<u64>().ok())
        .map(|kb| kb * 1024)
        .unwrap_or(0)
}
