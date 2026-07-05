//! GPU: graphics adapter model, e.g. "Quadro RTX 3000".
//! Primary source: /proc/driver/nvidia/gpus/*/information ("Model:" line).
//! lspci covers non-NVIDIA adapters: as the sole source when the nvidia proc
//! is absent, and additionally for the other card on hybrid (iGPU + NVIDIA)
//! systems — detected first via /sys so the common single-GPU run stays
//! subprocess-free.
use crate::detect::{Row, Rows};

pub fn detect() -> Rows {
    let mut rows = nvidia_gpus();
    if rows.is_empty() {
        return lspci_gpus(false);
    }
    if has_non_nvidia_card() {
        rows.extend(lspci_gpus(true));
    }
    rows
}

/// Whether /sys/class/drm has a card from a non-NVIDIA vendor (PCI vendor id
/// != 0x10de), i.e. an adapter the NVIDIA proc interface cannot report.
fn has_non_nvidia_card() -> bool {
    let Ok(entries) = std::fs::read_dir("/sys/class/drm") else {
        return false;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(n) = name.to_str() else { continue };
        // Whole-card nodes only ("card0", ...), not per-connector ones.
        if !n.starts_with("card") || n.contains('-') {
            continue;
        }
        if let Some(v) = crate::util::read_trim(&format!("/sys/class/drm/{n}/device/vendor")) {
            if !v.eq_ignore_ascii_case("0x10de") {
                return true;
            }
        }
    }
    false
}

/// One Row per NVIDIA GPU, parsed from the driver's proc information file.
fn nvidia_gpus() -> Rows {
    let mut rows = Vec::new();
    let Ok(entries) = std::fs::read_dir("/proc/driver/nvidia/gpus") else {
        return rows;
    };
    for entry in entries.flatten() {
        let info = entry.path().join("information");
        let Some(path) = info.to_str() else { continue };
        let Some(text) = crate::util::read_trim(path) else {
            continue;
        };
        for line in text.lines() {
            // The "Model:" value is tab-separated after the colon.
            if let Some((k, v)) = line.split_once(':') {
                if k.trim() == "Model" {
                    let model = v.trim();
                    if !model.is_empty() {
                        rows.push(Row::val(model));
                    }
                    break;
                }
            }
        }
    }
    rows
}

/// Parse `lspci` for graphics controllers; with `skip_nvidia`, NVIDIA cards
/// are left out (already reported from the driver's proc interface).
fn lspci_gpus(skip_nvidia: bool) -> Rows {
    let Some(out) = crate::util::cmd("lspci", &[]) else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    for line in out.lines() {
        if !(line.contains("VGA compatible controller")
            || line.contains("3D controller")
            || line.contains("Display controller"))
        {
            continue;
        }
        // Text after the last ": " is the vendor + device description.
        let Some((_, desc)) = line.rsplit_once(": ") else {
            continue;
        };
        if skip_nvidia && desc.contains("NVIDIA") {
            continue;
        }
        let name = clean_lspci(desc.trim());
        if !name.is_empty() {
            rows.push(Row::val(name));
        }
    }
    rows
}

/// Strip a known vendor prefix and prefer a bracketed marketing name.
fn clean_lspci(desc: &str) -> String {
    let stripped = desc
        .strip_prefix("NVIDIA Corporation ")
        .or_else(|| desc.strip_prefix("Intel Corporation "))
        .or_else(|| desc.strip_prefix("Advanced Micro Devices, Inc. "))
        .unwrap_or(desc);
    // AMD prefixes the die with a bracketed vendor tag ("[AMD/ATI] Navi 31
    // [Radeon ...]"); drop it so it isn't mistaken for the marketing name.
    let stripped = stripped
        .strip_prefix("[AMD/ATI] ")
        .or_else(|| stripped.strip_prefix("[AMD] "))
        .unwrap_or(stripped);
    // Drop a trailing PCI revision, e.g. "Raphael (rev c9)" -> "Raphael".
    let stripped = stripped
        .rsplit_once(" (rev ")
        .map_or(stripped, |(head, _)| head);
    // Prefer the marketing name in "[ ... ]" if present.
    if let Some(start) = stripped.find('[') {
        if let Some(end) = stripped[start + 1..].find(']') {
            let inner = stripped[start + 1..start + 1 + end].trim();
            if !inner.is_empty() {
                return inner.to_string();
            }
        }
    }
    stripped.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::clean_lspci;

    #[test]
    fn amd_prefers_marketing_name_over_vendor_tag() {
        assert_eq!(
            clean_lspci(
                "Advanced Micro Devices, Inc. [AMD/ATI] Navi 31 [Radeon RX 7900 XTX] (rev c8)"
            ),
            "Radeon RX 7900 XTX"
        );
        assert_eq!(
            clean_lspci("Advanced Micro Devices, Inc. [AMD/ATI] Rembrandt [Radeon 680M]"),
            "Radeon 680M"
        );
    }

    #[test]
    fn amd_apu_without_marketing_bracket() {
        assert_eq!(
            clean_lspci("Advanced Micro Devices, Inc. [AMD/ATI] Raphael (rev c9)"),
            "Raphael"
        );
    }

    #[test]
    fn nvidia_and_intel_unaffected() {
        assert_eq!(
            clean_lspci("NVIDIA Corporation TU106 [GeForce RTX 2060]"),
            "GeForce RTX 2060"
        );
        assert_eq!(
            clean_lspci("Intel Corporation CoffeeLake-H GT2 [UHD Graphics 630]"),
            "UHD Graphics 630"
        );
        assert_eq!(
            clean_lspci("Intel Corporation UHD Graphics 630"),
            "UHD Graphics 630"
        );
    }
}
