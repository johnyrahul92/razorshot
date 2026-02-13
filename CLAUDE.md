# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Razorshot is a Wayland-native screenshot and annotation tool written in Rust. It uses GTK4/Cairo for the UI, ashpd for xdg-desktop-portal screenshot capture, and ksni for system tray integration. Targets Wayland desktops (developed on Pop!_OS 24 / COSMIC).

## Build Commands

```bash
cargo build --release          # Release build
cargo build                    # Debug build
cargo run -- region            # Run with region capture
cargo run -- full              # Run with fullscreen capture
cargo run -- tray              # Run in system tray mode
cargo run -- config --show     # Show current config
cargo clippy                   # Lint
cargo fmt                      # Format
cargo deb --no-build           # Package .deb (after release build)
```

No test suite exists yet.

## Architecture

### Data Flow

1. **CLI** (`cli.rs`) parses args via clap into an `AppAction` enum
2. **App** (`app.rs`) creates a GTK Application and dispatches based on action
3. **Capture** (`capture/portal.rs`) runs ashpd xdg-desktop-portal call on a background thread with its own tokio runtime, returns a `cairo::ImageSurface`
4. **Region Selection** (`ui/selection_overlay.rs`) — optional fullscreen overlay where user click-drags to select a region; crops surface via `capture/region.rs`
5. **Annotation Editor** (`ui/window.rs` + `annotate/`) — GTK window with a Cairo DrawingArea canvas and toolbar
6. **Output** (`output/`) — saves PNG to disk and/or copies to clipboard

### Key Modules

- **`annotate/canvas.rs`** — Core drawing surface. Owns `CanvasState` (shared via `Rc<RefCell<>>`) holding the shape stack, undo stack, active tool, color, and draw-in-progress state. All rendering is direct Cairo calls.
- **`annotate/tools.rs`** — State machine for 5 tools (Arrow, Rectangle, Text, Freehand, Blur). Handles mouse press/motion/release to build shapes.
- **`annotate/toolbar.rs`** — GTK toolbar with tool buttons, color picker, and undo/redo.
- **`annotate/blur.rs`** — Pixelation algorithm operating on raw image data.
- **`tray/mod.rs`** — ksni system tray with channel-based communication back to the GTK main loop.
- **`output/clipboard.rs`** — Clipboard via arboard crate with wl-copy fallback.
- **`config.rs`** — TOML config at `~/.config/razorshot/config.toml`, auto-created on first run.

### Concurrency Model

- GTK runs on the main thread with event-driven callbacks
- Screenshot portal call runs on a background thread spawning its own tokio runtime
- System tray (ksni) runs on its own thread, sends actions via channel
- GTK main loop polls channels using `glib::timeout_add_local`
- Canvas state is shared via `Rc<RefCell<>>` (single-threaded interior mutability)

### App ID

`com.razorshot.Razorshot` — used for GTK application ID, desktop file, and icon naming.
