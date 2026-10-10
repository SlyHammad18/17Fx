# Feature Roadmap

## Implemented now (colors only)

- `list` — find the AlienFX controller (USB 187c:0530) and check permissions
- `zones` — all 14 lighting regions with masks
- `probe` — device identity
- `set <zone|all> <color>` — static color, single zone or comma-separated list
- `set-all <color>` — every region at once
- `off` / `on`
- `--brightness <0-100>` — scale any set/set-all color
- `dim` / `undim` — controller dim mode (CMD 0x1C, `--battery-only` supported)
- Configuration profiles — every set updates `~/.config/alienfx/state`;
  `save [name]`, `apply [name]`, `profiles` snapshot and restore configs
- `power` — per-power-state colors (charging / charged / battery / low-battery),
  including morphs per state (`--charging red,blue`)
- `morph <zone> <c1> <c2> [--speed n]` — looping color transition on any zone
- `blink <zone> <color> [--speed n]` — pulse a zone between a color and off
- `send <hex>` — raw packet debugging (output/feature, `--dry-run`, `--read-ms`)
- Session persistence: colors survive sleep, power-state changes and reboots
  (power-state slots are committed by default; `--no-save` opts out)

## Planned (packet builders already exist in `src/protocol.rs`)

- Touchpad lighting mode control — explore Dell BIOS tokens via libsmbios
  (`smbios-token-ctl`) for an "on touch / always on" switch from Linux;
  the AlienFX wire protocol itself has no command for it
- `speed <n>` — standalone tempo command (CMD 0x0E, range 0x1E–0x3AE, lower = faster)
- More power states in `power` — boot, ac-sleep, bat-sleep colors
  (CMD 0x08 slots, state table in `src/protocol.rs::STATES`)
- Config file — default color, default device, NVRAM-commit on/off
  (`~/.config/alienfx/config.toml`)
- Effect daemon — breathing, color wave across zones, ripple-on-keypress
  (evdev listener), audio-reactive mode
- GNOME/KDE tray applet with quick color toggle
- Import Alienware Command Center (Windows) themes
- Per-key RGB exploration on newer AlienFX controllers
- Extended 24-bit zone masks (CMD-level support already in packet builders)
