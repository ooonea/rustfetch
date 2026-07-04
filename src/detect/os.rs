//! OS: distro name + version + codename + arch,
//! e.g. "Debian GNU/Linux 13.5 (trixie) x86_64".
use crate::detect::{Row, Rows};

pub fn detect() -> Rows {
    let mut name = String::new();
    let mut version_id = String::new();
    let mut codename = String::new();
    let mut pretty = String::new();

    if let Ok(s) = std::fs::read_to_string("/etc/os-release") {
        for line in s.lines() {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let v = v.trim().trim_matches('"');
            match k {
                "NAME" => name = v.to_string(),
                "VERSION_ID" => version_id = v.to_string(),
                "VERSION_CODENAME" => codename = v.to_string(),
                "PRETTY_NAME" => pretty = v.to_string(),
                _ => {}
            }
        }
    }

    // Debian ships a fuller version in /etc/debian_version (e.g. "13.5") than
    // os-release VERSION_ID ("13"); prefer it when present.
    let version = std::fs::read_to_string("/etc/debian_version")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or(version_id);

    let arch = std::env::consts::ARCH;

    let base = if !name.is_empty() {
        let mut b = name;
        if !version.is_empty() {
            b.push(' ');
            b.push_str(&version);
        }
        if !codename.is_empty() {
            b.push_str(&format!(" ({codename})"));
        }
        b
    } else if !pretty.is_empty() {
        pretty
    } else {
        "Linux".to_string()
    };

    vec![Row::val(format!("{base} {arch}"))]
}
