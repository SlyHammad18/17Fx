use std::ffi::c_int;
use std::path::{Path, PathBuf};
use std::process::exit;

use clap::{Parser, Subcommand};

use alienfx::{apply, color, device, power, profile, protocol, zones};

const SIGPIPE: c_int = 13;
const SIG_DFL: usize = 0;

extern "C" {
    fn signal(signum: c_int, handler: usize) -> usize;
}

#[derive(Parser)]
#[command(
    name = "alienfx",
    version,
    about = "Alienware AlienFX lighting control (15 R3 / 17 R4, USB 187c:0530)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Args)]
struct Common {
    #[arg(long, value_name = "PATH", help = "Use a specific hidraw device instead of auto-detecting")]
    device: Option<PathBuf>,
    #[arg(short, long, help = "Print packets as they are sent")]
    verbose: bool,
}

#[derive(Subcommand)]
enum Command {
    List,
    Zones,
    Probe {
        #[command(flatten)]
        common: Common,
    },
    Set {
        zone: String,
        color: String,
        #[arg(long = "no-save", help = "Skip writing power-state slots (live apply only)")]
        no_save: bool,
        #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100), help = "Scale the color to 0-100% brightness")]
        brightness: Option<u8>,
        #[arg(long)]
        dry_run: bool,
        #[command(flatten)]
        common: Common,
    },
    SetAll {
        color: String,
        #[arg(long = "no-save")]
        no_save: bool,
        #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
        brightness: Option<u8>,
        #[arg(long)]
        dry_run: bool,
        #[command(flatten)]
        common: Common,
    },
    Morph {
        zone: String,
        color1: String,
        color2: String,
        #[arg(long, value_parser = clap::value_parser!(u16).range(30..=942), help = "Morph tempo, lower = faster (default 75)")]
        speed: Option<u16>,
        #[arg(long = "no-save")]
        no_save: bool,
        #[arg(long)]
        dry_run: bool,
        #[command(flatten)]
        common: Common,
    },
    Blink {
        zone: String,
        color: String,
        #[arg(long, value_parser = clap::value_parser!(u16).range(30..=942), help = "Blink tempo, lower = faster (default 75)")]
        speed: Option<u16>,
        #[arg(long = "no-save")]
        no_save: bool,
        #[arg(long)]
        dry_run: bool,
        #[command(flatten)]
        common: Common,
    },
    Off {
        #[command(flatten)]
        common: Common,
    },
    On {
        #[command(flatten)]
        common: Common,
    },
    Undim {
        #[command(flatten)]
        common: Common,
    },
    Save {
        #[arg(default_value = "default", help = "Profile name")]
        name: String,
    },
    Apply {
        #[arg(default_value = "default", help = "Profile name")]
        name: String,
        #[command(flatten)]
        common: Common,
    },
    Profiles,
    Power {
        #[arg(long, value_name = "COLOR", help = "Color while charging (AC plugged in)")]
        charging: Option<String>,
        #[arg(long, value_name = "COLOR", help = "Color when charged (AC, battery full)")]
        charged: Option<String>,
        #[arg(long, value_name = "COLOR", help = "Color while on battery")]
        battery: Option<String>,
        #[arg(long, value_name = "COLOR", help = "Color when the battery is critical (firmware blinks it itself)")]
        low: Option<String>,
        #[arg(long, default_value = "power-button")]
        zone: String,
        #[arg(long)]
        dry_run: bool,
        #[command(flatten)]
        common: Common,
    },
    Dim {
        #[arg(long, help = "Dim only while on battery power")]
        battery_only: bool,
        #[command(flatten)]
        common: Common,
    },
    Send {
        hex: String,
        #[arg(long, value_parser = ["output", "feature"], default_value = "output")]
        mode: String,
        #[arg(long, help = "Milliseconds to listen for device responses after sending")]
        read_ms: Option<u64>,
        #[arg(long)]
        dry_run: bool,
        #[command(flatten)]
        common: Common,
    },
}

const UDEV_HINT: &str = "Fix with:\n  sudo cp udev/99-alienfx.rules /etc/udev/rules.d/\n  sudo udevadm control --reload-rules && sudo udevadm trigger\nThen re-run this command.";

fn main() {
    unsafe { signal(SIGPIPE, SIG_DFL) };
    let cli = Cli::parse();
    if let Err(e) = run(cli.command) {
        eprintln!("error: {e}");
        exit(1);
    }
}

fn run(cmd: Command) -> Result<(), String> {
    match cmd {
        Command::List => cmd_list(),
        Command::Zones => cmd_zones(),
        Command::Probe { common } => cmd_probe(&common),
        Command::Set {
            zone,
            color,
            no_save,
            brightness,
            dry_run,
            common,
        } => cmd_set(&zone, &color, !no_save, brightness, dry_run, &common),
        Command::SetAll {
            color,
            no_save,
            brightness,
            dry_run,
            common,
        } => cmd_set("all", &color, !no_save, brightness, dry_run, &common),
        Command::Morph {
            zone,
            color1,
            color2,
            speed,
            no_save,
            dry_run,
            common,
        } => cmd_effect(
            &zone,
            &apply::Action::Morph(color::parse(&color1)?, color::parse(&color2)?),
            speed,
            !no_save,
            dry_run,
            &common,
            "morph",
        ),
        Command::Blink {
            zone,
            color,
            speed,
            no_save,
            dry_run,
            common,
        } => cmd_effect(
            &zone,
            &apply::Action::Blink(color::parse(&color)?),
            speed,
            !no_save,
            dry_run,
            &common,
            "blink",
        ),
        Command::Off { common } => {
            let mut dev = open_device(&common)?;
            dev.send_all(
                &apply::reset_packets(protocol::RESET_ALL_LIGHTS_OFF),
                false,
                common.verbose,
            )
            .map_err(|e| e.to_string())?;
            println!("all lights off");
            Ok(())
        }
        Command::On { common } => {
            let mut dev = open_device(&common)?;
            dev.send_all(
                &apply::reset_packets(protocol::RESET_ALL_LIGHTS_ON),
                false,
                common.verbose,
            )
            .map_err(|e| e.to_string())?;
            println!("all lights on");
            Ok(())
        }
        Command::Undim { common } => {
            let mut dev = open_device(&common)?;
            dev.send_all(
                &[
                    protocol::pkt_dim(protocol::DIM_DISABLE, protocol::DIM_MODE_ALWAYS),
                    protocol::pkt_transmit_execute(),
                ],
                false,
                common.verbose,
            )
            .map_err(|e| e.to_string())?;
            println!("dim disabled");
            Ok(())
        }
        Command::Save { name } => cmd_save(&name),
        Command::Apply { name, common } => cmd_apply(&name, &common),
        Command::Profiles => cmd_profiles(),
        Command::Power {
            charging,
            charged,
            battery,
            low,
            zone,
            dry_run,
            common,
        } => cmd_power(
            charging.as_deref(),
            charged.as_deref(),
            battery.as_deref(),
            low.as_deref(),
            &zone,
            dry_run,
            &common,
        ),
        Command::Dim {
            battery_only,
            common,
        } => {
            let mut dev = open_device(&common)?;
            let mode = if battery_only {
                protocol::DIM_MODE_BATTERY_ONLY
            } else {
                protocol::DIM_MODE_ALWAYS
            };
            dev.send_all(
                &[
                    protocol::pkt_dim(protocol::DIM_ENABLE, mode),
                    protocol::pkt_transmit_execute(),
                ],
                false,
                common.verbose,
            )
            .map_err(|e| e.to_string())?;
            println!(
                "dim enabled{}",
                if battery_only { " (battery only)" } else { "" }
            );
            Ok(())
        }
        Command::Send {
            hex,
            mode,
            read_ms,
            dry_run,
            common,
        } => cmd_send(&hex, &mode, read_ms, dry_run, &common),
    }
}

fn cmd_list() -> Result<(), String> {
    let devs = device::scan().map_err(|e| format!("scanning hidraw devices: {e}"))?;
    if devs.is_empty() {
        println!("no AlienFX controller (USB 187c:0530) found");
        return Ok(());
    }
    for d in &devs {
        let accessible = device::Hidraw::open(&d.node).is_ok();
        let name = if d.name.is_empty() { "-" } else { d.name.as_str() };
        let access = if accessible { "writable" } else { "no access" };
        println!("{}  {name}  {}  {access}", d.node.display(), d.id());
    }
    Ok(())
}

fn cmd_zones() -> Result<(), String> {
    println!("{:<8} {:<16} region", "mask", "name");
    for z in zones::ZONES {
        println!("0x{:04X}  {:<16} {}", z.mask, z.name, z.desc);
    }
    println!();
    println!(
        "aliases: keyboard (0x{:04X}), all (0x{:04X})",
        zones::KEYBOARD_MASK, zones::ALL_ZONES_MASK
    );
    println!("combine zones with commas, e.g.: alienfx set logo,touchpad red");
    Ok(())
}

fn cmd_probe(common: &Common) -> Result<(), String> {
    let dev = open_device(common)?;
    let detected = device::scan()
        .ok()
        .and_then(|devs| devs.into_iter().find(|d| d.node == dev.path));
    if let Some(d) = detected {
        println!("node:     {}", d.node.display());
        println!("hid id:   {}", d.id());
        if !d.name.is_empty() {
            println!("hid name: {}", d.name);
        }
        for (label, key) in [
            ("product", "product"),
            ("manufacturer", "manufacturer"),
            ("serial", "serial"),
        ] {
            if let Some(v) = device::usb_string(&d.sysfs, key) {
                println!("{label:<9} {v}");
            }
        }
    }
    Ok(())
}

fn send_preamble(dev: &mut device::Hidraw, common: &Common) -> Result<(), String> {
    dev.send_all(&apply::apply_preamble(), false, common.verbose)
        .map_err(|e| e.to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(500));
    Ok(())
}

fn cmd_set(
    zone: &str,
    color: &str,
    save: bool,
    brightness: Option<u8>,
    dry_run: bool,
    common: &Common,
) -> Result<(), String> {
    let mask = zones::parse(zone)?;
    let rgb = color::parse(color)?;
    let rgb = match brightness {
        Some(b) => color::scale(rgb, b),
        None => rgb,
    };
    let packets = apply::set_color_packets(mask, rgb, zones::ALL_ZONES_MASK, save);
    if dry_run {
        println!("zone:   {zone} (mask 0x{mask:04X})");
        println!("color:  #{:02X}{:02X}{:02X}", rgb.0, rgb.1, rgb.2);
        if let Some(b) = brightness {
            println!("bright: {b}%");
        }
        println!("slots:  {}", if save { "write" } else { "skip (--no-save)" });
        println!("packets: {}", packets.len());
        for p in &packets {
            println!("{:02x?}", p);
        }
        return Ok(());
    }
    let mut dev = open_device(common)?;
    send_preamble(&mut dev, common)?;
    dev.send_all(&packets, false, common.verbose)
        .map_err(|e| e.to_string())?;
    if let Err(e) = profile::record(mask, rgb) {
        eprintln!("warning: could not update state file: {e}");
    }
    println!(
        "{} -> #{:02X}{:02X}{:02X}{}",
        zone,
        rgb.0,
        rgb.1,
        rgb.2,
        if save {
            " (power-state slots updated)"
        } else {
            " (live only)"
        }
    );
    Ok(())
}

fn cmd_send(
    hex: &str,
    mode: &str,
    read_ms: Option<u64>,
    dry_run: bool,
    common: &Common,
) -> Result<(), String> {
    let bytes = parse_hex(hex)?;
    if bytes.is_empty() {
        return Err("empty hex string".into());
    }
    if bytes.len() > 64 {
        return Err(format!("max 64 bytes, got {}", bytes.len()));
    }
    if bytes[0] != protocol::REPORT_ID {
        eprintln!(
            "warning: first byte 0x{:02X} is not the AlienFX report id 0x{:02X}",
            bytes[0],
            protocol::REPORT_ID
        );
    }
    let mut pkt = [0u8; 64];
    pkt[..bytes.len()].copy_from_slice(&bytes);
    let n = bytes.len().max(protocol::PACKET_LEN);
    if dry_run {
        println!("mode:   {mode}");
        println!("length: {n}");
        println!("packet: {:02x?}", &pkt[..n]);
        return Ok(());
    }
    let mut dev = open_device(common)?;
    if mode == "feature" {
        dev.write_feature(&pkt[..n]).map_err(|e| e.to_string())?;
    } else {
        dev.write_packet(&pkt[..n]).map_err(|e| e.to_string())?;
    }
    if common.verbose {
        eprintln!("TX {:02x?}", &pkt[..n]);
    }
    println!("sent {n} bytes ({mode})");
    if let Some(ms) = read_ms {
        let frames = dev.read_for(ms).map_err(|e| e.to_string())?;
        if frames.is_empty() {
            println!("no device response within {ms} ms");
        } else {
            for f in frames {
                println!("RX {:02x?}", f);
            }
        }
    }
    Ok(())
}

fn cmd_save(name: &str) -> Result<(), String> {
    profile::validate_name(name)?;
    let entries = profile::load(&profile::state_path())
        .map_err(|_| "no tracked configuration yet; set some colors first".to_string())?;
    if entries.is_empty() {
        return Err("no tracked configuration yet; set some colors first".into());
    }
    let path = profile::profiles_dir().join(name);
    profile::write(&path, &entries).map_err(|e| e.to_string())?;
    println!(
        "{name}: {} zone group(s) saved to {}",
        entries.len(),
        path.display()
    );
    Ok(())
}

fn cmd_apply(name: &str, common: &Common) -> Result<(), String> {
    profile::validate_name(name)?;
    let entries = profile::load(&profile::profiles_dir().join(name))
        .map_err(|_| format!("profile {name:?} not found; run `alienfx profiles`"))?;
    if entries.is_empty() {
        return Err(format!("profile {name:?} is empty"));
    }
    let mut dev = open_device(common)?;
    send_preamble(&mut dev, common)?;
    for (mask, rgb) in &entries {
        dev.send_all(
            &apply::set_color_packets(*mask, *rgb, zones::ALL_ZONES_MASK, true),
            false,
            common.verbose,
        )
        .map_err(|e| e.to_string())?;
        println!("{mask:04X} -> #{:02X}{:02X}{:02X}", rgb.0, rgb.1, rgb.2);
    }
    println!("{name}: applied {} group(s)", entries.len());
    Ok(())
}

fn cmd_profiles() -> Result<(), String> {
    let names = profile::list_profiles().map_err(|e| e.to_string())?;
    if names.is_empty() {
        println!("no saved profiles");
        return Ok(());
    }
    for n in names {
        let count = profile::load(&profile::profiles_dir().join(&n))
            .map(|e| e.len())
            .unwrap_or(0);
        println!("{n} ({count} group(s))");
    }
    Ok(())
}

fn cmd_effect(
    zone: &str,
    action: &apply::Action,
    speed: Option<u16>,
    save: bool,
    dry_run: bool,
    common: &Common,
    label: &str,
) -> Result<(), String> {
    let mask = zones::parse(zone)?;
    let speed = speed.unwrap_or(protocol::DEFAULT_SPEED);
    let packets = apply::set_action_packets(mask, *action, zones::ALL_ZONES_MASK, save, speed);
    if dry_run {
        println!("zone:   {zone} (mask 0x{mask:04X})");
        println!("{label}:  {}", describe(action));
        println!("speed:  {speed}");
        println!("slots:  {}", if save { "write" } else { "skip (--no-save)" });
        println!("packets: {}", packets.len());
        for p in &packets {
            println!("{:02x?}", p);
        }
        return Ok(());
    }
    let mut dev = open_device(common)?;
    send_preamble(&mut dev, common)?;
    dev.send_all(&packets, false, common.verbose)
        .map_err(|e| e.to_string())?;
    println!(
        "{} -> {} (speed {speed}){}",
        zone,
        describe(action),
        if save { " (slots updated)" } else { " (live only)" }
    );
    Ok(())
}

fn parse_action(s: &str) -> Result<apply::Action, String> {
    let parts: Vec<&str> = s.split(',').map(str::trim).collect();
    match parts.as_slice() {
        [one] => Ok(apply::Action::Static(color::parse(one)?)),
        [a, b] => Ok(apply::Action::Morph(
            color::parse(a)?,
            color::parse(b)?,
        )),
        _ => Err(format!(
            "invalid color spec {s:?}; use COLOR, or COLOR1,COLOR2 for a morph"
        )),
    }
}

fn describe(action: &apply::Action) -> String {
    let hex = |c: (u8, u8, u8)| format!("#{:02X}{:02X}{:02X}", c.0, c.1, c.2);
    match action {
        apply::Action::Static(c) => hex(*c),
        apply::Action::Blink(c) => format!("blink {}", hex(*c)),
        apply::Action::Morph(a, b) => format!("morph {} <-> {}", hex(*a), hex(*b)),
    }
}

fn cmd_power(
    charging: Option<&str>,
    charged: Option<&str>,
    battery: Option<&str>,
    low: Option<&str>,
    zone: &str,
    dry_run: bool,
    common: &Common,
) -> Result<(), String> {
    let mut entries: Vec<(u8, apply::Action)> = Vec::new();
    for (name, val) in [
        ("ac-charging", charging),
        ("ac-charged", charged),
        ("bat-on", battery),
        ("bat-crit", low),
    ] {
        if let Some(s) = val {
            let code =
                protocol::state_code(name).ok_or_else(|| format!("unknown power state {name:?}"))?;
            let action = parse_action(s)?;
            if name == "bat-crit" && matches!(action, apply::Action::Morph(_, _)) {
                return Err("--low takes a single color (it is shown when the battery is critical)".into());
            }
            entries.push((code, action));
        }
    }
    if entries.is_empty() {
        return Err("nothing to do; pass --charging, --charged, --battery and/or --low".into());
    }
    let mask = zones::parse(zone)?;
    if dry_run {
        println!("zone: {zone} (mask 0x{mask:04X})");
        for (code, action) in &entries {
            println!("{:<12} {}", protocol::state_name(*code), describe(action));
        }
        return Ok(());
    }
    let mut dev = open_device(common)?;
    send_preamble(&mut dev, common)?;
    entries.sort_by_key(|(code, _)| (*code == protocol::STATE_BAT_CRIT) as u8);
    for (code, action) in &entries {
        dev.send_all(
            &apply::state_slot_packets(*code, mask, *action),
            false,
            common.verbose,
        )
        .map_err(|e| e.to_string())?;
    }
    dev.send_all(
        &[protocol::pkt_set_speed(protocol::DEFAULT_SPEED)],
        false,
        common.verbose,
    )
    .map_err(|e| e.to_string())?;
    let mut live = None;
    if let Some(cur) = power::current_state() {
        if let Some((_, action)) = entries.iter().find(|(c, _)| *c == cur) {
            dev.send_all(
                &apply::state_slot_packets(cur, mask, *action),
                false,
                common.verbose,
            )
            .map_err(|e| e.to_string())?;
            live = Some(*action);
        }
    }
    if let Some(action) = live {
        dev.send_all(
            &apply::live_block_packets(mask, action, zones::ALL_ZONES_MASK),
            false,
            common.verbose,
        )
        .map_err(|e| e.to_string())?;
    }
    for (code, action) in &entries {
        println!("{:<12} -> {}", protocol::state_name(*code), describe(action));
    }
    if live.is_none() {
        println!("note: current power state not configured; colors take effect on the next state change");
    }
    Ok(())
}

fn open_device(common: &Common) -> Result<device::Hidraw, String> {
    if let Some(p) = &common.device {
        return device::Hidraw::open(p).map_err(|e| open_error(p, &e));
    }
    let devs = device::scan().map_err(|e| format!("scanning hidraw devices: {e}"))?;
    let d = devs
        .first()
        .ok_or_else(|| "no AlienFX controller (USB 187c:0530) found".to_string())?;
    device::Hidraw::open(&d.node).map_err(|e| open_error(&d.node, &e))
}

fn open_error(path: &Path, e: &std::io::Error) -> String {
    if e.kind() == std::io::ErrorKind::PermissionDenied {
        format!("permission denied on {} ({e})\n{UDEV_HINT}", path.display())
    } else {
        format!("opening {}: {e}", path.display())
    }
}

fn parse_hex(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    if s.contains(' ') || s.contains(':') {
        s.split([' ', ':'])
            .filter(|p| !p.is_empty())
            .map(|p| u8::from_str_radix(p, 16).map_err(|_| format!("invalid byte {p:?}")))
            .collect()
    } else {
        if s.len() % 2 != 0 {
            return Err(format!("odd-length hex string: {s:?}"));
        }
        (0..s.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&s[i..i + 2], 16)
                    .map_err(|_| format!("invalid byte at offset {i}"))
            })
            .collect()
    }
}
