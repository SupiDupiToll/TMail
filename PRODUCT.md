# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

<!-- Desktop WebView (Tauri 2) on Arch Linux; UI must remain portable to other
     desktops and, later, an adaptive mobile surface. -->

## Users

Privacy-conscious Linux power users, developers and technical professionals who
already use Tuta Mail and process a large volume of mail daily on a desktop.
They work at a keyboard, often in long sessions, sometimes offline, and are
suspicious of telemetry, cloud sync and Electron-sized apps. Secondary users
are Tuta users on other Linux desktops who want a native-feeling client without
an IMAP bridge.

_Primary assumption: the owner's brief, not an interview. Confirm with real
users before a public release._

## Product Purpose

TMail is a native desktop client for Tuta Mail. It exists so that encrypted
mail can be triaged at the speed of a local app: instant list scrolling, instant
local search, read state and moves that do not wait for the network, and a
reader that never loads a tracking pixel. Success means a user clears an inbox
of 200 messages in minutes and forgets the client is there.

## Positioning

An independent, keyboard-first client that keeps Tuta's end-to-end encryption
intact while treating the mailbox as a local database (encrypted cache + FTS)
instead of a remote web page. It is not a bridge and does not expose plaintext
mail over IMAP/SMTP; the only consumer of decrypted mail is the sandboxed
renderer.

## Operating Context

- Arch Linux, Wayland preferably, X11 supported; HiDPI and fractional scaling.
- Secret Service keyring (gnome-keyring or KWallet) is the session store.
- System light/dark preference changes while the app runs.
- Tray icon, close-to-tray, desktop notifications, `mailto:` handler.
- One or a few accounts, thousands to tens of thousands of messages, many
  folders, occasional large attachments.
- Network is sometimes slow or absent; the server enforces rate limits.

## Capabilities and Constraints

- Tauri 2 shell, Rust core (`tmail-core`), Svelte 5 UI.
- Sign-in with password + TOTP; U2F/WebAuthn refused with a clear message.
- Read, search, compose, reply, forward, move, trash, archive, delete.
- Attachment download and upload; no inline rendering of active content.
- Encrypted local cache (SQLCipher) and FTS5 search; offline reading of opened
  mail.
- No telemetry, analytics or crash reporting; network allowlist is the Tuta API
  host only.
- Never invent cryptography: the official GPL Tuta Rust SDK performs E2E.
- Rate limits: no bulk sending, no background sending; 100 mails/hour is the
  server cap.
- Multiple accounts, contacts, calendar, inbox rules and draft editing are
  later phases, not current promises.

## Brand Commitments

- Name: **TMail**, explicitly labelled unofficial. No Tuta logo, icon, colour
  mark or typeface. Nominative use of "Tuta" only.
- Voice: direct, calm, precise. No hype, no exclamation marks, no emoji in
  product copy.
- The app should feel like a tool an engineer would keep open all day, not a
  marketing surface.

## Evidence on Hand

No customer quotes, benchmarks or screenshots from real accounts exist. Do not
fabricate them. Illustrative mailbox content in docs or tests must be labelled
synthetic. Real screenshots for review may be saved under `docs/review-shots/`
by the owner.

## Product Principles

1. **The keyboard is the primary input.** Every action is reachable, labelled
   and discoverable through the command palette.
2. **Local first, network second.** Lists, search and read state answer from
   the encrypted cache; the network refreshes in the background.
3. **Density with calm.** More messages per screen, fewer competing accents;
   colour carries state, not decoration.
4. **Security is visible where it matters.** Remote content blocked, encryption
   shown honestly, errors name the recovery.
5. **No dark patterns, no telemetry, no invented crypto.**

## Accessibility & Inclusion

Target WCAG 2.2 AA: 4.5:1 body contrast, visible focus on every interactive
element, complete keyboard operation, no keyboard traps, semantic roles and
names, reduced-motion support, and text scaling to 200% without loss of
function. Screen-reader support is a release gate for reading and triage, not
an afterthought.
