use std::ffi::{c_int, c_ulong};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::protocol;

pub const VID: u16 = 0x187C;
pub const PID: u16 = 0x0530;

const HIDRAW_SYSFS: &str = "/sys/class/hidraw";
const F_GETFL: c_int = 3;
const F_SETFL: c_int = 4;
const O_NONBLOCK: c_int = 0o4000;

extern "C" {
    fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
    fn fcntl(fd: c_int, cmd: c_int, ...) -> c_int;
}

fn hidraw_ioctl(kind: u8, len: usize) -> c_ulong {
    const IOC_READWRITE: c_ulong = 3;
    (IOC_READWRITE << 30)
        | ((len as c_ulong) << 16)
        | ((b'H' as c_ulong) << 8)
        | kind as c_ulong
}

#[derive(Debug, Clone)]
pub struct Detected {
    pub node: PathBuf,
    pub sysfs: PathBuf,
    pub name: String,
}

impl Detected {
    pub fn id(&self) -> String {
        format!("{VID:04x}:{PID:04x}")
    }
}

pub fn scan() -> io::Result<Vec<Detected>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(HIDRAW_SYSFS)? {
        let entry = entry?;
        let uevent = entry.path().join("device").join("uevent");
        let Ok(text) = std::fs::read_to_string(&uevent) else {
            continue;
        };
        let (mut vendor, mut product) = (None, None);
        let mut name = String::new();
        for line in text.lines() {
            if let Some(v) = line.strip_prefix("HID_ID=") {
                let parts: Vec<&str> = v.split(':').collect();
                if parts.len() == 3 {
                    vendor = u16::from_str_radix(parts[1], 16).ok();
                    product = u16::from_str_radix(parts[2], 16).ok();
                }
            } else if let Some(v) = line.strip_prefix("HID_NAME=") {
                name = v.to_string();
            }
        }
        if vendor == Some(VID) && product == Some(PID) {
            let sysfs = entry.path();
            let node = PathBuf::from("/dev").join(sysfs.file_name().unwrap_or_default());
            out.push(Detected {
                node,
                sysfs,
                name,
            });
        }
    }
    out.sort_by(|a, b| a.node.cmp(&b.node));
    Ok(out)
}

pub fn usb_string(node: &Path, key: &str) -> Option<String> {
    std::fs::read_to_string(node.join("device").join("..").join(key))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub struct Hidraw {
    file: File,
    pub path: PathBuf,
}

impl Hidraw {
    pub fn open(path: &Path) -> io::Result<Hidraw> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        Ok(Hidraw {
            file,
            path: path.to_path_buf(),
        })
    }

    pub fn open_first() -> io::Result<Hidraw> {
        let devs = scan()?;
        match devs.first() {
            Some(d) => Hidraw::open(&d.node),
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("no AlienFX controller (USB {VID:04x}:{PID:04x}) found"),
            )),
        }
    }

    pub fn write_packet(&mut self, data: &[u8]) -> io::Result<()> {
        self.file.write_all(data)
    }

    pub fn write_feature(&mut self, data: &[u8]) -> io::Result<()> {
        let rc = unsafe {
            ioctl(
                self.file.as_raw_fd(),
                hidraw_ioctl(0x06, data.len()),
                data.as_ptr(),
            )
        };
        if rc < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub fn send_all(
        &mut self,
        packets: &[[u8; protocol::PACKET_LEN]],
        use_feature: bool,
        verbose: bool,
    ) -> io::Result<()> {
        for (i, pkt) in packets.iter().enumerate() {
            if verbose {
                eprintln!("TX {:02x?}", pkt);
            }
            if use_feature {
                self.write_feature(pkt)?;
            } else {
                self.write_packet(pkt)?;
            }
            if i + 1 < packets.len() {
                std::thread::sleep(Duration::from_millis(25));
            }
        }
        Ok(())
    }

    pub fn read_for(&mut self, ms: u64) -> io::Result<Vec<Vec<u8>>> {
        let fd = self.file.as_raw_fd();
        let flags = unsafe { fcntl(fd, F_GETFL) };
        if flags >= 0 {
            unsafe { fcntl(fd, F_SETFL, flags | O_NONBLOCK) };
        }
        let deadline = Instant::now() + Duration::from_millis(ms);
        let mut out = Vec::new();
        let mut buf = [0u8; 64];
        while Instant::now() < deadline {
            match self.file.read(&mut buf) {
                Ok(0) => std::thread::sleep(Duration::from_millis(20)),
                Ok(n) => out.push(buf[..n].to_vec()),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(e) => return Err(e),
            }
        }
        Ok(out)
    }
}
