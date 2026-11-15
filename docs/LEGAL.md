# Legal notes

This is not legal advice. It records what we checked and which choices we made.

## Unofficial status

TMail is an independent, unofficial client. It is not produced, endorsed,
certified or supported by Tutao GmbH. The name "Tuta" is used only to describe
what the software connects to (nominative use). We do not use Tuta's logo,
icons, colour marks or typefaces, and the app identifies itself as
"TMail (unofficial)" in its About screen, user agent and session list entry.

## Tuta Terms and Conditions (as of 2022-02-23)

Checked at <https://tuta.com/terms>.

- **Clause 12 (licenses)** forbids reverse engineering, decompiling or
  disassembling the software. TMail does not do this: it links the official
  GPL-3.0 Rust SDK published by Tutao and uses the public HTTPS API that the
  official clients use.
- **Clause 7.3** caps an account at 100 sent emails per hour. TMail sends only
  in direct response to a user action; it has no bulk, scheduled or background
  sending. Composing is not throttled, but the server's limit applies.
- **Clause 7.4** forbids bulk mailings/newsletters. TMail is a personal mail
  client and does not offer list management or bulk send.
- **Clause 7.1** caps message size (20 MB unless the tariff says otherwise);
  TMail relies on the server's rejection and surfaces the error.
- The terms grant a right to use Tuta's software for the service. They do not
  explicitly authorise third-party clients. Using the API with a third-party
  client is therefore a tolerated-but-unsupported configuration; users should
  be aware that Tuta may change or restrict it. This risk is stated in the
  README.

## Trademarks

"Tuta", "Tutanota" and the Tuta logo are trademarks of Tutao GmbH. TMail uses
the word only descriptively and ships no Tuta artwork. Icons are original.

## Copyright and GPL

- TMail: GPL-3.0-or-later (`LICENSE`).
- Vendored Tuta Rust SDK: GPL-3.0, see `vendor/tuta/LICENSE.txt`.
- Adapted tutabridge code: GPL-3.0, attribution in `NOTICE.md` and in file
  headers.
- The corresponding source for any binary is this repository, including the
  exact vendored SDK and patch stack recorded in `vendor/tuta/README.md`.

## Privacy

TMail has no telemetry, analytics or crash reporting. The only mail-data
destination is the configured Tuta API host, enforced by an allowlist in the
HTTP client. The optional in-app updater additionally contacts the project's
GitHub Releases endpoint and verifies the Tauri signature; it is disabled for
package-managed Linux installs. Local data is encrypted at rest; the session
token lives in the OS keyring.
