//! Host: machine model. On Lenovo the marketing name lives in `product_version`
//! ("ThinkPad P53") and the model code in `product_name` ("20QQS0JD01"), so we
//! show "ThinkPad P53 (20QQS0JD01)". On a Mac: "Mac mini (Mac14,3)".
use crate::detect::{Row, Rows};
#[cfg(target_os = "linux")]
use crate::util::read_trim;

#[cfg(target_os = "linux")]
pub fn detect() -> Rows {
    let base = "/sys/devices/virtual/dmi/id";
    let product_name = read_trim(&format!("{base}/product_name"));
    let product_version = read_trim(&format!("{base}/product_version"));
    let board_name = read_trim(&format!("{base}/board_name"));

    let (primary, secondary) = match (&product_version, &product_name) {
        (Some(pv), pn) if looks_like_name(pv) => (pv.clone(), pn.clone()),
        (_, Some(pn)) if looks_like_name(pn) => (pn.clone(), product_version.clone()),
        (Some(pv), pn) => (pv.clone(), pn.clone()),
        (_, Some(pn)) => (pn.clone(), None),
        _ => (board_name.unwrap_or_default(), None),
    };

    if primary.is_empty() {
        return Vec::new();
    }

    let value = match secondary {
        Some(s) if !s.is_empty() && s != primary => format!("{primary} ({s})"),
        _ => primary,
    };
    vec![Row::val(value)]
}

/// A human-readable model name has letters and a space, unlike a bare serial.
#[cfg(target_os = "linux")]
fn looks_like_name(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_alphabetic()) && s.contains(' ')
}

/// macOS: the marketing name ("Mac mini") lives in the device tree as
/// `product-name` on Apple Silicon — `ioreg` is the only zero-dependency way
/// at it. The board id (hw.model, "Mac14,3") is the always-available
/// fallback; no static model table to keep up to date.
#[cfg(target_os = "macos")]
pub fn detect() -> Rows {
    let model = crate::sys::sysctl_string("hw.model");
    let marketing = crate::util::cmd("ioreg", &["-rd1", "-c", "IOPlatformExpertDevice"])
        .and_then(|dump| crate::util::ioreg_string(&dump, "product-name"));
    match (marketing, model) {
        (Some(name), Some(id)) if name != id => vec![Row::val(format!("{name} ({id})"))],
        (Some(name), _) => vec![Row::val(name)],
        (None, Some(id)) => vec![Row::val(id)],
        (None, None) => Vec::new(),
    }
}
