//! Packages: installed counts per manager, fastfetch-style,
//! e.g. "2422 (dpkg), 1 (flatpak)". Only managers with count>0, order dpkg,
//! flatpak, snap. Single Row.
use crate::detect::{Row, Rows};
use std::collections::HashSet;

pub fn detect() -> Rows {
    let mut parts: Vec<String> = Vec::new();

    let dpkg = dpkg_count();
    if dpkg > 0 {
        parts.push(format!("{dpkg} (dpkg)"));
    }

    let flatpak = flatpak_count();
    if flatpak > 0 {
        parts.push(format!("{flatpak} (flatpak)"));
    }

    let snap = snap_count();
    if snap > 0 {
        parts.push(format!("{snap} (snap)"));
    }

    if parts.is_empty() {
        return Vec::new();
    }
    vec![Row::val(parts.join(", "))]
}

/// Count dpkg stanzas whose status line is exactly "install ok installed".
/// Pure file read of /var/lib/dpkg/status, no subprocess.
fn dpkg_count() -> usize {
    let Ok(status) = std::fs::read_to_string("/var/lib/dpkg/status") else {
        return 0;
    };
    status
        .lines()
        .filter(|l| *l == "Status: install ok installed")
        .count()
}

/// Count unique flatpak application ids across the system and user roots.
/// Each subdirectory of an app/ root is one app id.
fn flatpak_count() -> usize {
    let mut ids: HashSet<String> = HashSet::new();
    let mut roots: Vec<String> = vec!["/var/lib/flatpak/app".to_string()];
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            roots.push(format!("{home}/.local/share/flatpak/app"));
        }
    }
    for root in roots {
        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                // Each app id is a directory named after the id.
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    if let Ok(name) = entry.file_name().into_string() {
                        ids.insert(name);
                    }
                }
            }
        }
    }
    ids.len()
}

/// Count /snap/<name>/current symlinks, excluding the "bin" entry.
fn snap_count() -> usize {
    let Ok(entries) = std::fs::read_dir("/snap") else {
        return 0;
    };
    let mut n = 0;
    for entry in entries.flatten() {
        let name = entry.file_name();
        if name == std::ffi::OsStr::new("bin") {
            continue;
        }
        // A snap is present when <name>/current resolves.
        let mut current = entry.path();
        current.push("current");
        if current.symlink_metadata().is_ok() {
            n += 1;
        }
    }
    n
}
