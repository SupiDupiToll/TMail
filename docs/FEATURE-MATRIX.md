# Feature matrix

Audit of the official Tuta Mail client (web/desktop, v362 model era) against
TMail. `MVP` = must work before the first release; `later` = planned; `skip` =
deliberately out of scope.

| Area | Feature | Status | Notes |
|---|---|---|---|
| Account | Password sign-in | **MVP** | core/auth.rs |
| Account | TOTP second factor | **MVP** | U2F/WebAuthn refused with a clear message |
| Account | Session in OS keyring | **MVP** | memory fallback with warning |
| Account | Multiple accounts | later | storage and service can be instantiated per account |
| Account | Sign out / remove cache | **MVP** | |
| Mail | Folder tree (system + nested custom) | **MVP** | |
| Mail | Labels | later | not shown as folders |
| Mail | Message list, newest first, virtualised | **MVP** | |
| Mail | Unread state, mark read/unread | **MVP** | |
| Mail | Message body (blob + draft storage) | **MVP** | safe mail CSS/legacy markup preserved |
| Mail | Attachment listing + save + image/text preview | **MVP** | PDF preview stays external |
| Mail | Send, reply, reply-all, forward | **MVP** | reply threading via conversation entry; send path corrected 2026-11-15 |
| Mail | Choose sender address / alias | **MVP** | From selector, validated against enabled aliases |
| Mail | Drafts (create/save, resume editing) | partial | create/save works; editing an existing draft is `later` |
| Mail | Signatures | later | settings model reserved |
| Mail | Inbox rules | later | server rules are visible in API; UI later |
| Mail | Move/trash/archive/delete | **MVP** | |
| Mail | Spam handling | **MVP** | move to Spam folder |
| Mail | Local search (subject, sender, body) | **MVP** | FTS5 |
| Mail | Import/export (EML) | later | |
| Mail | Confidential mail to Tuta recipients | **MVP** | automatic; internal recipient key data, TutaCrypt/RSA; own addresses use symmetric wrapping |
| Mail | Confidential mail to external recipients | blocked | password/external-group flow not exposed by the SDK; refused rather than downgraded |
| Mail | Offline reading of cached mail | **MVP** | headers always; bodies once opened |
| Mail | Remote content opt-in per sender | **MVP** | mail canvas is light for contrast |
| Mail | Scheduled send | **MVP** | composer clock; server `sendAt` |
| Mail | In-app attachment preview | **MVP** | images and text; PDFs open externally |
| Mail | Keyboard-first triage | **MVP** | `j/k`, `u`, `e`, `#`, `r`, `a`, `f`, `Ctrl+K` |
| Desktop | Collapsible rail + resizable columns | **MVP** | drag or arrow keys; persists |
| Contacts | Address book, autocomplete | later | SDK contact facade exists |
| Calendar | View/edit events | later | SDK calendar facade exists |
| Settings | Theme follows system | **MVP** | |
| Settings | Default sender, signature, plaintext, notifications, layout | **MVP** | local settings |
| Settings | Density, text size | **MVP** | tokens |
| Settings | Notification preferences | later | |
| Desktop | Tray icon, close-to-tray | **MVP** | |
| Desktop | Desktop notifications | **MVP** | |
| Desktop | `mailto:` handler | **MVP** | .desktop + CLI arg |
| Desktop | Single instance | **MVP** | |
| Desktop | HiDPI / fractional scaling | **MVP** | WebView handles scale; tests at 1x/1.5x |
| Desktop | Auto-update | skip | package manager owns updates |
| Desktop | Calendar/Drive apps | skip | mail client only |
| Security | Encrypted cache | **MVP** | SQLCipher |
| Security | No telemetry | **MVP** | asserted by dependency/network review |
| Security | HTML sandbox + CSP | **MVP** | ADR 0007 |
| Security | Custom-domain Tuta recipient detection | **MVP** | public-key service, no silent downgrade |
