use crate::protocol as p;

pub fn no_zone_mask(all_zones: u32) -> u32 {
    !all_zones & 0x00FF_FFFF
}

pub fn set_color_packets(
    zone_mask: u32,
    rgb: (u8, u8, u8),
    all_zones: u32,
    commit: bool,
) -> Vec<[u8; p::PACKET_LEN]> {
    let mut cmds = Vec::new();
    for (_, state) in p::STATES {
        if *state == 1 {
            cmds.push(p::pkt_set_color(1, zone_mask, rgb));
            cmds.push(p::pkt_loop_block_end());
            cmds.push(p::pkt_set_color(2, no_zone_mask(all_zones), (0, 0, 0)));
            cmds.push(p::pkt_loop_block_end());
        } else {
            cmds.push(p::pkt_save_next(*state));
            cmds.push(p::pkt_set_color(1, zone_mask, rgb));
            cmds.push(p::pkt_save_next(*state));
            cmds.push(p::pkt_loop_block_end());
            if commit {
                cmds.push(p::pkt_save());
            }
        }
    }
    cmds.push(p::pkt_set_speed(p::DEFAULT_SPEED));
    cmds.push(p::pkt_set_color(1, zone_mask, rgb));
    cmds.push(p::pkt_loop_block_end());
    cmds.push(p::pkt_set_color(2, no_zone_mask(all_zones), (0, 0, 0)));
    cmds.push(p::pkt_loop_block_end());
    cmds.push(p::pkt_transmit_execute());
    cmds
}

pub fn reset_packets(kind: u8) -> Vec<[u8; p::PACKET_LEN]> {
    vec![p::pkt_reset(kind), p::pkt_transmit_execute()]
}
