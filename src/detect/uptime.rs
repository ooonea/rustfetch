//! Uptime: humanized, e.g. "6 days, 12 hours, 18 mins". Linux reads
//! /proc/uptime; macOS derives it from the kern.boottime sysctl.
use crate::detect::{Row, Rows};

pub fn detect() -> Rows {
    let secs = uptime_secs().unwrap_or(0);
    if secs == 0 {
        return Vec::new();
    }
    vec![Row::val(humanize(secs))]
}

#[cfg(target_os = "linux")]
fn uptime_secs() -> Option<u64> {
    let raw = crate::util::read_trim("/proc/uptime")?;
    raw.split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .map(|s| s as u64)
}

#[cfg(target_os = "macos")]
fn uptime_secs() -> Option<u64> {
    let boot = crate::sys::boottime_secs()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some(now.saturating_sub(boot))
}

fn humanize(mut s: u64) -> String {
    let days = s / 86_400;
    s %= 86_400;
    let hours = s / 3_600;
    s %= 3_600;
    let mins = s / 60;
    let secs = s % 60;

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
        // Uptime under a minute: show seconds.
        parts.push(format!("{secs} sec{}", plural(secs)));
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

#[cfg(test)]
mod tests {
    use super::humanize;

    #[test]
    fn humanize_pluralizes_and_composes() {
        assert_eq!(humanize(59), "59 secs"); // under a minute
        assert_eq!(humanize(1), "1 sec");
        assert_eq!(humanize(60), "1 min");
        assert_eq!(humanize(3600), "1 hour");
        assert_eq!(humanize(2 * 86_400), "2 days");
        assert_eq!(humanize(90_000), "1 day, 1 hour");
        assert_eq!(humanize(86_400 + 3600 + 60), "1 day, 1 hour, 1 min");
    }
}
