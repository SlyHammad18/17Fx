# alienfx

CLI to control the AlienFX lighting of Alienware 15 R3 / 17 R4 laptops on
Linux. Talks to the lighting controller (USB HID `187c:0530`, product string
"AW1517") directly through raw `/dev/hidraw` — no vendor software, no
hidapi, no system libraries beyond a libc.

## Build

```
cargo build --release
```

Binary: `target/release/alienfx`.

## Permissions (one-time setup)

```
sudo cp udev/99-alienfx.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger
```

If your user is not in `plugdev`: `sudo usermod -aG plugdev $USER`, then log
out and back in. Without this, run commands with `sudo`.

## Usage

```
alienfx list                          # find the controller
alienfx zones                         # all 14 lighting regions
alienfx probe                         # device info + controller status
alienfx set <zone> <color>            # one zone, or a comma-separated list
alienfx set-all <color>               # every region at once
alienfx off                           # all lights off
alienfx on                            # all lights on
alienfx send "02 07 03" --dry-run     # raw packets for debugging
```

Colors: `#RRGGBB`, `#RGB`, or a name: black, white, red, green, blue, yellow,
cyan, magenta, orange, purple.

Examples:

```
alienfx set-all red
alienfx set keyboard-left yellow
alienfx set logo,touchpad "#00aaff"
alienfx set all magenta --no-save
```

Colors are written to the controller's power-state slots by default, so they
survive power-state changes and reboots. `--no-save` applies live only and
leaves the saved slots untouched — note that the power button follows the
saved slots, so with `--no-save` it can revert to its factory charging morph.

Useful flags: `--device /dev/hidrawN` (override auto-detection), `-v/--verbose`
(packet hex dumps), `send --mode feature`, `send --read-ms N` (listen for
responses), `send --dry-run`.

## Lighting regions (Alienware 17 R4)

| Mask   | Zone name       | Region                            |
|--------|-----------------|-----------------------------------|
| 0x0001 | keyboard-right  | Keyboard right block              |
| 0x0002 | keyboard-mright | Keyboard middle-right block       |
| 0x0004 | keyboard-mleft  | Keyboard middle-left block        |
| 0x0008 | keyboard-left   | Keyboard left block               |
| 0x0020 | alien-head      | Alien head on the lid (outside)   |
| 0x0040 | logo            | Alienware logo below the screen   |
| 0x0080 | touchpad        | Touchpad                          |
| 0x0100 | power-button    | Power button                      |
| 0x0400 | speaker-left    | Bottom front edge, left light bar |
| 0x0800 | speaker-right   | Bottom front edge, right light bar|
| 0x1000 | display-left    | Display top edge, left light bar  |
| 0x2000 | display-right   | Display top edge, right light bar |
| 0x4000 | media-bar       | Macro/media key bar               |

Aliases: `keyboard` = 0x000F (all four keyboard blocks), `all` = 0x7DEF
(every region).

## Notes

- Colors are committed to the controller's slots (states: boot, ac-sleep,
  ac-charged, ac-charging, bat-sleep, bat-on, bat-crit), overwriting whatever
  theme was stored there before — including factory indicators such as the
  charging morph on the power button.
- Touchpad lighting must be set to "always on" in the BIOS for the touchpad
  zone to respond.
- The hidraw index can change after a replug; `alienfx list` shows the current
  node and `--device` overrides detection.
- `send` expects packets starting with the report id `02`; without it you get
  a warning.

## Protocol summary

12-byte HID output reports, first byte is the report id `0x02`:

| Cmd  | Name               | Packet layout                                  |
|------|--------------------|------------------------------------------------|
| 0x01 | morph color        | `02 01 <blk> <z16 z8 z0> <r g b> <R G B>`      |
| 0x02 | blink color        | `02 02 <blk> <z16 z8 z0> <r g b> 00 00 00`     |
| 0x03 | set color          | `02 03 <blk> <z16 z8 z0> <r g b> 00 00 00`     |
| 0x04 | loop block end     | `02 04 ...`                                    |
| 0x05 | transmit execute   | `02 05 ...` (must be sent last)                |
| 0x06 | get status         | `02 06 ...` → feature report: 0x10 ready, 0x11 busy |
| 0x07 | reset              | `02 07 <t>` — t=3 all off, t=4 all on          |
| 0x08 | save next to slot  | `02 08 <state>`                                |
| 0x09 | save slots         | `02 09 ...` (commit to NVRAM)                  |
| 0x0E | set speed          | `02 0E <hi lo>` — 0x1E–0x3AE, lower = faster   |

Power states: 1 boot, 2 ac-sleep, 5 ac-charged, 6 ac-charging, 7 bat-sleep,
8 bat-on, 9 bat-crit.

A static color is applied as: boot block + one slot per power state
(`SAVE_NEXT` / `SET_COLOR` / `LOOP_BLOCK_END` [+ `SAVE` with `--save`]) + live
apply (`SET_SPEED`, color blocks, `TRANSMIT_EXECUTE`). See `src/apply.rs`.

## Roadmap

See [FEATURES.md](FEATURES.md) — blink, morph, speed, themes, effect daemon
and more.

## References

- [trackmastersteve/alienfx](https://github.com/trackmastersteve/alienfx) —
  zone bits and protocol documentation
- [kiniavinash/alienware-light-control](https://github.com/kiniavinash/alienware-light-control)
  — transport and apply sequences proven on this exact controller
- [OpenRGB AlienFX wiki](https://gitlab.com/OpenRGBDeveloper/OpenRGB/-/wikis/Alienware-AlienFX)
  — protocol background
