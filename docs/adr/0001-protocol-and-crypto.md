# ADR 0001 — Protocol and cryptography layer

- **Status:** accepted
- **Date:** 2026-11-14

## Decision

Use the **official Tuta Rust SDK (`tuta-sdk`)** as the protocol and crypto
layer, vendored at a pinned upstream tag and patched locally. We do not write
our own cryptographic primitives and we do not reimplement the entity/service
protocol beyond the thin helpers in `core/`.

## Context

The SDK already contains: login and session keys, Argon2/bcrypt KDF, RSA and
X25519 key handling, AEAD primitives, entity encryption/decryption, service
definitions for every app, blob upload, contacts and calendar facades. It is
the same code Tuta ships in its Android and iOS apps.

It is, however, not complete enough for a mail client on its own: at the chosen
tag it lacks interactive 2FA, batch list loads, blob **downloads** (mail bodies
live in blob storage), bucket-key attachment resolution and AEAD field
decryption. These are protocol features, not opinions.

## Options considered

| Option | Verdict |
|---|---|
| **(a) Use the official SDK as-is** | Rejected: cannot read bodies or attachments, cannot sign in with TOTP, cannot decrypt AEAD fields. |
| **(b) Use + extend the SDK** | **Chosen.** Vendor a pinned copy, apply a reviewed patch stack, write all client logic in our core. |
| **(c) Reimplement parts** | Rejected for crypto/protocol: high risk of subtle E2E failures; only UI-facing helpers (folder tree, id codec, sync) are ours. |
| **(d) Embed the official TS SDK** | Rejected: there is no standalone TS SDK package; the web app is a browser/worker/IndexedDB bundle with WASM builds. Embedding it would mean shipping a second client runtime and would violate "written from scratch". |

## Consequences

- Forks must track upstream releases, because Tuta rejects old client versions
  (`InvalidSoftwareVersionError`). A version bump is a normal maintenance task.
- The patch stack is part of our attack surface and must be reviewed; patches
  are kept small, one concern each, and documented in `vendor/tuta/PATCHES.md`.
- All cryptography stays in upstream code paths. Our patches only wire existing
  primitives into the entity decoder.
- Licensing: the SDK is GPL-3.0; TMail is GPL-3.0-or-later (ADR 0005).

## Update procedure

1. Fetch the new upstream tag; note its commit.
2. Rebase `vendor/tuta/patches/*` (drop patches that upstream merged).
3. Run upstream SDK tests, our core tests, and a live smoke test on the test
   account.
4. Record the new tag/commit in `vendor/tuta/README.md` and `PATCHES.md`.
