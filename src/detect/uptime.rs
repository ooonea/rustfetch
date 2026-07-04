//! Uptime: humanized from /proc/uptime, e.g. "6 days, 12 hours, 18 mins".
use crate::detect::{Row, Rows};

pub fn detect() -> Rows {
    let Some(raw) = crate::util::read_trim("/proc/uptime") else {
        return Vec::new();
    };
    let secs = raw
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0) as u64;
    if secs == 0 {
        return Vec::new();
    }
    vec![Row::val(humanize(secs))]
}

fn humanize(mut s: u64) -> String {
    let days = s / 86_400;
    s %= 86_400;
    let hours = s / 3_600;
    s %= 3_600;
    let mins = s / 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{days} day{}", plural(days)));
    }
    if hours > 0 {
        parts.push(format!("{hours} hour{}", plural(hours)));
    }
    if mins > 0 {
        parts.push(format!("{mins} min{}", plural(mins)));
    }
    if parts.is_empty() {
        // Uptime under a minute.
        parts.push(format!("{mins} min{}", plural(mins)));
    }
    parts.join(", ")
}

fn plural(n: u64) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}
