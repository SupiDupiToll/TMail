# ADR 0006 — Reuse of tutabridge material

- **Status:** accepted (owner amendment)
- **Date:** 2026-11-14

## Decision

Code may be copied from `spartanz51/tutabridge` and adapted, but:

1. it is never a build or runtime dependency;
2. adapted files carry a header naming the source, and `NOTICE.md` lists them;
3. the adaptation is substantive (our own API shape, error model, tests and
   changes), not a verbatim dump;
4. both projects are GPL-3.0, so the combination stays GPL-3.0-or-later.

## What we reused and how it changed

| Area | Source | Our change |
|---|---|---|
| SDK patch stack | `sdk/patches/*` | vendored, reviewed, documented; version pinned to our own base |
| Event bus transport | `crates/tuta/src/event_bus.rs` | own module, error surface and state publishing, tests kept |
| Blob/body/attachment reads | `crates/tuta/src/mail.rs` | split metadata from payload download; typed error classes |
| Folder tree | `crates/tuta/src/folder_system.rs` | flat serialisable DTOs for the UI, path helpers |
| Mail-set-entry id codec | `crates/tuta/src/mail_set_entry_id.rs` | fallible API (no panics on untrusted event data) |
| Draft/send service sequence | `crates/bridge/src/tuta.rs` | compose model instead of SMTP parsing, explicit refusal of unsupported confidential-to-external sends |

## Why this is safe

- The reusable parts are protocol plumbing whose correctness matters more than
  novelty; the crypto remains upstream Tuta code.
- We did not inherit tutabridge's IMAP/SMTP server, its session format, its
  updater or its store; those are out of scope and would add attack surface.
- Attribution and the GPL keep the provenance clear.
