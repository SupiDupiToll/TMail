# NOTICE

TMail is an unofficial client and is not affiliated with Tutao GmbH.

## Upstream components

### tutao/tutanota — Tuta Rust SDK

- Source: <https://github.com/tutao/tutanota>
- Vendored path: `vendor/tuta/tuta-sdk/`
- Version: `tutanota-release-362.261020.0`, commit
  `f5e2d8628f17be558fe63130396ff625bbe69c77`
- License: GPL-3.0 (`vendor/tuta/LICENSE.txt`)
- Local changes: `vendor/tuta/PATCHES.md` and `vendor/tuta/patches/`

Tuta Mail and the Tuta SDK are Copyright (c) Tutao GmbH and contributors.

### spartanz51/tutabridge — adapted material

- Source: <https://github.com/spartanz51/tutabridge>
- License: GPL-3.0
- Use: reference and adapted implementation, per ADR 0006. Not a dependency.

Adapted files in this repository:

| File | Origin |
|---|---|
| `core/src/event_bus.rs` | `crates/tuta/src/event_bus.rs` |
| `core/src/folder*` (`folders.rs`) | `crates/tuta/src/folder_system.rs` |
| `core/src/mail_set_entry_id.rs` | `crates/tuta/src/mail_set_entry_id.rs` |
| `core/src/mail_ext.rs` | `crates/tuta/src/mail.rs` |
| `core/src/send.rs` | `crates/bridge/src/tuta.rs` (draft/send section) |
| `vendor/tuta/patches/*` | `sdk/patches/*` (reviewed, pinned, documented) |

Changes are described per-file in the module documentation and in
`docs/adr/0006-tutabridge-reuse.md`.

### Fonts

- **Atkinson Hyperlegible Next** — Copyright 2020-2024 Braille Institute of
  America, Inc. SIL Open Font License 1.1; license text in
  `ui/static/fonts/OFL-Atkinson.txt`.
- **Source Serif 4** — Copyright 2014-2024 Adobe. SIL Open Font License 1.1;
  license text in `ui/static/fonts/OFL-SourceSerif.txt`.

Both are self-hosted; no font CDN is contacted at runtime.

### Other dependencies

The Rust and JavaScript dependency trees are pinned in `Cargo.lock` and
`ui/pnpm-lock.yaml`. Their licenses are reported by `cargo deny` and
`pnpm licenses list`; all are compatible with GPL-3.0.
