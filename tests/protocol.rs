use alienfx::{apply, color, protocol, zones};

#[test]
fn set_color_packet_layout() {
    let pkt = protocol::pkt_set_color(1, 0x0008, (0xAB, 0xCD, 0xEF));
    assert_eq!(
        pkt,
        [0x02, 0x03, 0x01, 0x00, 0x00, 0x08, 0xAB, 0xCD, 0xEF, 0x00, 0x00, 0x00]
    );
}

#[test]
fn morph_packet_layout() {
    let pkt = protocol::pkt_set_morph_color(2, 0x0040, (0, 0, 255), (255, 0, 0));
    assert_eq!(
        pkt,
        [0x02, 0x01, 0x02, 0x00, 0x00, 0x40, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00]
    );
}

#[test]
fn blink_packet_layout() {
    let pkt = protocol::pkt_set_blink_color(1, 0x000F, (255, 255, 0));
    assert_eq!(
        pkt,
        [0x02, 0x02, 0x01, 0x00, 0x00, 0x0F, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00]
    );
}

#[test]
fn reset_packet() {
    assert_eq!(
        protocol::pkt_reset(protocol::RESET_ALL_LIGHTS_OFF),
        [0x02, 0x07, 0x03, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        apply::reset_packets(protocol::RESET_ALL_LIGHTS_ON),
        vec![
            [0x02, 0x07, 0x04, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0x02, 0x05, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        ]
    );
}

#[test]
fn masks() {
    assert_eq!(zones::ALL_ZONES_MASK, 0x7DEF);
    assert_eq!(zones::KEYBOARD_MASK, 0xF);
    assert_eq!(apply::no_zone_mask(zones::ALL_ZONES_MASK), 0xFF8210);
    let or = zones::ZONES.iter().fold(0u32, |a, z| a | z.mask);
    assert_eq!(or, zones::ALL_ZONES_MASK);
}

#[test]
fn zone_parse() {
    assert_eq!(zones::parse("all").unwrap(), 0x7DEF);
    assert_eq!(zones::parse("keyboard").unwrap(), 0xF);
    assert_eq!(zones::parse("logo,touchpad").unwrap(), 0xC0);
    assert_eq!(zones::parse("alien-head+media-bar").unwrap(), 0x4020);
    assert!(zones::parse("bogus").is_err());
    assert!(zones::parse("").is_err());
}

#[test]
fn color_parse() {
    assert_eq!(color::parse("red").unwrap(), (255, 0, 0));
    assert_eq!(color::parse("#00ff00").unwrap(), (0, 255, 0));
    assert_eq!(color::parse("0af").unwrap(), (0, 170, 255));
    assert!(color::parse("nope").is_err());
}

#[test]
fn set_color_sequence() {
    let pkts = apply::set_color_packets(0xF, (255, 0, 0), zones::ALL_ZONES_MASK, true);
    assert_eq!(pkts.len(), 40);
    assert_eq!(pkts.last().unwrap()[1], protocol::CMD_TRANSMIT_EXECUTE);
    let without_commit = apply::set_color_packets(0xF, (1, 2, 3), zones::ALL_ZONES_MASK, false);
    assert_eq!(without_commit.len(), 34);
}

#[test]
fn speed_packet() {
    assert_eq!(
        protocol::pkt_set_speed(75),
        [0x02, 0x0E, 0x00, 0x4B, 0, 0, 0, 0, 0, 0, 0, 0]
    );
}
