# TMail — an unofficial native client for Tuta Mail

> **Unofficial.** TMail is not affiliated with, endorsed by or supported by
> Tutao GmbH. "Tuta" is used only to say which service this client talks to.

A keyboard-first desktop client for Tuta Mail on Linux, built with Rust and
Tauri. The goal is the important part of the official client with a calmer,
faster interface: a real three-pane reader, instant local search, offline
access to what you have read, and no telemetry.

```
┌────────────┬──────────────────────────┬──────────────────────────────┐
│ Inbox   12 │ Ada Lovelace      09:41  │  Re: Analytical engines      │
│ Drafts   1 │ Quarterly report  08:12  │  Ada Lovelace <ada@…>        │
│ Sent       │ Lunch             昨天     │                              │
│ …          │ …                        │  [ sanitised HTML body ]     │
└────────────┴──────────────────────────┴──────────────────────────────┘
```

## Status

Working now: sign-in with TOTP, encrypted local cache, folder tree, a
keyboard-first three-pane UI, sender avatars and a timestamp-free list, body
and attachment reads that keep safe mail formatting, send/reply/forward with
sender-alias selection, automatic E2E for Tuta recipients, move/trash/archive/
delete, local full-text search, realtime event bus and a sandboxed HTML reader.
Draft editing, rich-text compose, password-protected external mail, contacts and
calendar are next; see `docs/STATUS.md` for the exact state and
`docs/FEATURE-MATRIX.md` for the roadmap.

## Design principles

1. **Keyboard first.** Every action has a binding; the command palette is
   `Ctrl+K`. Mouse is optional, not required.
2. **Calm and dense.** Information density without visual noise. One accent,
   no gradients, no animation that delays work.
3. **Local and private.** Encrypted cache, OS keyring, no analytics, no
   network calls except to your Tuta host.
4. **Honest about security.** HTML mail is sandboxed; remote content is blocked
   until you allow it per sender. We never invent cryptography: the official
   GPL Tuta SDK does the E2E work.

## Build (Arch Linux)

```bash
sudo pacman -S --needed base-devel rust nodejs pnpm webkit2gtk-4.1 gtk3 libsoup3 openssl
git clone <this repo> && cd TMail-for-Tuta
pnpm -C ui install --frozen-lockfile
pnpm tauri build                      # or: cargo build --release -p tmail-desktop
```

A `PKGBUILD` lives in `packaging/`. It builds from the tagged source with
locked Rust and pnpm dependencies.

### Development

```bash
cargo test -p tmail-core            # headless core
cargo clippy --workspace --all-targets -- -D warnings
pnpm -C ui check
pnpm tauri dev
```

Live integration tests need a dedicated test account; they are skipped unless
`TMAIL_TEST_EMAIL` and `TMAIL_TEST_PASSWORD` are set. Never point tests at a
real mailbox.

## Security

- Cache: SQLCipher, key derived from the Tuta group key (ADR 0004).
- Session: Secret Service (gnome-keyring / KWallet); no plaintext fallback.
- Renderer: DOMPurify + a script-free sandboxed iframe + strict CSP; remote
  images blocked by default (ADR 0007). Inline CSS is stripped from mail so a
  sender cannot exfiltrate via `url()` or break the reader layout; typography is
  ours.
- Network: host allowlist, bounded concurrency, backoff on 429/5xx.
- HTML mail, attachments and event payloads are untrusted input.
- Mail CSS is filtered: presentation survives, but `url()` resource loads,
  fixed positioning and animation are removed.

## License

GPL-3.0-or-later. The vendored Tuta SDK and adapted tutabridge material are
GPL-3.0; see `LICENSE`, `NOTICE.md` and `docs/adr/0005-license.md`.
