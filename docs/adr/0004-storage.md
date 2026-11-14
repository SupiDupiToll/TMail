# ADR 0004 — Local storage and offline behaviour

- **Status:** accepted
- **Date:** 2026-11-14

## Decision

One **SQLCipher** database per account at
`$XDG_DATA_HOME/tmail/accounts/<hash>/cache.db`, keyed by
`BLAKE3-KDF(user group key, "tmail local cache v1", 32 bytes)`.

Inside the encrypted database:

- `mails` — header DTOs per (mail, folder), unread flag, cached plain-text body.
- `bodies` — decrypted HTML + text, written when a message is opened.
- `mail_fts` — FTS5 index over subject, sender and body text.
- `folders`, `event_cursors`, `meta`.

Nothing outside the database is plaintext except the SQLite WAL/temp files,
which SQLCipher encrypts too. The SDK's type-model JSON is schema data, not
account data.

## Why SQLCipher and not per-file encryption

- Search must be fast across 10k+ messages; FTS5 inside the encrypted DB gives
  that without a separate index format.
- One key, one lifecycle, no orphaned encrypted files.
- `secure_delete=ON` makes deletion meaningful.
- Bundled OpenSSL makes the Arch package self-contained and reproducible.

## Offline behaviour

- Folder lists and previously opened bodies are served from the cache first;
  a network refresh replaces them.
- If a body was never cached and the network is down, the reader shows the
  headers and a retry action rather than a blank screen.
- Sending requires the network; the composer keeps the draft text in memory
  and tells the user it is not saved to the server until a draft is created.

## Search

FTS5 with `unicode61 remove_diacritics 2`; user input is tokenised and quoted,
so `:`, `*`, quotes and operator words cannot break the query. Results are
deduplicated across folder placements.

## Key handling

- The derived cache key is a `Zeroizing<Vec<u8>>` and is wiped after
  `PRAGMA key`; SQLCipher keeps it internally.
- The Tuta session (access token + encrypted passphrase key) lives in the
  Secret Service. There is no plaintext fallback: without a keyring the user
  gets an in-memory session and a visible warning.
- A wrong key cannot open the DB; the failure is reported as `cache_locked`,
  never as "empty mailbox".

## Future

Per-message encrypted body files (the tutabridge model) would reduce rewrite
cost on large mailboxes. Deferred until measurements show SQLCipher WAL growth
is a problem; the `Store` API is small enough to swap.
