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

- Does Tuta intend to keep the `v362` model version accepted for long? The
  client tracks upstream tags; a version bump is routine.
- Should the cache keep every body by default or only opened messages? Default:
  only opened messages; a setting is planned.
- Is SQLCipher's WAL growth acceptable for 50k+ message mailboxes? Default:
  current design; measure before changing (see ADR 0004).
