//! OS: name + version + codename + arch,
//! e.g. "Debian GNU/Linux 13.5 (trixie) x86_64" or "macOS 26.1 (Tahoe) arm64".
use crate::detect::{Row, Rows};

#[cfg(target_os = "linux")]
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

    let arch = crate::detect::arch();

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

/// macOS: version + build from SystemVersion.plist — the same file `sw_vers`
/// reads, present on every macOS — plus the marketing codename we map locally.
#[cfg(target_os = "macos")]
pub fn detect() -> Rows {
    let plist = std::fs::read_to_string("/System/Library/CoreServices/SystemVersion.plist")
        .unwrap_or_default();
    let name = plist_value(&plist, "ProductName").unwrap_or_else(|| "macOS".to_string());
    let version = plist_value(&plist, "ProductVersion").unwrap_or_default();

    let mut base = name;
    if !version.is_empty() {
        base.push(' ');
        base.push_str(&version);
    }
    if let Some(code) = codename(&version) {
        base.push_str(&format!(" ({code})"));
    }
    vec![Row::val(format!("{base} {}", crate::detect::arch()))]
}

/// Value of `<key>K</key><string>V</string>` in a plist, by simple scanning —
/// SystemVersion.plist is flat, tiny and Apple-generated, so a real XML parser
/// would be overkill.
#[cfg(any(target_os = "macos", test))]
fn plist_value(xml: &str, key: &str) -> Option<String> {
    let pos = xml.find(&format!("<key>{key}</key>"))?;
    let rest = &xml[pos..];
    let start = rest.find("<string>")? + "<string>".len();
    let end = rest[start..].find("</string>")?;
    let v = rest[start..start + end].trim();
    if v.is_empty() {
        None
    } else {
        Some(v.to_string())
    }
}

/// Marketing codename for a macOS version, from the major number (Apple's
/// scheme since 11; 26 followed 15 with the switch to year-based numbering).
#[cfg(any(target_os = "macos", test))]
fn codename(version: &str) -> Option<&'static str> {
    let major: u32 = version.split('.').next()?.parse().ok()?;
    Some(match major {
        11 => "Big Sur",
        12 => "Monterey",
        13 => "Ventura",
        14 => "Sonoma",
        15 => "Sequoia",
        26 => "Tahoe",
        _ => return None,
    })
}

/// Debian ships a fuller version in /etc/debian_version (e.g. "13.5") than
/// os-release VERSION_ID ("13"), so prefer it — but ONLY on Debian itself: on
/// derivatives (Ubuntu, Mint, ...) the same file holds the BASE distro's
/// codename (e.g. "trixie/sid"), not the derivative's own version.
#[cfg(any(target_os = "linux", test))]
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
    use super::{codename, effective_version, plist_value};

    #[test]
    fn plist_value_scans_flat_systemversion() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<dict>
        <key>ProductBuildVersion</key>
        <string>25E246</string>
        <key>ProductName</key>
        <string>macOS</string>
        <key>ProductVersion</key>
        <string>26.4</string>
</dict>"#;
        assert_eq!(plist_value(xml, "ProductName").as_deref(), Some("macOS"));
        assert_eq!(plist_value(xml, "ProductVersion").as_deref(), Some("26.4"));
        assert_eq!(plist_value(xml, "Missing"), None);
    }

    #[test]
    fn codename_maps_major_version() {
        assert_eq!(codename("15.1"), Some("Sequoia"));
        assert_eq!(codename("26.4"), Some("Tahoe"));
        assert_eq!(codename("14"), Some("Sonoma"));
        assert_eq!(codename("9.99"), None);
        assert_eq!(codename(""), None);
    }

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
