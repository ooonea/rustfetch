//! Terminal: terminal emulator (+ version if cheap), e.g. "kitty 0.41.1".
//! Primary source: walk the parent-process chain and pick the first ancestor
//! that is a known terminal (shells and purefetch itself are simply skipped).
//! Fallback: environment hints ($TERM, $KITTY_WINDOW_ID, $TERM_PROGRAM, ...),
//! needed when we run under `bash -c`/`script` and the chain has no terminal.
use crate::detect::{Row, Rows};

pub fn detect() -> Rows {
    let term = match walk_parents().or_else(from_env) {
        Some(t) => t,
        None => return Vec::new(),
    };
    let value = match version_of(term).or_else(|| env_version(term)) {
        Some(v) => format!("{term} {v}"),
        None => term.to_string(),
    };
    vec![Row::val(value)]
}

/// Climb the parent chain from our own ppid; return the first known terminal.
fn walk_parents() -> Option<&'static str> {
    let mut pid = crate::sys::ppid_comm(std::process::id())?.0;
    // Guard against cycles / runaway with a hard cap; stop at init.
    for _ in 0..64 {
        if pid <= 1 {
            break;
        }
        let Some((next, comm)) = crate::sys::ppid_comm(pid) else {
            break;
        };
        if let Some(canon) = canonical(&comm) {
            return Some(canon);
        }
        if next == pid {
            break;
        }
        pid = next;
    }
    None
}

/// Map a process `comm` (possibly truncated to 15 chars by the kernel) to a
/// canonical terminal display name, or None if it is not a known terminal.
fn canonical(comm: &str) -> Option<&'static str> {
    // (comm-as-seen, display-name). Ordered longest-first is not required since
    // we match exactly, with a prefix rule only for the 15-char truncation case.
    const TERMS: &[(&str, &str)] = &[
        ("kitty", "kitty"),
        ("alacritty", "alacritty"),
        ("foot", "foot"),
        ("wezterm-gui", "wezterm"),
        ("wezterm", "wezterm"),
        ("konsole", "konsole"),
        ("gnome-terminal-server", "gnome-terminal"),
        ("xterm", "xterm"),
        ("st", "st"),
        ("urxvt", "urxvt"),
        ("rxvt", "rxvt"),
        ("terminator", "terminator"),
        ("tilix", "tilix"),
        ("contour", "contour"),
        ("ghostty", "ghostty"),
        ("tmux", "tmux"),
        ("screen", "screen"),
        // macOS GUI terminals, as libproc reports their process names.
        ("Terminal", "Apple Terminal"),
        ("iTerm2", "iTerm2"),
        ("WarpTerminal", "Warp"),
    ];
    for (name, display) in TERMS {
        if comm == *name {
            return Some(display);
        }
        // Kernel truncates comm to 15 chars; match a long name by its prefix.
        if comm.len() == 15 && name.starts_with(comm) {
            return Some(display);
        }
    }
    None
}

/// Resolve the terminal from environment hints when the chain gave nothing.
fn from_env() -> Option<&'static str> {
    let has = |k: &str| std::env::var_os(k).is_some_and(|v| !v.is_empty());
    if has("KITTY_WINDOW_ID") {
        return Some("kitty");
    }
    if has("ALACRITTY_SOCKET") {
        return Some("alacritty");
    }
    if has("WEZTERM_EXECUTABLE") {
        return Some("wezterm");
    }
    if has("KONSOLE_VERSION") {
        return Some("konsole");
    }
    if has("TERMINATOR_UUID") {
        return Some("terminator");
    }
    if let Ok(tp) = std::env::var("TERM_PROGRAM") {
        if let Some(t) = from_token(&tp) {
            return Some(t);
        }
    }
    if let Ok(term) = std::env::var("TERM") {
        if let Some(t) = from_term(&term) {
            return Some(t);
        }
    }
    // $VTE_VERSION is set by every VTE-based terminal; gnome-terminal is the
    // most likely and matches the label fastfetch shows. Checked last.
    if has("VTE_VERSION") {
        return Some("gnome-terminal");
    }
    None
}

/// Map a $TERM value (e.g. "xterm-kitty", "foot-extra") to a terminal.
fn from_term(term: &str) -> Option<&'static str> {
    let t = term.to_ascii_lowercase();
    if t.contains("kitty") {
        Some("kitty")
    } else if t.contains("alacritty") {
        Some("alacritty")
    } else if t.contains("ghostty") {
        Some("ghostty")
    } else if t.contains("foot") {
        Some("foot")
    } else if t.contains("wezterm") {
        Some("wezterm")
    } else if t.contains("contour") {
        Some("contour")
    } else if t.starts_with("rxvt-unicode") {
        Some("urxvt")
    } else if t.starts_with("rxvt") {
        Some("rxvt")
    } else if t.starts_with("st-") || t == "st" {
        Some("st")
    } else if t.starts_with("tmux") {
        Some("tmux")
    } else if t.starts_with("screen") {
        Some("screen")
    } else {
        None
    }
}

/// Map a $TERM_PROGRAM token to a terminal.
fn from_token(tp: &str) -> Option<&'static str> {
    let t = tp.to_ascii_lowercase();
    match t.as_str() {
        "kitty" => Some("kitty"),
        "alacritty" => Some("alacritty"),
        "ghostty" => Some("ghostty"),
        "wezterm" => Some("wezterm"),
        "tmux" => Some("tmux"),
        "konsole" => Some("konsole"),
        // macOS conventions: Terminal.app and iTerm2 identify themselves only
        // through this variable.
        "apple_terminal" => Some("Apple Terminal"),
        "iterm.app" => Some("iTerm2"),
        "warpterminal" => Some("Warp"),
        _ if t.contains("foot") => Some("foot"),
        _ => None,
    }
}

/// Best-effort version for a small allowlist of terminals that answer
/// `--version` cheaply and safely. Returns the first version-like token.
fn version_of(term: &str) -> Option<String> {
    // Only probe binaries known to support --version without side effects.
    const SAFE: &[&str] = &[
        "kitty",
        "alacritty",
        "foot",
        "wezterm",
        "konsole",
        "ghostty",
        "contour",
        "tilix",
        "terminator",
        "gnome-terminal",
        "xterm",
        "tmux",
        "screen",
    ];
    if !SAFE.contains(&term) {
        return None;
    }
    let out = crate::util::cmd(term, &["--version"])?;
    out.lines().next().and_then(version_token)
}

/// $TERM_PROGRAM_VERSION, for terminals with no --version binary (Apple
/// Terminal, iTerm2, ...) — only when $TERM_PROGRAM names the same terminal
/// we detected, so a nested tmux doesn't inherit the outer app's version.
fn env_version(term: &str) -> Option<String> {
    let tp = std::env::var("TERM_PROGRAM").ok()?;
    if from_token(&tp)? != term {
        return None;
    }
    std::env::var("TERM_PROGRAM_VERSION")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Pick the first whitespace token that looks like a version number,
/// e.g. "kitty 0.41.1 created by ..." -> "0.41.1". A leading 'v' is stripped.
fn version_token(line: &str) -> Option<String> {
    for tok in line.split_whitespace() {
        let candidate = tok
            .strip_prefix('v')
            .or_else(|| tok.strip_prefix('V'))
            .unwrap_or(tok);
        if candidate.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return Some(candidate.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{canonical, from_term, version_token};

    #[test]
    fn canonical_exact_and_15char_truncation() {
        assert_eq!(canonical("kitty"), Some("kitty"));
        assert_eq!(canonical("foot"), Some("foot"));
        // The kernel truncates comm to 15 chars: "gnome-terminal-server" -> "gnome-terminal-".
        assert_eq!(canonical("gnome-terminal-"), Some("gnome-terminal"));
        assert_eq!(canonical("zsh"), None);
    }

    #[test]
    fn version_token_picks_first_numeric() {
        assert_eq!(
            version_token("kitty 0.41.1 created by ...").as_deref(),
            Some("0.41.1")
        );
        assert_eq!(version_token("v1.2.3").as_deref(), Some("1.2.3"));
        assert_eq!(version_token("no digits here"), None);
    }

    #[test]
    fn from_term_matches_term_value() {
        assert_eq!(from_term("xterm-kitty"), Some("kitty"));
        assert_eq!(from_term("foot-extra"), Some("foot"));
        assert_eq!(from_term("rxvt-unicode-256color"), Some("urxvt"));
        assert_eq!(from_term("dumb"), None);
    }
}
