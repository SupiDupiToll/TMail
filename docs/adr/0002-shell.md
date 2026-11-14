# ADR 0002 — Application shell

- **Status:** accepted
- **Date:** 2026-11-14

## Decision

**Tauri 2** with a Rust core crate (`core/`) and a web frontend (`ui/`).

## Why

- The core is a normal Rust library: a CLI, tests and future mobile bindings
  can reuse it without Tauri.
- Tauri gives native WebView, tray, notifications, single-instance handling,
  Wayland/X11 windows and small binaries compared with Electron.
- The renderer is a real sandboxed WebView; HTML mail is isolated in a
  nested sandboxed iframe, which is the strongest practical HTML-mail sandbox
  on Linux.
- Mobile is not blocked: the same core can be wrapped by a future
  Tauri mobile or native shell.

## Rejected alternatives

- **Electron**: +100 MB, Chromium bundled, weaker process model for our needs
  and a much larger attack surface.
- **Pure GTK/Qt**: excellent native feel, but no reuse of web rendering for
  mail HTML, far more work for the dense, keyboard-first UI, and no path to
  mobile.
- **egui/iced**: fast, but HTML mail rendering and accessibility are poor.

## Consequences

- Requirements: `webkit2gtk-4.1`, `gtk3`, `libsoup3` on Arch.
- The UI cannot access the network directly; all data goes through Tauri
  commands. The core enforces the Tuta-only allowlist.
- Tauri's IPC is the trust boundary; commands validate all input and never
  accept paths or HTML from the renderer without checks.
