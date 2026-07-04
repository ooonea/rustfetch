//! Distro ASCII logos. Each logo is a slice of lines plus a single SGR color;
//! `render` pads the block to a uniform width. Selection is by `/etc/os-release`
//! `ID` when the selector is "auto".

pub struct Logo {
    pub lines: &'static [&'static str],
    pub sgr: &'static str,
}

/// Resolve a logo selector ("auto", "debian", "tux", "none", ...) to a logo.
/// A known name wins; an unknown *explicit* name falls back to the detected
/// distro (matching fastfetch), and finally to the generic Tux logo.
pub fn get(selector: &str) -> Option<Logo> {
    let sel = selector.to_ascii_lowercase();
    if sel == "none" || sel == "off" {
        return None;
    }
    let name = if sel == "auto" { detect_distro() } else { sel };
    Some(
        by_name(&name)
            .or_else(|| by_name(&detect_distro()))
            .unwrap_or(Logo {
                lines: TUX,
                sgr: TUX_SGR,
            }),
    )
}

/// A logo for a known distro/name, or None if unrecognized.
fn by_name(name: &str) -> Option<Logo> {
    match name {
        "debian" => Some(Logo {
            lines: DEBIAN,
            sgr: "38;2;215;7;81",
        }),
        "tux" | "linux" | "generic" => Some(Logo {
            lines: TUX,
            sgr: TUX_SGR,
        }),
        _ => None,
    }
}

const TUX_SGR: &str = "38;2;236;236;236";

fn detect_distro() -> String {
    if let Ok(s) = std::fs::read_to_string("/etc/os-release") {
        for line in s.lines() {
            if let Some(v) = line.strip_prefix("ID=") {
                return v.trim().trim_matches('"').to_ascii_lowercase();
            }
        }
    }
    "tux".to_string()
}

const DEBIAN: &[&str] = &[
    r#"       _,met$$$$$gg."#,
    r#"    ,g$$$$$$$$$$$$$$$P."#,
    r#"  ,g$$P"     """Y$$."."#,
    r#" ,$$P'              `$$$."#,
    r#"',$$P       ,ggs.     `$$b:"#,
    r#"`d$$'     ,$P"'   .    $$$"#,
    r#" $$P      d$'     ,    $$P"#,
    r#" $$:      $$.   -    ,d$$'"#,
    r#" $$;      Y$b._   _,d$P'"#,
    r#" Y$$.    `.`"Y$$$$P"'"#,
    r#" `$$b      "-.__"#,
    r#"  `Y$$"#,
    r#"   `Y$$."#,
    r#"     `$$b."#,
    r#"       `Y$$b."#,
    r#"          `"Y$b._"#,
    r#"              `""""#,
];

const TUX: &[&str] = &[
    r#"        a8888b."#,
    r#"       d888888b."#,
    r#"       8P"YP"Y88"#,
    r#"       8|o||o|88"#,
    r#"       8'    .88"#,
    r#"       8`._.' Y8."#,
    r#"      d/      `8b."#,
    r#"     dP   .    Y8b."#,
    r#"    d8:'  "  `::88b"#,
    r#"   d8"         'Y88b"#,
    r#"  :8P    '      :888"#,
    r#"   8a.  :     _a88P"#,
    r#" ._/"Yaa_:   .| 88P|"#,
    r#" \    YP"    `| 8P  `."#,
    r#" /     \.___.d|    .'"#,
    r#" `--..__)8888P`._.'"#,
];
