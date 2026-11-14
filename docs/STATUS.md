# Status

## Phase 0 — Research & decisions (2026-11-14)

Done.

- Studied upstream `tutanota` (README, BUILDING, HACKING, events, tuta-sdk
  API surface) and `tutabridge` (login, sync, event bus, blob/AEAD patches,
  session storage, send path).
- ADRs 0001–0007: protocol/crypto, shell, frontend, storage, license,
  tutabridge reuse, HTML-mail security.
- Feature matrix in `docs/FEATURE-MATRIX.md`.
- Legal review in `docs/LEGAL.md` (unofficial status, ToS clauses 7 and 12).

Risks:
- The official Rust SDK needs a patch stack to read bodies/attachments and
  AEAD fields. Kept small and pinned; upstream changes can invalidate it.
- No test account yet (see `docs/QUESTIONS.md`).
- Tuta may restrict third-party clients at any time.

## Phase 1 — Core crate (in progress)

Working now:

- Vendored, patched SDK builds; upstream + our unit tests pass.
- `tmail-core`: sign-in incl. TOTP, session in keyring (memory fallback),
  SQLCipher cache + FTS5, folder tree, mail listing/body/attachments, send
  and draft creation, move/trash/archive/delete, event-bus transport with
  reconnect/backoff, host allowlist and HTTP retry.
- 57 unit tests pass.

Not yet:
- Live account verification.
- Full incremental event application into the cache (events currently trigger
  a targeted folder refresh; no replay of missed batches into the store).
- Draft editing in place.

Next: Tauri commands, Svelte UI, test account smoke test.
