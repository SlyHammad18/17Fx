# Feature Roadmap

## Implemented now (colors only)

- `list` — find the AlienFX controller (USB 187c:0530) and check permissions
- `zones` — all 14 lighting regions with masks
- `probe` — device identity + controller status
- `set <zone|all> <color>` — static color, single zone or comma-separated list
- `set-all <color>` — every region at once
- `off` / `on`
- `send <hex>` — raw packet debugging (output/feature, `--dry-run`, `--read-ms`)
- Session persistence: colors survive sleep and power-state changes
- `--save`: commit to controller NVRAM so colors survive reboots

## Planned (packet builders already exist in `src/protocol.rs`)

- `blink <zone> <color>` — pulse effect (CMD 0x02 `SET_BLINK_COLOR`)
- `morph <zone> <color1> <color2>` — two-color morph (CMD 0x01 `SET_MORPH_COLOR`)
- `speed <n>` — blink/morph tempo (CMD 0x0E, range 0x1E–0x3AE, lower = faster)
- Per-power-state colors — distinct colors for boot, ac-sleep, ac-charged,
  ac-charging, bat-sleep, bat-on, bat-crit (CMD 0x08 slots, state table in
  `src/protocol.rs::STATES`)
- Theme save/load — JSON/TOML files mapping zones to colors, `apply <theme>`
- Config file — default color, default device, NVRAM-commit on/off
  (`~/.config/alienfx/config.toml`)
- Effect daemon — breathing, color wave across zones, ripple-on-keypress
  (evdev listener), audio-reactive mode
- GNOME/KDE tray applet with quick color toggle
- Import Alienware Command Center (Windows) themes
- Per-key RGB exploration on newer AlienFX controllers
- Extended 24-bit zone masks (CMD-level support already in packet builders)
