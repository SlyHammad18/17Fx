use alienfx::profile;

#[test]
fn parse_render_roundtrip() {
    let entries = vec![(0x0008, (255, 0, 0)), (0x0041, (0, 255, 0))];
    let text = profile::render(&entries);
    assert_eq!(profile::parse(&text), entries);
}

#[test]
fn parse_skips_bad_lines() {
    let text = "# comment\n0008 FF0000\nbogus\n\n0040 00FF00\n";
    assert_eq!(
        profile::parse(text),
        vec![(0x0008, (255, 0, 0)), (0x0040, (0, 255, 0))]
    );
}

#[test]
fn merge_replaces_overlapping_zones() {
    let entries = vec![(0x000F, (1, 1, 1)), (0x0040, (2, 2, 2))];
    let merged = profile::merge(&entries, 0x0003, (9, 9, 9));
    assert_eq!(
        merged,
        vec![
            (0x000C, (1, 1, 1)),
            (0x0040, (2, 2, 2)),
            (0x0003, (9, 9, 9))
        ]
    );
}

#[test]
fn names_are_validated() {
    assert!(profile::validate_name("default").is_ok());
    assert!(profile::validate_name("gaming-2").is_ok());
    assert!(profile::validate_name("../etc").is_err());
    assert!(profile::validate_name("").is_err());
    assert!(profile::validate_name("a b").is_err());
}
