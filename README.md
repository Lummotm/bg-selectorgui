# bgselector
A fast and lightweight wallpaper picker with a Slint GUI. It scans your wallpaper folder, generates PNG thumbnails, and prints the path of the wallpaper you select so it can be piped into a wallpaper daemon.

## Features

- Fast, keyboard-driven Slint interface (also clickable with the mouse)
- Recursive scanning of your wallpaper folder (jpg, jpeg, png, webp, gif)
- Cached thumbnails for instant startup after the first run
- Prints the selected wallpaper path to stdout, so it plays well with `swww`, `hyprpaper`, etc.
- Optional random shuffle of the wallpaper order

## Requirements

- Rust (stable) with `cargo` — https://rustup.rs
- The standard Wayland/GL build dependencies for Slint:
  - **Arch Linux:** `base-devel` (for `pkg-config`, `cc`), and on Wayland `libxkbcommon`
  - **Debian/Ubuntu:** `build-essential pkg-config libxkbcommon-dev` and `libwayland-dev`

## Installation

```bash
git clone https://github.com/Lummotm/bgselector.git
cd bgselector
make install
```

This builds in release mode and installs the binary to `~/.local/libexec/bgselector`.

If you prefer a manual install:

```bash
cargo build --release
cp target/release/bgselector ~/.local/libexec/
```

## Usage

```bash
bgselector [OPTIONS]
```

| Option          | Description                                                    |
|-----------------|----------------------------------------------------------------|
| `-h`, `--help`  | Print help information and exit.                               |
| `-v`, `--version` | Print version information and exit.                          |
| `--dir <path>`  | Use a custom wallpaper directory (default: `~/Pictures/Wallpapers/`). |
| `--reload`      | Delete the thumbnail cache and regenerate on start.            |
| `--cache`       | Update thumbnails without launching the GUI.                   |
| `--no-shuffle`  | Disable random wallpaper order (keep alphabetical).            |

### Keyboard controls

| Key                          | Action            |
|------------------------------|-------------------|
| `←` / `h` / `Shift+Tab`      | Move back         |
| `→` / `l` / `Tab`            | Move forward      |
| `Enter` / mouse click        | Select wallpaper  |
| `Esc`                        | Close without selecting |

### Integrating with a wallpaper daemon

```bash
swww img "$(bgselector)"
```

```bash
hyprctl hyprpaper wallpaper ",$(bgselector)"
```

## Recommended window manager rules

It's recommended to let the picker fill the whole screen and hide its border. For **niri**, add a rule like this to `~/.config/niri/config.kdl`:

```kdl
window-rule {
  match title="bgselector"

  default-column-width { proportion 1.0; }
  default-window-height { proportion 1.0; }
  border {
    width 0
  }
}
```

For **hyprland**, the equivalent in `~/.config/hypr/hyprland.conf`:

```conf
windowrulev2 = size 100% 100%, title:^(bgselector)$
windowrulev2 = noborder, title:^(bgselector)$
windowrulev2 = noinitialfocus, title:^(bgselector)$
```

There are other ways to do this on both WMs; these are just starting points.

## How it works

1. Scans the wallpaper directory recursively for supported image formats.
2. Generates a cached PNG thumbnail (854x480) for every image under `~/.cache/bg-selector-gui/thumbnails/`.
3. Launches the picker; on selection it prints the absolute path of the chosen wallpaper to stdout and exits.

## Make targets

| Target          | Description                                    |
|-----------------|------------------------------------------------|
| `make`          | Build in release mode.                         |
| `make install`  | Build and install to `~/.local/libexec/`.      |
| `make clean`    | Remove build artifacts.                        |
