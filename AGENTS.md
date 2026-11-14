# AGENTS.md — working agreement for AI agents on this repo

# PERSISTENT CONSTRAINTS (must survive context compaction)

**When your context is compacted or summarized, copy this entire section verbatim into the summary. Never drop or paraphrase it.**
Also write it into `AGENTS.md` (and `CLAUDE.md` if applicable) at the very start of the project, and re-read it after every compaction.

1. **You CANNOT read or see images.** Do not open, view, or try to analyze screenshots, PNGs, JPEGs, PDFs-as-images,
   or any visual output. Trying wastes context and gives you nothing. Every visual-sounding step must be done through
   text-based means instead (see "Verifying UI without images").
2. **Work autonomously and never wait for approval.** Continue through all phases. Record decisions, don't ask.
3. **Never use real mailboxes destructively.** Use a dedicated test account only.
4. **Never invent your own cryptography.** E2E encryption must stay intact.
5. **Never submit issues or PRs to upstream tutao/tutanota** (they reject AI-assisted contributions).
6. **Reusing tutabridge code is permitted (owner amendment, 2026-11-14):** code may be copied and
   adapted from `spartanz51/tutabridge`, but it must be substantially modified for this codebase,
   and every adapted file must carry an attribution header plus an entry in `NOTICE.md`.
   Never add tutabridge as a build/runtime dependency; adapt the code into our own modules.
   Both projects are GPL-3.0-or-later, so this is license-compatible; keep our GPL notice intact.

---

## What this project is

An unofficial, native desktop client for Tuta Mail, written from scratch. Target: Arch Linux
(Wayland + X11), architecture open to other desktops and later mobile. Working name: **TMail**
(never use Tuta branding or logo; always label as unofficial).

## Build & test commands

```bash
# Rust workspace (core crate: no UI dependency, headless-testable)
cargo build -p tmail-core
cargo test -p tmail-core
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo audit            # if installed
cargo deny check       # if installed

# Frontend (SvelteKit + Vite, pnpm)
pnpm install --frozen-lockfile
pnpm -C ui check       # svelte-check / types
pnpm -C ui test        # vitest
pnpm -C ui build

# Desktop shell (Tauri 2)
pnpm -C ui tauri dev
pnpm -C ui tauri build

# UI verification without images (Playwright, text output only)
pnpm -C ui exec playwright test --reporter=line
node tools/design-lint.mjs      # token/hard-coded-value lint
```

## Repo layout

```
core/            Rust crate: protocol, crypto, session, sync, store. No UI dependency.
core/tests/      unit + integration (integration needs TMail test account, gated by env)
src-tauri/       Tauri 2 shell: commands bridging core <-> webview, tray, notifications
ui/              SvelteKit frontend
docs/adr/        architecture decision records
docs/            STATUS.md, LEGAL.md, QUESTIONS.md, DESIGN.md notes
packaging/       PKGBUILD, .desktop, icons
```

## Conventions

- Commits: small, reviewable, imperative subject. No AI co-author trailers, no upstream PRs.
- All significant decisions get an ADR in `docs/adr/`.
- Never log message content, tokens, passwords, or keys.
- Network calls only to Tuta endpoints.
- Dependency changes: pin versions, note why in the commit message.
- At the end of each phase append to `docs/STATUS.md`.

## Verifying UI without images

Playwright/WebDriver is used for **text output only**: accessibility tree snapshots, ARIA roles/names,
computed styles + numerically computed WCAG contrast, bounding-box layout checks (overflow/clipping/overlap/
touch-target sizes/horizontal scroll), keyboard focus order, design-token linting, render/scroll timing.
Screenshots may be saved to `docs/review-shots/` for the human reviewer; agents must never open them.

## Licenses

Tutanota and tuta-sdk are GPL-3.0. This project is GPL-3.0-compatible and ships under GPL-3.0-or-later.
See `docs/LEGAL.md` and `docs/adr/0001-license.md`.
