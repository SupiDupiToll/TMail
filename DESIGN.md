---
name: TMail
description: A calm, dense, keyboard-first mailroom for Tuta Mail
colors:
  paper: "#f5f6f3"
  panel: "#fbfcf9"
  panel-2: "#eef1ec"
  ink: "#1a211d"
  muted: "#56625b"
  faint: "#626e65"
  border: "#d5dbd4"
  verdigris: "#1d6b5f"
  verdigris-soft: "#dcebe6"
  brass: "#7d5310"
  brass-soft: "#f3e6cb"
  danger: "#9b2f29"
  border-strong: "#b8c2b9"
  dark-paper: "#121714"
  dark-panel: "#171d19"
  dark-ink: "#e6ebe4"
  dark-verdigris: "#6bceb9"
  dark-brass: "#e0b25a"
  dark-muted: "#a7b3a9"
  dark-border: "#2b342d"
  dark-border-strong: "#3d493f"
  dark-link: "#8adcc9"
  code-wash: "rgba(127, 127, 127, 0.08)"
typography:
  ui:
    fontFamily: "Atkinson Hyperlegible Next, Noto Sans, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 420
    lineHeight: 1.45
  reading:
    fontFamily: "Source Serif 4, Georgia, Times New Roman, serif"
    fontSize: "16px"
    fontWeight: 430
    lineHeight: 1.62
  title:
    fontFamily: "Atkinson Hyperlegible Next, Noto Sans, system-ui, sans-serif"
    fontSize: "22px"
    fontWeight: 720
    lineHeight: 1.25
  label:
    fontFamily: "Atkinson Hyperlegible Next, Noto Sans, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 650
    lineHeight: 1.4
  micro:
    fontFamily: "Atkinson Hyperlegible Next, Noto Sans, system-ui, sans-serif"
    fontSize: "9px"
    fontWeight: 700
    lineHeight: 1.1
  micro-lg:
    fontFamily: "Atkinson Hyperlegible Next, Noto Sans, system-ui, sans-serif"
    fontSize: "10px"
    fontWeight: 750
    lineHeight: 1.2
  meta:
    fontFamily: "Atkinson Hyperlegible Next, Noto Sans, system-ui, sans-serif"
    fontSize: "10.5px"
    fontWeight: 600
    lineHeight: 1.2
  small:
    fontFamily: "Atkinson Hyperlegible Next, Noto Sans, system-ui, sans-serif"
    fontSize: "11px"
    fontWeight: 650
    lineHeight: 1.3
  brand:
    fontFamily: "Atkinson Hyperlegible Next, Noto Sans, system-ui, sans-serif"
    fontSize: "1.3rem"
    fontWeight: 760
    lineHeight: 1.15
  brand-lg:
    fontFamily: "Atkinson Hyperlegible Next, Noto Sans, system-ui, sans-serif"
    fontSize: "1.2rem"
    fontWeight: 760
    lineHeight: 1.2
  title-xl:
    fontFamily: "Atkinson Hyperlegible Next, Noto Sans, system-ui, sans-serif"
    fontSize: "1.55rem"
    fontWeight: 740
    lineHeight: 1.15
    letterSpacing: "-0.025em"
  display-min:
    fontFamily: "Atkinson Hyperlegible Next, Noto Sans, system-ui, sans-serif"
    fontSize: "1.8rem"
    fontWeight: 720
    lineHeight: 1.1
    letterSpacing: "-0.028em"
  display:
    fontFamily: "Atkinson Hyperlegible Next, Noto Sans, system-ui, sans-serif"
    fontSize: "2.55rem"
    fontWeight: 720
    lineHeight: 1.1
    letterSpacing: "-0.028em"
  mail-h1:
    fontFamily: "Source Serif 4, Georgia, serif"
    fontSize: "1.5rem"
    fontWeight: 700
    lineHeight: 1.25
  mail-h2:
    fontFamily: "Source Serif 4, Georgia, serif"
    fontSize: "1.25rem"
    fontWeight: 700
    lineHeight: 1.25
  mail-h3:
    fontFamily: "Source Serif 4, Georgia, serif"
    fontSize: "1.08rem"
    fontWeight: 700
    lineHeight: 1.25
rounded:
  hairline: "2px"
  tiny: "3px"
  mail: "5px"
  xs: "6px"
  code: "4px"
  sm: "10px"
  md: "14px"
  lg: "20px"
  pill: "999px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "16px"
  xl: "24px"
  xxl: "32px"
components:
  compose:
    backgroundColor: "{colors.verdigris}"
    textColor: "#ffffff"
    rounded: "{rounded.sm}"
    padding: "0 13px"
    height: "40px"
  compose-hover:
    backgroundColor: "#155448"
    textColor: "#ffffff"
  folder-active:
    backgroundColor: "{colors.verdigris-soft}"
    textColor: "{colors.verdigris}"
    rounded: "{rounded.sm}"
    height: "36px"
  input:
    backgroundColor: "{colors.panel}"
    textColor: "{colors.ink}"
    rounded: "{rounded.sm}"
    padding: "0 12px"
    height: "40px"
  button-primary:
    backgroundColor: "{colors.verdigris}"
    textColor: "#ffffff"
    rounded: "{rounded.sm}"
    padding: "0 12px"
    height: "34px"
  button-ghost:
    backgroundColor: "transparent"
    textColor: "{colors.ink}"
    rounded: "{rounded.sm}"
    padding: "0 12px"
    height: "34px"
  chip:
    backgroundColor: "{colors.panel-2}"
    textColor: "{colors.muted}"
    rounded: "{rounded.pill}"
    padding: "0 7px"
    height: "20px"
---

# Design System: TMail

## Overview

**Creative North Star: "The Quiet Workspace."**

TMail should feel like a calm desktop workspace: warm paper, ink text, one
verdigris working accent, and rounded surfaces that keep the density from
feeling like a spreadsheet. It is a tool, not a poster. The interface earns
trust by being consistent, quiet and exact: every row aligns, every state has
a name, every action has a key. Density is the point; calm is the result.

The visual identity is carried by four things only: **type** (a legible UI face
and a serif for reading), **palette** (paper, ink, verdigris, brass), **rounded
surfaces** (a 58 px row with a 14 px card radius), and one **signature move** —
the deterministic sender avatar: a soft, tinted initial disc that makes a
mailbox recognisable at a glance without a rainbow of labels. Navigation and
controls stay familiar: a sidebar, a virtualised list, a reader, a composer.
Timestamps are deliberately absent from the list; the sender and subject carry
the scan, and the reader shows the full date when it matters.

**Key characteristics:**

- Paper ground, ink text; no glass, no gradients, no decorative shadows.
- One accent does the work: select, focus, primary action.
- Brass is reserved for unread time and the day gutter.
- Reading is serif; operating is sans.
- Motion is short, exponential and optional.

## Colors

The palette is restrained: two neutral families (paper, ink) plus verdigris and
brass. Colour is state, not decoration.

### Primary

- **Verdigris** (`#1d6b5f`, dark `#6bceb9`): selection, focus, primary buttons,
  links, active folder. On dark, the lighter tint keeps 4.5:1 on the panel.
- **Verdigris soft** (`#dcebe6`, dark `#1c3a33`): selected rows, active folder
  background, informational chips.

### Secondary

- **Brass** (`#7d5310`, dark `#e0b25a`): attachment/encryption flags on unread
  rows and synthetic preview warnings. Never used for primary actions.
- **Brass soft** (`#f3e6cb`, dark `#3a2f1a`): warning surfaces.

### Semantic

- **Danger** (`#9b2f29`, dark `#ff9a90`) and **danger soft**: destructive
  actions, errors, phishing warnings.
- **Warn** (`#6f470a`, dark `#e8b965`) and **warn soft**: degraded/offline
  states and confirmations.

### Rules

- Text on paper is `ink`; secondary text is `muted` (≥4.5:1) and `faint` is
  only for non-essential metadata, still ≥4.5:1.
- Never use gray on a tinted surface; secondary text is tinted from the same
  hue.
- Remote images are blocked before colour ever touches the reader.

## Typography

**Atkinson Hyperlegible Next** is the operating voice: high legibility at 11–14
px, distinct numerals and generous apertures. **Source Serif 4** is the reading
voice: message bodies, quoted text, the composer. A monospace stack appears only
for protocol identifiers and code. Mail HTML keeps the sender's own typography
where it is safe; the reading face is the fallback, not a cage.

### Hierarchy

- **Page title** (22 px / 720 / 1.25): folder name in the list.
- **Subject** (1.55 rem / 740 / 1.15): the reader title.
- **Row sender** (14 px / 600 read, 760 unread): the scan target.
- **Row subject** (13 px / 400 read, 560 unread): one line, ellipsis.
- **Body** (16 px / 430 / 1.62, measure 68ch): message HTML.
- **Label** (12 px / 650): field labels and section labels in settings.
- **Meta** (10.5–11 px): counts, flags, status line.

Rules: tabular numerals for times and counts; never letter-space below -0.03em;
no display face in the UI; body measure stops at 68ch.

## Layout

Three panes: a 248 px mailbox rail, a flexible list (`clamp(320px, 34vw, 460px)`),
and the reader. The list virtualises at a fixed row height (58/70/82 px by
density) inside an independently scrolling viewport. The reader header stays
fixed; the message body scrolls.

- 48 px top strip height shared by the account row and list header.
- The mailbox rail and message list are drag-resizable; the rail collapses to a
  72 px icon strip. Widths and collapse state persist locally.
- 4 px spacing scale; 8 px between controls, 12–16 px inside panels, 24–28 px
  around the reader.
- Breakpoints: ≤1120 px the reader overlays the list with a back action;
  ≤820 px the mailbox rail becomes a drawer and the list takes the width.
- Rows are rounded cards inset by 2 px; there is no timestamp gutter, no day
  separator and no column of times. At ≤380 px container width the attachment
  and encryption flags hide first.

## Elevation & Depth

Flat by default. Borders (1 px `border`) separate surfaces; elevation appears
only for floating layers (composer, palette, menus, toasts) as one soft,
offset shadow (`0 10px 30px rgba(24,36,29,.14)`). No inner glows, no coloured
halos, no hard offset shadows.

## Shapes

Soft radii, calm surfaces: 6 px for keys and small chips, 10 px for
inputs/buttons and folder rows, 14 px for mail rows and menus, 20 px for
dialogs and the login card, pill only for counts and status chips. Borders are
1 px; focus is a 2 px outline with 2 px offset (or -2 px inset on scroll
containers). Icons are authored 1.5 px-stroke SVGs; no icon font, no emoji.

## Components

- **Compose**: full-width verdigris button in the rail, white text, `c` keycap
  at the right. It is the only always-visible primary action.
- **Folder row**: 31 px, icon + name + count; active uses verdigris soft with a
  2 px left rail; unread counts are pills, muted until active.
- **Mail row**: tinted avatar, sender, subject, flags. Unread = ink + 760 weight
  + a small verdigris dot. Selected = `selected-bg` with a 1 px `selected-border`.
  Hover is a single tonal step. No timestamp is shown.
- **Reader**: subject, sender/recipient meta, action toolbar, blocked-content
  notice, sandboxed body, attachment list with in-app image/text preview. Mail
  always renders on a light canvas for contrast, independent of the app theme;
  no card chrome around the body.
- **Composer**: a rounded, resizable sheet anchored bottom-right on desktop
  (full-screen on mobile), with an expand toggle; a From selector for aliases,
  bare inputs with bottom rules, serif body, schedule-send clock, attachment
  preview, and plain-text mode. The encryption state is automatic: a lock chip
  for Tuta recipients, an honest "not encrypted" chip for external recipients.
- **Palette**: 640 px, top 12vh, search first, grouped commands, message results
  with snippets, keyboard hints in the footer.
- **Settings**: right-side sheet with appearance, sending, notifications,
  privacy, account and about groups; segmented controls and no nested cards.
  Server-side rules/vacation/templates are explicitly out of scope for now.
- **Toasts**: bottom-right stack, one icon, one sentence, optional undo.

## Do's and Don'ts

- **Do** let the list carry the density; keep the reader calm.
- **Do** use the keyboard hints in context; every action must be discoverable.
- **Do** keep unread signal to a small verdigris dot plus weight, not a second
  colour.
- **Do** let safe presentation CSS through in mail; do not flatten every message
  into the app's own typography.
- **Do** show the server/keyring truth in errors and settings.
- **Don't** add a second accent, gradient text, glass panels or decorative
  shadows.
- **Don't** use cards for rows, settings groups or the reader.
- **Don't** put a logo wall, illustration or marketing copy inside the app.
- **Don't** animate entrances per row; one short selection transition is enough.
- **Don't** add timestamps or day separators back into the list.
- **Don't** use the time gutter, a ledger rail or dense 44 px rows.
- **Don't** use colour alone for state: pair it with a label or an icon.
