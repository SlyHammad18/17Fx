use crate::protocol as p;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Static((u8, u8, u8)),
    Blink((u8, u8, u8)),
    Morph((u8, u8, u8), (u8, u8, u8)),
}

impl Action {
    pub fn packet(self, block: u8, zone_mask: u32) -> [u8; p::PACKET_LEN] {
        match self {
            Action::Static(rgb) => p::pkt_set_color(block, zone_mask, rgb),
            Action::Blink(rgb) => p::pkt_set_blink_color(block, zone_mask, rgb),
            Action::Morph(a, b) => p::pkt_set_morph_color(block, zone_mask, a, b),
        }
    }
}

pub fn no_zone_mask(all_zones: u32) -> u32 {
    !all_zones & 0x00FF_FFFF
}

pub fn apply_preamble() -> Vec<[u8; p::PACKET_LEN]> {
    vec![p::pkt_get_status(), p::pkt_reset(p::RESET_ALL_LIGHTS_ON)]
}

pub fn set_action_packets(
    zone_mask: u32,
    action: Action,
    all_zones: u32,
    commit: bool,
    speed: u16,
) -> Vec<[u8; p::PACKET_LEN]> {
    let mut cmds = Vec::new();
    for (_, state) in p::STATES {
        cmds.push(p::pkt_save_next(*state));
        cmds.push(action.packet(1, zone_mask));
        cmds.push(p::pkt_save_next(*state));
        cmds.push(p::pkt_loop_block_end());
        if commit {
            cmds.push(p::pkt_save());
        }
    }
    cmds.push(p::pkt_set_speed(speed));
    cmds.push(action.packet(1, zone_mask));
    cmds.push(p::pkt_loop_block_end());
    cmds.push(p::pkt_set_color(2, no_zone_mask(all_zones), (0, 0, 0)));
    cmds.push(p::pkt_loop_block_end());
    cmds.push(p::pkt_transmit_execute());
    cmds
}

pub fn set_color_packets(
    zone_mask: u32,
    rgb: (u8, u8, u8),
    all_zones: u32,
    commit: bool,
) -> Vec<[u8; p::PACKET_LEN]> {
    set_action_packets(
        zone_mask,
        Action::Static(rgb),
        all_zones,
        commit,
        p::DEFAULT_SPEED,
    )
}

pub fn set_morph_packets(
    zone_mask: u32,
    a: (u8, u8, u8),
    b: (u8, u8, u8),
    all_zones: u32,
    commit: bool,
    speed: u16,
) -> Vec<[u8; p::PACKET_LEN]> {
    set_action_packets(zone_mask, Action::Morph(a, b), all_zones, commit, speed)
}

pub fn state_slot_packets(
    state: u8,
    zone_mask: u32,
    action: Action,
) -> Vec<[u8; p::PACKET_LEN]> {
    vec![
        p::pkt_save_next(state),
        action.packet(1, zone_mask),
        p::pkt_save_next(state),
        p::pkt_loop_block_end(),
        p::pkt_save(),
    ]
}

pub fn live_block_packets(
    zone_mask: u32,
    action: Action,
    all_zones: u32,
) -> Vec<[u8; p::PACKET_LEN]> {
    vec![
        action.packet(1, zone_mask),
        p::pkt_loop_block_end(),
        p::pkt_set_color(2, no_zone_mask(all_zones), (0, 0, 0)),
        p::pkt_loop_block_end(),
        p::pkt_transmit_execute(),
    ]
}

pub fn reset_packets(kind: u8) -> Vec<[u8; p::PACKET_LEN]> {
    vec![p::pkt_reset(kind), p::pkt_transmit_execute()]
}
