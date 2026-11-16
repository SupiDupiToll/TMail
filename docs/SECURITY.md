# Dependency security notes

Run before a release:

```bash
cargo audit
pnpm audit --prod
cargo deny check        # if installed
```

## Current audit state (2026-11-14)

- `pnpm audit --prod`: no known vulnerabilities.
- `cargo audit`: one vulnerability and several unmaintained-crate warnings.

### Fixed

- **libsqlite3-sys / SQLite CVE-2026-119375** — `rusqlite` upgraded from 0.32
  to 0.42 (`libsqlite3-sys` >= 0.39). Cache tests and the wrong-key test pass.

### Accepted for now

- **rsa 0.9.10 — Marvin attack (RUSTSEC-2023-0071).** The crate is pinned by
  the official Tuta SDK. RSA private-key operations happen locally when
  unwrapping session keys; there is no remote oracle that observes our
  decryption timing. rsa 0.10 is a full API rewrite (`BoxedUint`,
  `TryCryptoRng`) and porting it by hand would put E2E crypto at risk. We stay
  on the upstream-supported version until the SDK moves, and re-check every
  SDK bump. This is recorded as a known residual risk, not silently ignored.
- **Unmaintained transitive crates** (`basic-toml`, `bincode`, `paste`,
  `pqcrypto-*`, `zbus*`): pulled by the Tauri GUI stack and the SDK's
  post-quantum dependency. No known exploitable vulnerability; tracked for the
  next ecosystem update.

## Rules

- Never add a dependency that performs cryptography. Use the SDK's
  `crypto-primitives`.
- Pin versions in `Cargo.lock` / `pnpm-lock.yaml`; review lockfile diffs.
- Network destinations are allowlisted to Tuta-owned domains in
  `core/src/net.rs`; a dependency that phones home would fail that test only if
  it uses our injected client, so review new dependencies for their own
  networking.
- In-app updates contact GitHub Releases only and verify the Tauri minisign
  signature against the public key in `tauri.conf.json`. `.deb`/`.rpm`/Arch
  installs disable in-app updates and use the package manager.
- Tauri's CSP nonce/hash injection is disabled for `style-src` only. This keeps
  `'unsafe-inline'` effective for Svelte's dynamic style attributes and the
  sanitized mail frame; scripts remain nonce/hash protected and the mail frame
  still runs in a sandboxed, opaque-origin iframe with its own CSP.
