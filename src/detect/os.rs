//! OS: distro name + version + codename + arch,
//! e.g. "Debian GNU/Linux 13.5 (trixie) x86_64".
use crate::detect::{Row, Rows};

pub fn detect() -> Rows {
    let mut id = String::new();
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
                "ID" => id = v.to_string(),
                "NAME" => name = v.to_string(),
                "VERSION_ID" => version_id = v.to_string(),
                "VERSION_CODENAME" => codename = v.to_string(),
                "PRETTY_NAME" => pretty = v.to_string(),
                _ => {}
            }
        }
    }

    let debian_version = std::fs::read_to_string("/etc/debian_version")
        .ok()
        .map(|s| s.trim().to_string());
    let version = effective_version(&id, version_id, debian_version);

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

/// Debian ships a fuller version in /etc/debian_version (e.g. "13.5") than
/// os-release VERSION_ID ("13"), so prefer it — but ONLY on Debian itself: on
/// derivatives (Ubuntu, Mint, ...) the same file holds the BASE distro's
/// codename (e.g. "trixie/sid"), not the derivative's own version.
fn effective_version(id: &str, version_id: String, debian_version: Option<String>) -> String {
    if id == "debian" {
        if let Some(v) = debian_version.filter(|s| !s.is_empty()) {
            return v;
        }
    }
    version_id
}

#[cfg(test)]
mod tests {
    use super::effective_version;

    #[test]
    fn debian_version_only_refines_debian_itself() {
        let s = |x: &str| x.to_string();
        // Genuine Debian: the fuller point version wins.
        assert_eq!(
            effective_version("debian", s("13"), Some(s("13.5"))),
            "13.5"
        );
        // Derivative: /etc/debian_version holds the base codename — ignore it.
        assert_eq!(
            effective_version("ubuntu", s("24.04"), Some(s("trixie/sid"))),
            "24.04"
        );
        // Missing or empty file: keep VERSION_ID.
        assert_eq!(effective_version("debian", s("13"), None), "13");
        assert_eq!(effective_version("debian", s("13"), Some(s(""))), "13");
    }
}
