# AGENTS.md

Instructions for AI agents and humans working in this repository.

## Project

`alienfx` is a Rust CLI that controls the AlienFX lighting controller found on
Alienware 15 R3 / 17 R4 laptops (USB HID VID:PID `187c:0530`, product string
"AW1517"). It talks to the controller directly through raw `/dev/hidraw*` file
I/O — no hidapi, libusb, or other system libraries.

## Build and test

```
cargo build --release
cargo test
./target/release/alienfx --help
```

The binary runs against real hardware on this machine. `alienfx list` and
`alienfx zones` are safe to run any time. Commands that open the device
(`set`, `set-all`, `off`, `on`, `send`) need the udev rule from
`udev/99-alienfx.rules` installed, or root.

## Hardware quick facts

- Controller: USB HID `187c:0530` ("AW1517"), 12-byte reports, report ID `0x02`
- Zone table lives in `src/zones.rs` (14 lighting regions, one bit each)
- Packet builders live in `src/protocol.rs`; command sequences in `src/apply.rs`
- Never blast raw packets with `alienfx send` without `--dry-run` first
- Current scope is static colors only; the full feature list is in `FEATURES.md`

## Rules for commits

- Do NOT add any AI/agent attribution — no "Co-Authored-By: Command Code" (or
  similar) trailers, no "generated with" notes. Plain commit messages only.
- Commit at meaningful milestones (scaffold, protocol, device I/O, docs, fixes).
