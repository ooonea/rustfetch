//! Detection framework.
//!
//! CONTRACT for every module `detect/<name>.rs`:
//!
//! ```ignore
//! pub fn detect() -> crate::detect::Rows { ... }
//! ```
//!
//! * Return one `Row` per output line. Most modules return exactly one.
//! * Return an empty `Vec` when the info is unavailable — the line is skipped
//!   entirely (e.g. `battery` on a desktop, `swap` with no swap configured).
//! * `Row::val(v)` uses the default label supplied by `main`. `Row::keyed(k, v)`
//!   overrides the label (used when a module emits several distinct rows).
//! * Never panic and never block: read `/proc` and `/sys`, or fall back to
//!   `crate::util::cmd(...)` only where no file source exists.

pub struct Row {
    pub key: Option<String>,
    pub value: String,
}

impl Row {
    /// A row using the module's default label.
    pub fn val(value: impl Into<String>) -> Self {
        Row {
            key: None,
            value: value.into(),
        }
    }

    /// A row with an explicit label override.
    #[allow(dead_code)]
    pub fn keyed(key: impl Into<String>, value: impl Into<String>) -> Self {
        Row {
            key: Some(key.into()),
            value: value.into(),
        }
    }
}

pub type Rows = Vec<Row>;

pub mod battery;
pub mod cpu;
pub mod de;
pub mod disk;
pub mod display;
pub mod gpu;
pub mod host;
pub mod kernel;
pub mod locale;
pub mod memory;
pub mod os;
pub mod packages;
pub mod shell;
pub mod swap;
pub mod terminal;
pub mod uptime;
pub mod wm;
