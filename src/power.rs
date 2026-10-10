use std::path::Path;

use crate::protocol;

pub fn current_state() -> Option<u8> {
    current_state_from(Path::new("/sys/class/power_supply"))
}

pub fn current_state_from(base: &Path) -> Option<u8> {
    let mut ac_online = false;
    let mut status = String::new();
    for entry in std::fs::read_dir(base).ok()? {
        let Ok(entry) = entry else { continue };
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('A') {
            if std::fs::read_to_string(entry.path().join("online"))
                .map(|s| s.trim() == "1")
                .unwrap_or(false)
            {
                ac_online = true;
            }
        } else if name.starts_with('B') {
            if let Ok(s) = std::fs::read_to_string(entry.path().join("status")) {
                status = s.trim().to_string();
            }
        }
    }
    Some(match status.as_str() {
        "Charging" => state_code("ac-charging"),
        "Full" | "Not charging" => state_code("ac-charged"),
        "Discharging" => state_code("bat-on"),
        _ if ac_online => state_code("ac-charged"),
        _ => state_code("bat-on"),
    })
}

fn state_code(name: &str) -> u8 {
    protocol::state_code(name).unwrap_or(0)
}
