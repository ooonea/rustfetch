//! CPU: cleaned model name @ max frequency,
//! e.g. "Intel(R) Core(TM) i7-9850H @ 4.60 GHz".
use crate::detect::{Row, Rows};

pub fn detect() -> Rows {
    let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") else {
        return Vec::new();
    };

    let mut model = String::new();
    for line in cpuinfo.lines() {
        if let Some((k, v)) = line.split_once(':') {
            if k.trim() == "model name" {
                model = v.trim().to_string();
                break;
            }
        }
    }
    if model.is_empty() {
        return Vec::new();
    }

    let name = clean_model(&model);
    let value = match max_freq_ghz() {
        Some(f) => format!("{name} @ {f:.2} GHz"),
        None => name,
    };
    vec![Row::val(value)]
}

/// Drop the trailing " CPU @ x.xxGHz" but keep the vendor "(R)"/"(TM)" marks,
/// matching fastfetch's `{name}` field.
fn clean_model(s: &str) -> String {
    let base = s.split(" @ ").next().unwrap_or(s).trim();
    let base = base.strip_suffix(" CPU").unwrap_or(base);
    base.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn max_freq_ghz() -> Option<f64> {
    crate::util::read_trim("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq")
        .and_then(|s| s.parse::<f64>().ok())
        .map(|khz| khz / 1_000_000.0)
}
