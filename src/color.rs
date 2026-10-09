pub fn parse(s: &str) -> Result<(u8, u8, u8), String> {
    let s = s.trim().trim_start_matches('#');
    if let Some(c) = named(s) {
        return Ok(c);
    }
    match s.len() {
        3 => {
            let chars: Vec<char> = s.chars().collect();
            if !chars.iter().all(|c| c.is_ascii_hexdigit()) {
                return Err(format!("invalid color {s:?}"));
            }
            let expand = |c: char| u8::from_str_radix(&c.to_string(), 16).unwrap_or(0) * 17;
            Ok((expand(chars[0]), expand(chars[1]), expand(chars[2])))
        }
        6 => {
            let v = u32::from_str_radix(s, 16).map_err(|_| format!("invalid color {s:?}"))?;
            Ok(((v >> 16) as u8, (v >> 8) as u8, v as u8))
        }
        _ => Err(format!(
            "invalid color {s:?}; use #RRGGBB or a name like red/blue/white"
        )),
    }
}

fn named(s: &str) -> Option<(u8, u8, u8)> {
    let c = match s.to_ascii_lowercase().as_str() {
        "black" => (0x00, 0x00, 0x00),
        "white" => (0xFF, 0xFF, 0xFF),
        "red" => (0xFF, 0x00, 0x00),
        "green" => (0x00, 0xFF, 0x00),
        "blue" => (0x00, 0x00, 0xFF),
        "yellow" => (0xFF, 0xFF, 0x00),
        "cyan" => (0x00, 0xFF, 0xFF),
        "magenta" => (0xFF, 0x00, 0xFF),
        "orange" => (0xFF, 0x80, 0x00),
        "purple" => (0x80, 0x00, 0x80),
        _ => return None,
    };
    Some(c)
}
