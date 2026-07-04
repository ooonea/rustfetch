//! Shell: parent shell + version like fastfetch, e.g. "zsh 5.9".
//! Walks the process-parent chain from our own ppid up to the first ancestor
//! whose `comm` is a known shell, then reads its `--version`.
use crate::detect::{Row, Rows};

/// Interactive/login shells we recognise as a "shell" ancestor.
const SHELLS: &[&str] = &[
    "bash", "zsh", "fish", "dash", "sh", "ksh", "tcsh", "csh", "nu", "elvish", "xonsh",
];

pub fn detect() -> Rows {
    // Prefer the real parent-process shell; fall back to $SHELL's basename.
    let Some(shell) = find_shell_ancestor().or_else(shell_from_env) else {
        return Vec::new();
    };
    let value = match shell_version(&shell) {
        Some(v) => format!("{shell} {v}"),
        None => shell,
    };
    vec![Row::val(value)]
}

/// Climb the parent chain from our own ppid; return the `comm` of the first
/// ancestor that is a known shell. `None` if none is found before pid 1.
fn find_shell_ancestor() -> Option<String> {
    let mut pid = ppid_of("self")?;
    let mut guard = 0u32;
    while pid > 1 && guard < 64 {
        // comm may carry a leading '-' for login shells, e.g. "-zsh".
        if let Some(comm) = crate::util::read_trim(&format!("/proc/{pid}/comm")) {
            let name = comm.strip_prefix('-').unwrap_or(&comm);
            if SHELLS.contains(&name) {
                return Some(name.to_string());
            }
        }
        let next = ppid_of(&pid.to_string())?;
        if next == pid {
            break; // cycle guard
        }
        pid = next;
        guard += 1;
    }
    None
}

/// Read the ppid field from `/proc/<who>/stat` (`who` = "self" or a pid string).
/// The process name can contain spaces/parens, so split after the LAST ')':
/// the remainder is "state ppid pgrp ..." — ppid is the 2nd field.
fn ppid_of(who: &str) -> Option<u32> {
    let stat = crate::util::read_trim(&format!("/proc/{who}/stat"))?;
    let after = &stat[stat.rfind(')')? + 1..];
    after.split_whitespace().nth(1)?.parse().ok()
}

/// Extract a version like "5.9" or "5.2.37" from `<shell> --version`.
/// Takes the first whitespace token that starts with a digit, then keeps the
/// leading run of digits and dots (drops trailing "(1)-release" etc.).
fn shell_version(shell: &str) -> Option<String> {
    let out = crate::util::cmd(shell, &["--version"])?;
    let line = out.lines().next()?;
    for token in line.split_whitespace() {
        if token.starts_with(|c: char| c.is_ascii_digit()) {
            let ver: String = token
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if !ver.is_empty() {
                return Some(ver);
            }
        }
    }
    None
}

/// Fallback: basename of the `$SHELL` environment variable.
fn shell_from_env() -> Option<String> {
    let sh = std::env::var("SHELL").ok()?;
    let base = sh.rsplit('/').next()?;
    if base.is_empty() {
        None
    } else {
        Some(base.to_string())
    }
}
