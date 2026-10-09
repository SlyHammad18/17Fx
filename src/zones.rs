pub struct Zone {
    pub name: &'static str,
    pub mask: u32,
    pub desc: &'static str,
}

pub const ZONES: &[Zone] = &[
    Zone { name: "keyboard-right", mask: 0x0001, desc: "Keyboard right block" },
    Zone { name: "keyboard-mright", mask: 0x0002, desc: "Keyboard middle-right block" },
    Zone { name: "keyboard-mleft", mask: 0x0004, desc: "Keyboard middle-left block" },
    Zone { name: "keyboard-left", mask: 0x0008, desc: "Keyboard left block" },
    Zone { name: "alien-head", mask: 0x0020, desc: "Alien head on the lid (outside)" },
    Zone { name: "logo", mask: 0x0040, desc: "Alienware logo below the screen" },
    Zone { name: "touchpad", mask: 0x0080, desc: "Touchpad" },
    Zone { name: "power-button", mask: 0x0100, desc: "Power button" },
    Zone { name: "speaker-left", mask: 0x0400, desc: "Bottom front edge, left light bar" },
    Zone { name: "speaker-right", mask: 0x0800, desc: "Bottom front edge, right light bar" },
    Zone { name: "display-left", mask: 0x1000, desc: "Display top edge, left light bar" },
    Zone { name: "display-right", mask: 0x2000, desc: "Display top edge, right light bar" },
    Zone { name: "media-bar", mask: 0x4000, desc: "Macro/media key bar" },
];

pub const KEYBOARD_MASK: u32 = 0x000F;
pub const ALL_ZONES_MASK: u32 = 0x7DEF;

pub fn lookup(name: &str) -> Option<u32> {
    match name.trim().to_ascii_lowercase().as_str() {
        "all" => Some(ALL_ZONES_MASK),
        "keyboard" => Some(KEYBOARD_MASK),
        n => ZONES.iter().find(|z| z.name == n).map(|z| z.mask),
    }
}

pub fn parse(spec: &str) -> Result<u32, String> {
    let mut mask = 0u32;
    for part in spec.split([',', '+']) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        mask |= lookup(part)
            .ok_or_else(|| format!("unknown zone {part:?}; run `alienfx zones` for the list"))?;
    }
    if mask == 0 {
        return Err("no zone specified".into());
    }
    Ok(mask)
}
