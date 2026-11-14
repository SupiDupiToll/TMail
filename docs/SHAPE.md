# Surface brief — Mail workspace (desktop, Operate)

_Written without a human interview. Assumptions are marked **[assumed]** and
derive from the owner's brief. This surface is code-led: Operate screens are
built, not comped._

## Job and audience

A Tuta user sits down to clear mail. They know the keyboard, have 5k–50k
messages, several folders, and want to reach zero without waiting on spinners.
They arrive with a goal ("reply to Ada, archive the rest"), not to admire the
app. Visitor mode: **Operate**.

## Outcome and proof

Primary task: scan the list, open a message, reply/forward/archive/delete, and
search the archive — all without leaving the keyboard. Success: the first row
is visible in under 300 ms from the encrypted cache, `j/k` moves instantly,
`Ctrl+K` reaches every action, and remote content never loads by itself.

Real evidence: the owner's own mailbox during review. No invented metrics or
testimonials.

## Selected direction

**"The quiet workspace."** A warm paper ground, ink text, a single verdigris
accent for action/selection, and soft rounded surfaces. The signature move is
the **sender avatar** — a deterministic tinted initial disc in every row and in
the reader, so a mailbox is recognisable at a glance. Timestamps are removed
from the list; sender weight and subject carry the scan, and the reader shows
the full date.

- Palette: paper/ink neutrals; verdigris `#1f6f63` (light) / `#5ec8b3` (dark);
  brass `#b07d2b` (light) / `#e0b25a` (dark); state red/amber kept semantic.
- Type: **Atkinson Hyperlegible Next** for UI (legibility at density),
  **Source Serif 4** for message bodies, system mono only for protocol/IDs.
- Density: compact 58 px rows (default), comfortable 70 px, spacious 82 px.
- Hierarchy: folder rail 248 px; list flexible (min 340 px); reader flexible.
  No toolbar of icons; one top strip carries search, sync state and account.
- Motion: one authored moment — the selected row's surface cross-fades over
  120 ms with an ease-out; no row fade-ins, no skeleton shimmer loops except a
  single subtle progress line while a folder loads.

## Scope and boundaries

In scope: sign-in + TOTP, folder rail, virtualised list, reader with sandboxed
HTML, attachments, compose/reply/forward, search, command palette, settings
sheet, empty/error/offline states, light/dark/system theme.

Untouched: server-side rules, contacts, calendar, multi-account UI (data layer
supports it; UI later), auto-update.

Anti-goals: a webmail clone with a logo wall; a Gmail-style coloured chip for
every label; cards inside cards; a permanently visible formatting toolbar;
modal dialogs for reading; onboarding carousels.

## States and ranges

- 0 messages (new account), 1 message, 200, 20 000; 40-character subjects;
  senders with no display name; 30 folders nested 4 deep.
- Loading (first open), refreshing (cached shown + a 2 px progress strip),
  offline (banner with retry), expired session (re-auth sheet), empty search,
  search with 200 hits, attachment download progress, send failure with the
  draft preserved.
- Light, dark, system; 1x and 1.5x scale; 800×600, 1280×800, 1920×1080.

## Interaction and layout

- `Ctrl+K` command palette; `/` focuses search; `j/k` next/prev; `Enter` open;
  `u` unread; `e` archive; `#` trash; `r/a/f` reply/reply-all/forward; `c`
  compose; `x` select; `Esc` closes overlay/returns focus.
- Selection is one row at a time; multi-select shows a count bar with bulk
  actions. Focus ring is a 2 px verdigris outline with 2 px offset, never a
  box-shadow-only hint.
- The reader scrolls independently; the list keeps its position. Opening a
  message marks it read after 800 ms of visibility, with an undo toast.
- Remote content is blocked with an inline banner naming the sender; the
  allow decision is per sender and revocable in settings.
- Below 1040 px the reader overlays the list and has a back button; below
  760 px the folder rail becomes a drawer. All actions remain keyboard
  reachable.

## Constraints and open decisions

- WCAG 2.2 AA, 200% text zoom, reduced motion.
- No icon font; authored 1.5 px-stroke SVG icons.
- Mail HTML keeps safe inline styles, class/id and legacy presentational markup;
  CSS is filtered for resource loads, fixed positioning and animation. The app
  must not flatten a sender's layout into its own typography.
- No component library; all tokens in `DESIGN.md`.
- Tauri IPC is the only data path; the UI never fetches the network.
- Open: whether "TMail" is the final name; whether signatures ship in MVP.
