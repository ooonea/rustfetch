//! Battery: charge percent + status, e.g. "87% (Discharging)".
//! Linux reads /sys/class/power_supply/*/ entries whose "type" is "Battery"
//! (excludes Mains/AC and ucsi-source-psy-* USB-C PD sources). macOS parses
//! `pmset -g batt` — the stable power-management CLI; there is no file/sysctl
//! source. Skipped on desktops with no battery.
use crate::detect::{Row, Rows};

#[cfg(target_os = "macos")]
pub fn detect() -> Rows {
    let Some(out) = crate::util::cmd("pmset", &["-g", "batt"]) else {
        return Vec::new();
    };
    parse_pmset(&out)
}

/// One row per "InternalBattery" line, e.g.
/// ` -InternalBattery-0 (id=123)\t87%; discharging; 3:42 remaining ...`
/// -> "87% (Discharging)".
#[cfg(any(target_os = "macos", test))]
fn parse_pmset(out: &str) -> Rows {
    out.lines()
        .filter(|l| l.contains("InternalBattery"))
        .filter_map(|l| {
            let pct_end = l.find('%')?;
            let digits_rev: String = l[..pct_end]
                .chars()
                .rev()
                .take_while(char::is_ascii_digit)
                .collect();
            let pct: String = digits_rev.chars().rev().collect();
            if pct.is_empty() {
                return None;
            }
            // The status word sits between the first two ';' separators.
            let status = l[pct_end + 1..]
                .trim_start_matches(';')
                .split(';')
                .next()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(capitalize)
                .unwrap_or_else(|| "Unknown".to_string());
            Some(Row::val(format!("{pct}% ({status})")))
        })
        .collect()
}

/// Uppercase the first ASCII letter: "discharging" -> "Discharging".
#[cfg(any(target_os = "macos", test))]
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(target_os = "linux")]
pub fn detect() -> Rows {
    let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") else {
        return Vec::new();
    };
    // Collect directory names so output is deterministic (BAT0, BAT1, ...).
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort();

    let mut rows = Vec::new();
    for name in names {
        let base = format!("/sys/class/power_supply/{name}");
        // Only real batteries; skip Mains (AC) and USB-C PD source supplies.
        match crate::util::read_trim(&format!("{base}/type")).as_deref() {
            Some("Battery") => {}
            _ => continue,
        }
        let Some(capacity) = crate::util::read_trim(&format!("{base}/capacity")) else {
            continue;
        };
        let status = crate::util::read_trim(&format!("{base}/status"))
            .unwrap_or_else(|| "Unknown".to_string());
        rows.push(Row::val(format!("{capacity}% ({status})")));
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::parse_pmset;

    #[test]
    fn pmset_lines_parse_percent_and_status() {
        let out = "Now drawing from 'Battery Power'\n \
                   -InternalBattery-0 (id=6357091)\t87%; discharging; 3:42 remaining present: true\n";
        let rows = parse_pmset(out);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].value, "87% (Discharging)");

        // AC power, charged, single-digit percent elsewhere.
        let out = "Now drawing from 'AC Power'\n \
                   -InternalBattery-0 (id=1)\t100%; charged; 0:00 remaining present: true\n";
        assert_eq!(parse_pmset(out)[0].value, "100% (Charged)");

        // A desktop Mac: no InternalBattery lines at all.
        assert!(parse_pmset("Now drawing from 'AC Power'\n").is_empty());
    }
}
