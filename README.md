# Razorshot

A native Wayland screenshot and annotation tool built with Rust + GTK4, designed for Pop!_OS 24 (COSMIC desktop) and other Wayland-based Linux desktops.

Flameshot doesn't support Wayland properly — Razorshot is the fix.

## Features

- **Screenshot capture** via `xdg-desktop-portal` (works natively on Wayland)
- **Region selection** with fullscreen overlay, click-and-drag to select
- **Annotation editor** with 5 tools:
  - Arrow
  - Rectangle
  - Text (with inline text entry)
  - Freehand drawing
  - Blur/Pixelate
- **Undo/Redo** support (Ctrl+Z / Ctrl+Y)
- **Clipboard copy** via arboard with `wl-copy` fallback
- **System tray** integration (StatusNotifierItem via ksni)
- **Save to PNG** with configurable directory and timestamp filename
- **TOML configuration** at `~/.config/razorshot/config.toml`
- **CLI interface** for scripting and keybindings
- **Low memory footprint** — Rust, no garbage collector, no Electron

## Installation

### Build from source

**Prerequisites:**

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# System dependencies (Ubuntu/Pop!_OS/Debian)
sudo apt install libgtk-4-dev libcairo2-dev libpango1.0-dev libgdk-pixbuf-2.0-dev libglib2.0-dev libdbus-1-dev pkg-config
```

**Build and install:**

```bash
git clone https://github.com/johnyrahul92/razorshot.git
cd razorshot
cargo build --release
sudo cp target/release/razorshot /usr/local/bin/
```

### Install via .deb package

```bash
cargo install cargo-deb
git clone https://github.com/johnyrahul92/razorshot.git
cd razorshot
cargo build --release
cargo deb --no-build
sudo dpkg -i target/debian/razorshot_0.1.0-1_amd64.deb
sudo apt-get install -f  # install any missing dependencies
```

## Usage

```bash
# Region selection + annotation editor
razorshot region

# Region selection, save immediately (no editor)
razorshot region --no-edit

# Full screen capture + annotation editor
razorshot full

# Full screen capture, save immediately
razorshot full --no-edit

# Full screen capture of a specific monitor
razorshot full --monitor 1

# Start in system tray (right-click for menu)
razorshot tray

# Default action (reads from config, defaults to tray)
razorshot

# View current configuration
razorshot config --show

# Change save directory
razorshot config --save-dir ~/Screenshots
```

### Keyboard shortcuts (annotation editor)

| Key | Action |
|-----|--------|
| Ctrl+Z | Undo |
| Ctrl+Y | Redo |
| Escape | Cancel / Close |

### Bind to a keyboard shortcut

Add a keybinding in your desktop settings to run:

```
razorshot region
```

For example, bind `Print Screen` to `razorshot region` and `Shift+Print Screen` to `razorshot full`.

## Configuration

Config file is created automatically at `~/.config/razorshot/config.toml` on first run:

```toml
save_dir = "~/Pictures/Screenshots"
filename_template = "Screenshot_%Y-%m-%d_%H-%M-%S.png"

[annotation]
default_color = "#ff0000"
line_width = 3.0
font_size = 16.0
blur_block_size = 10

[behavior]
open_editor = true
copy_to_clipboard = true
show_notification = true
default_action = "tray"
```

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust |
| UI Framework | GTK4 (plain, not libadwaita) |
| Screenshot | ashpd (xdg-desktop-portal) |
| Drawing | Cairo via cairo-rs |
| Text rendering | Pango via pangocairo |
| Clipboard | arboard + wl-copy fallback |
| System tray | ksni (StatusNotifierItem) |
| CLI | clap |
| Config | serde + toml |
| Packaging | cargo-deb |

## Project Structure

```
src/
├── main.rs                # Entry point, CLI dispatch
├── app.rs                 # GtkApplication setup, action routing
├── cli.rs                 # clap argument definitions
├── config.rs              # TOML config loading/saving
├── capture/
│   ├── portal.rs          # xdg-desktop-portal screenshot via ashpd
│   └── region.rs          # Post-capture cropping logic
├── annotate/
│   ├── canvas.rs          # GTK4 DrawingArea + Cairo rendering
│   ├── tools.rs           # Tool state machines
│   ├── shapes.rs          # Shape data structures
│   ├── toolbar.rs         # Tool buttons, color picker, undo/redo
│   └── blur.rs            # Pixelation algorithm
├── output/
│   ├── clipboard.rs       # Copy image via arboard / wl-copy
│   └── file.rs            # Save PNG with timestamp
├── tray/
│   └── mod.rs             # System tray + channel to GTK
└── ui/
    ├── window.rs          # Annotation editor window
    └── selection_overlay.rs  # Fullscreen region selector
```

## Tested On

- Pop!_OS 24.04 (COSMIC desktop)

Should work on any Wayland desktop with `xdg-desktop-portal` support (GNOME, KDE Plasma, Sway, Hyprland, etc.).

## Known Limitations

- System tray requires a StatusNotifierItem host (most modern desktops have one)
- Clipboard via arboard may not work on all Wayland compositors — falls back to `wl-copy` automatically
- Multi-monitor: portal may return a stitched image; use `--monitor` flag to capture a specific display

## Contributing

Contributions are welcome! Feel free to open issues and pull requests.

## License

[MIT](LICENSE)
