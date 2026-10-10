use std::fs;
use std::path::PathBuf;

use alienfx::power;

fn fixture(status: &str, ac_online: bool) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "alienfx-test-{}-{}",
        std::process::id(),
        status.replace(' ', "_")
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("AC0")).unwrap();
    fs::write(dir.join("AC0").join("online"), if ac_online { "1" } else { "0" }).unwrap();
    fs::create_dir_all(dir.join("BAT0")).unwrap();
    fs::write(dir.join("BAT0").join("status"), format!("{status}\n")).unwrap();
    dir
}

#[test]
fn detects_charging() {
    let dir = fixture("Charging", true);
    assert_eq!(power::current_state_from(&dir), Some(6));
}

#[test]
fn detects_charged() {
    let dir = fixture("Full", true);
    assert_eq!(power::current_state_from(&dir), Some(5));
}

#[test]
fn detects_on_battery() {
    let dir = fixture("Discharging", false);
    assert_eq!(power::current_state_from(&dir), Some(8));
}

#[test]
fn falls_back_to_ac_when_status_unknown() {
    let dir = fixture("Unknown", true);
    assert_eq!(power::current_state_from(&dir), Some(5));
    let dir = fixture("Unknown", false);
    assert_eq!(power::current_state_from(&dir), Some(8));
}

#[test]
fn missing_dir_is_none() {
    assert_eq!(
        power::current_state_from(PathBuf::from("/nonexistent-alienfx-test").as_path()),
        None
    );
}
