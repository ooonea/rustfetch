//! ANSI color palette. Colors are SGR parameter strings (the part between
//! `ESC[` and `m`). The default palette matches the user's Dracula-ish
//! fastfetch theme on unicorn.

pub struct Palette {
    pub enabled: bool,
    /// Info key labels (left column).
    pub key: &'static str,
    /// The `user@host` title line.
    pub title: &'static str,
    /// The separator rule lines.
    pub sep: &'static str,
}

impl Palette {
    pub fn new(enabled: bool) -> Self {
        Palette {
            enabled,
            key: "38;2;149;128;255",
            title: "1;38;2;149;128;255",
            sep: "38;2;121;112;169",
        }
    }

    /// Wrap `text` in the given SGR parameters, or return it unchanged when
    /// color is disabled or the parameter string is empty.
    pub fn paint(&self, sgr: &str, text: &str) -> String {
        if self.enabled && !sgr.is_empty() {
            format!("\x1b[{sgr}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
}
