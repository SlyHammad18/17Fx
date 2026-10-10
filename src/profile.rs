use std::fs;
use std::path::{Path, PathBuf};

pub type Entry = (u32, (u8, u8, u8));

pub fn base_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".config").join("alienfx")
}

pub fn state_path() -> PathBuf {
    base_dir().join("state")
}

pub fn profiles_dir() -> PathBuf {
    base_dir().join("profiles")
}

pub fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Err(format!(
            "invalid profile name {name:?}; use letters, digits, '-', '_', '.'"
        ));
    }
    Ok(())
}

pub fn parse(text: &str) -> Vec<Entry> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let (Some(mask_s), Some(color_s)) = (parts.next(), parts.next()) else {
            continue;
        };
        let Ok(mask) = u32::from_str_radix(mask_s.trim_start_matches("0x"), 16) else {
            continue;
        };
        let Ok(v) = u32::from_str_radix(color_s.trim_start_matches('#'), 16) else {
            continue;
        };
        out.push((mask, ((v >> 16) as u8, (v >> 8) as u8, v as u8)));
    }
    out
}

pub fn render(entries: &[Entry]) -> String {
    let mut s = String::from("# alienfx configuration (mask hex, RRGGBB hex)\n");
    for (mask, (r, g, b)) in entries {
        s.push_str(&format!("{mask:04X} {r:02X}{g:02X}{b:02X}\n"));
    }
    s
}

pub fn load(path: &Path) -> std::io::Result<Vec<Entry>> {
    let text = fs::read_to_string(path)?;
    Ok(parse(&text))
}

pub fn write(path: &Path, entries: &[Entry]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, render(entries))
}

pub fn merge(entries: &[Entry], mask: u32, rgb: (u8, u8, u8)) -> Vec<Entry> {
    let mut out: Vec<Entry> = entries
        .iter()
        .filter_map(|(m, c)| {
            let m = m & !mask;
            (m != 0).then_some((m, *c))
        })
        .collect();
    out.push((mask, rgb));
    out
}

pub fn record(mask: u32, rgb: (u8, u8, u8)) -> std::io::Result<()> {
    let path = state_path();
    let existing = load(&path).unwrap_or_default();
    write(&path, &merge(&existing, mask, rgb))
}

pub fn list_profiles() -> std::io::Result<Vec<String>> {
    let dir = profiles_dir();
    let mut names = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.path().is_file() {
            if let Some(name) = entry.file_name().to_str() {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}
