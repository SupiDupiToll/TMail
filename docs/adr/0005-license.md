# ADR 0005 — License

- **Status:** accepted
- **Date:** 2026-11-14

## Decision

TMail is licensed **GPL-3.0-or-later**.

## Reason

It links the official `tuta-sdk`, which is GPL-3.0 (`LICENSE.txt` in the
Tutanota repository), and it adapts code from `spartanz51/tutabridge`, also
GPL-3.0. A permissive license is not available while using these components.
GPL-3.0-or-later matches the SDK's "or later" spirit and keeps the whole
derivative free.

## Obligations we meet

- Full license text in `LICENSE`.
- Source for the exact binary is this repository, including the vendored SDK
  and patch stack.
- `NOTICE.md` lists upstream projects and adapted files.
- No Tuta trademarks or logos; the product is labelled unofficial.
- The `PKGBUILD` builds from the tagged source archive.

## Note

Upstream tutao/tutanota does not accept AI-assisted contributions. We never
send patches there; SDK changes stay in this repository's patch stack.
