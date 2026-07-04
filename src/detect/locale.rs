//! Locale: from the environment (LC_ALL / LC_CTYPE / LANG).
use crate::detect::{Row, Rows};

pub fn detect() -> Rows {
    for var in ["LC_ALL", "LC_CTYPE", "LANG"] {
        if let Ok(v) = std::env::var(var) {
            if !v.is_empty() && v != "C" && v != "POSIX" {
                return vec![Row::val(v)];
            }
        }
    }
    match std::env::var("LANG") {
        Ok(v) if !v.is_empty() => vec![Row::val(v)],
        _ => Vec::new(),
    }
}
