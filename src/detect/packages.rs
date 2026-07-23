//! Packages: installed counts per manager, fastfetch-style,
//! e.g. "2422 (dpkg), 1 (flatpak)" or "291 (brew), 13 (brew-cask)". Only
//! managers with count>0. Single Row. Everything is counted from the
//! managers' on-disk layouts — no package-manager subprocesses.
use crate::detect::{Row, Rows};
#[cfg(target_os = "linux")]
use std::collections::HashSet;

#[cfg(target_os = "macos")]
pub fn detect() -> Rows {
    // Homebrew keeps one directory per installed formula (Cellar) or cask
    // (Caskroom); $HOMEBREW_PREFIX overrides the per-arch default prefix.
    let prefix = std::env::var("HOMEBREW_PREFIX")
        .ok()
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| {
            if cfg!(target_arch = "aarch64") {
                "/opt/homebrew".to_string()
            } else {
                "/usr/local".to_string()
            }
        });

    let mut parts: Vec<String> = Vec::new();
    for (dir, label) in [
        (format!("{prefix}/Cellar"), "brew"),
        (format!("{prefix}/Caskroom"), "brew-cask"),
        // MacPorts: one directory per installed port.
        ("/opt/local/var/macports/software".to_string(), "macports"),
    ] {
        let n = dir_count(&dir);
        if n > 0 {
            parts.push(format!("{n} ({label})"));
        }
    }
    if parts.is_empty() {
        return Vec::new();
    }
    vec![Row::val(parts.join(", "))]
}

/// Non-hidden subdirectories of `path` (0 if it does not exist).
#[cfg(target_os = "macos")]
fn dir_count(path: &str) -> usize {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .count()
}

#[cfg(target_os = "linux")]
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
#[cfg(target_os = "linux")]
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
#[cfg(target_os = "linux")]
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
#[cfg(target_os = "linux")]
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
