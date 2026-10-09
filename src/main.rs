use std::ffi::c_int;
use std::path::{Path, PathBuf};
use std::process::exit;

use clap::{Parser, Subcommand};

use alienfx::{apply, color, device, protocol, zones};

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
        #[arg(long, help = "Also commit to controller NVRAM so colors survive reboots")]
        save: bool,
        #[arg(long)]
        dry_run: bool,
        #[command(flatten)]
        common: Common,
    },
    SetAll {
        color: String,
        #[arg(long)]
        save: bool,
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
            save,
            dry_run,
            common,
        } => cmd_set(&zone, &color, save, dry_run, &common),
        Command::SetAll {
            color,
            save,
            dry_run,
            common,
        } => cmd_set("all", &color, save, dry_run, &common),
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
    let mut dev = open_device(common)?;
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
    match dev.status() {
        Ok(s) => {
            let state = if s == protocol::STATUS_READY {
                " (ready)"
            } else if s == protocol::STATUS_BUSY {
                " (busy)"
            } else {
                ""
            };
            println!("status:   0x{s:02X}{state}");
        }
        Err(e) => println!("status:   unavailable ({e})"),
    }
    Ok(())
}

fn cmd_set(
    zone: &str,
    color: &str,
    save: bool,
    dry_run: bool,
    common: &Common,
) -> Result<(), String> {
    let mask = zones::parse(zone)?;
    let rgb = color::parse(color)?;
    let packets = apply::set_color_packets(mask, rgb, zones::ALL_ZONES_MASK, save);
    if dry_run {
        println!("zone:   {zone} (mask 0x{mask:04X})");
        println!("color:  #{:02X}{:02X}{:02X}", rgb.0, rgb.1, rgb.2);
        println!("save:   {save}");
        println!("packets: {}", packets.len());
        for p in &packets {
            println!("{:02x?}", p);
        }
        return Ok(());
    }
    let mut dev = open_device(common)?;
    dev.send_all(&packets, false, common.verbose)
        .map_err(|e| e.to_string())?;
    println!(
        "{} -> #{:02X}{:02X}{:02X}{}",
        zone,
        rgb.0,
        rgb.1,
        rgb.2,
        if save { " (committed to NVRAM)" } else { "" }
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
