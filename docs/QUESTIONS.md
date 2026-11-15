# Open questions / blocked items

We do not wait for answers; the safest default is implemented and noted here.

## Needed from the owner

1. **Test account credentials.** A dedicated Tuta account (not a personal
   mailbox) with:
   - address + password,
   - TOTP secret or the ability to disable 2FA during tests,
   - at least one custom folder and a few messages with attachments.
   Without it, integration tests and real send/receive verification cannot run.
   Default: all tests are unit/mocked; live tests are gated by
   `TMAIL_TEST_EMAIL` / `TMAIL_TEST_PASSWORD` and skipped otherwise.
2. **TOTP for CI.** A fixed test secret would let CI test the 2FA path. Default:
   the path is unit-tested with mocked services and manually exercised by the
   owner.
3. **Trademark/name confirmation.** "TMail" is a working name. If it conflicts,
   a rename touches only the bundle id, .desktop file and strings.

## Open technical questions

- The send path was corrected on 2026-11-15 (empty `bodyText`, own-address
  symmetric wrapping, confidential attachment keys, correct reply Message-ID).
  It still needs a live send test before it is considered proven.
- Sender logos/BIMI are deliberately not fetched: that would make requests to
  arbitrary third-party domains and creates a phishing signal. Tuta contact
  photos would be the privacy-preserving source; that is a later phase.
- Server-side settings (rules, vacation, templates) remain in Tuta web until
  their facades are implemented and tested.

- Does Tuta intend to keep the `v362` model version accepted for long? The
  client tracks upstream tags; a version bump is routine.
- Should the cache keep every body by default or only opened messages? Default:
  only opened messages; a setting is planned.
- Is SQLCipher's WAL growth acceptable for 50k+ message mailboxes? Default:
  current design; measure before changing (see ADR 0004).
- `rsa` 0.9.10 has a Marvin-attack advisory. It is pinned by the official SDK;
  porting to 0.10 by hand would put E2E crypto at risk. Default: stay on the
  upstream-supported version (see `docs/SECURITY.md`). Revisit on the next SDK
  bump.
- Password-protected external mail needs Tuta's external-group creation and a
  Bcrypt KDF. The Rust SDK exposes neither at this tag. Default: refuse the
  mode and show an explicit "Not encrypted" state. Revisit with a live account.
- Custom-domain Tuta recipients are indistinguishable from external addresses
  by domain alone. Default: only `@tuta.com`, `@tutanota.com`, `@tuta.io` and
  `@tutanota.de` are auto-encrypted; custom domains send unencrypted with the
  external notice.
- Which Tuta blob CDN hosts are used in production? The allowlist currently
  permits the API host plus `*.tuta.com`, `*.tutanota.com`, `*.tuta.io`,
  `*.tutanota.de`. If a real attachment needs another Tuta-owned host, add it
  deliberately and record it in the ADR.
