# Blur-AutoClicker — macOS Port

This repo contains the macOS-specific Rust backend files for [Blur-AutoClicker](https://github.com/Blur009/Blur-AutoClicker).

## What's changed from the Windows version

| File | Windows | macOS |
|------|---------|-------|
| `src-tauri/src/engine/mouse.rs` | `windows-sys` (`SendInput`, `SetCursorPos`, `GetCursorPos`, `GetSystemMetrics`) | Core Graphics FFI (`CGEventCreateMouseEvent`, `CGWarpMouseCursorPosition`, `CGEventCreate`) |
| `src-tauri/src/hotkeys.rs` | `WH_KEYBOARD_LL` / `WH_MOUSE_LL` hooks, `GetAsyncKeyState` | Carbon `GetKeys` + `CGEventTap` for scroll wheel |
| `src-tauri/src/overlay.rs` | Win32 `SetWindowLongW`, `DwmSetWindowAttribute`, `SetWindowPos` | Tauri cross-platform window API only (`set_decorations`, `set_always_on_top`, `hide`/`show`) |
| `src-tauri/Cargo.toml` | `windows-sys`, `winreg`, `windows-targets` | Removed — uses macOS system frameworks via `#[link]` |

## How to apply

Copy these 4 files into your local `Blur-AutoClicker` clone, replacing the originals:

```bash
REPO=/Users/amm/Documents/Blur-AutoClicker-3.4.1
MAC=/Users/amm/Documents/Blur

cp $MAC/src-tauri/src/engine/mouse.rs  $REPO/src-tauri/src/engine/mouse.rs
cp $MAC/src-tauri/src/hotkeys.rs       $REPO/src-tauri/src/hotkeys.rs
cp $MAC/src-tauri/src/overlay.rs       $REPO/src-tauri/src/overlay.rs
cp $MAC/src-tauri/Cargo.toml           $REPO/src-tauri/Cargo.toml
```

Then build:

```bash
cd $REPO
npm install
npm run tauri build
```

## Permissions required on macOS

- **Accessibility** — needed for `CGEventTap` (scroll hotkeys) and simulated mouse clicks.
  Go to **System Settings → Privacy & Security → Accessibility** and add the built app.
- **Input Monitoring** — may also be required on macOS 14+.
