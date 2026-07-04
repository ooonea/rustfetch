//! Battery: charge percent + status, e.g. "87% (Discharging)".
//! Reads /sys/class/power_supply/*/ entries whose "type" is "Battery"
//! (excludes Mains/AC and ucsi-source-psy-* USB-C PD sources).
//! Skipped on desktops with no battery.
use crate::detect::{Row, Rows};

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
