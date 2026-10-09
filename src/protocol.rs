pub const REPORT_ID: u8 = 0x02;
pub const PACKET_LEN: usize = 12;

pub const CMD_SET_MORPH_COLOR: u8 = 0x01;
pub const CMD_SET_BLINK_COLOR: u8 = 0x02;
pub const CMD_SET_COLOR: u8 = 0x03;
pub const CMD_LOOP_BLOCK_END: u8 = 0x04;
pub const CMD_TRANSMIT_EXECUTE: u8 = 0x05;
pub const CMD_GET_STATUS: u8 = 0x06;
pub const CMD_RESET: u8 = 0x07;
pub const CMD_SAVE_NEXT: u8 = 0x08;
pub const CMD_SAVE: u8 = 0x09;
pub const CMD_SET_SPEED: u8 = 0x0E;
pub const CMD_SET_DIM: u8 = 0x1C;

pub const DIM_ENABLE: u8 = 0x20;
pub const DIM_DISABLE: u8 = 0x40;
pub const DIM_MODE_ALWAYS: u8 = 0x01;
pub const DIM_MODE_BATTERY_ONLY: u8 = 0x00;

pub const STATUS_READY: u8 = 0x10;
pub const STATUS_BUSY: u8 = 0x11;

pub const RESET_ALL_LIGHTS_OFF: u8 = 0x03;
pub const RESET_ALL_LIGHTS_ON: u8 = 0x04;

pub const DEFAULT_SPEED: u16 = 75;
pub const TEMPO_MIN: u16 = 0x001E;
pub const TEMPO_MAX: u16 = 0x03AE;

pub const STATES: &[(&str, u8)] = &[
    ("boot", 1),
    ("ac-sleep", 2),
    ("ac-charged", 5),
    ("ac-charging", 6),
    ("bat-sleep", 7),
    ("bat-on", 8),
    ("bat-crit", 9),
];

fn pkt() -> [u8; PACKET_LEN] {
    let mut p = [0u8; PACKET_LEN];
    p[0] = REPORT_ID;
    p
}

fn put_zone(p: &mut [u8; PACKET_LEN], zone_mask: u32) {
    p[3] = (zone_mask >> 16) as u8;
    p[4] = (zone_mask >> 8) as u8;
    p[5] = zone_mask as u8;
}

pub fn pkt_set_color(block: u8, zone_mask: u32, rgb: (u8, u8, u8)) -> [u8; PACKET_LEN] {
    let mut p = pkt();
    p[1] = CMD_SET_COLOR;
    p[2] = block;
    put_zone(&mut p, zone_mask);
    p[6] = rgb.0;
    p[7] = rgb.1;
    p[8] = rgb.2;
    p
}

pub fn pkt_set_morph_color(
    block: u8,
    zone_mask: u32,
    a: (u8, u8, u8),
    b: (u8, u8, u8),
) -> [u8; PACKET_LEN] {
    let mut p = pkt();
    p[1] = CMD_SET_MORPH_COLOR;
    p[2] = block;
    put_zone(&mut p, zone_mask);
    p[6] = a.0;
    p[7] = a.1;
    p[8] = a.2;
    p[9] = b.0;
    p[10] = b.1;
    p[11] = b.2;
    p
}

pub fn pkt_set_blink_color(block: u8, zone_mask: u32, rgb: (u8, u8, u8)) -> [u8; PACKET_LEN] {
    let mut p = pkt();
    p[1] = CMD_SET_BLINK_COLOR;
    p[2] = block;
    put_zone(&mut p, zone_mask);
    p[6] = rgb.0;
    p[7] = rgb.1;
    p[8] = rgb.2;
    p
}

pub fn pkt_loop_block_end() -> [u8; PACKET_LEN] {
    let mut p = pkt();
    p[1] = CMD_LOOP_BLOCK_END;
    p
}

pub fn pkt_transmit_execute() -> [u8; PACKET_LEN] {
    let mut p = pkt();
    p[1] = CMD_TRANSMIT_EXECUTE;
    p
}

pub fn pkt_get_status() -> [u8; PACKET_LEN] {
    let mut p = pkt();
    p[1] = CMD_GET_STATUS;
    p
}

pub fn pkt_reset(kind: u8) -> [u8; PACKET_LEN] {
    let mut p = pkt();
    p[1] = CMD_RESET;
    p[2] = kind;
    p
}

pub fn pkt_save_next(state: u8) -> [u8; PACKET_LEN] {
    let mut p = pkt();
    p[1] = CMD_SAVE_NEXT;
    p[2] = state;
    p
}

pub fn pkt_save() -> [u8; PACKET_LEN] {
    let mut p = pkt();
    p[1] = CMD_SAVE;
    p
}

pub fn pkt_set_speed(speed: u16) -> [u8; PACKET_LEN] {
    let mut p = pkt();
    p[1] = CMD_SET_SPEED;
    p[2] = (speed >> 8) as u8;
    p[3] = speed as u8;
    p
}

pub fn pkt_dim(option: u8, mode: u8) -> [u8; PACKET_LEN] {
    let mut p = pkt();
    p[1] = CMD_SET_DIM;
    p[2] = option;
    p[3] = mode;
    p
}
