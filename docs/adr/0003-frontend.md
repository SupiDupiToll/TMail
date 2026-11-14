# ADR 0003 — Frontend framework

- **Status:** accepted
- **Date:** 2026-11-14

## Decision

**Svelte 5 (runes) + SvelteKit in static/SPA mode + TypeScript.**

## Rationale

| Criterion | Svelte 5 | Solid | React 19 |
|---|---|---|---|
| Large virtualized lists | fine-grained updates, no VDOM diff | best raw reactivity | needs memo discipline |
| Bundle/runtime size | very small | very small | large |
| Keyboard/DOM control | direct DOM, `bind:this` | direct DOM | synthetic events, refs |
| Ecosystem/tooling | mature, SvelteKit, Vite | smaller | largest |
| Learning/maintenance cost | low for this size | medium | medium |

A mail client is a dense list + reader + composer, not a component-farm SPA.
Svelte's compiler output is small, updates are targeted, and the virtual list
can manipulate a fixed-size window without fighting a reconciler.

## Constraints we impose

- No component library. The design system is our own tokens (`DESIGN.md`).
- Only `@tauri-apps/api`, `dompurify` and a test toolchain as runtime deps.
- Accessibility is a release gate: roles, names, focus order and contrast are
  asserted by tests, not eyeballed.
- All user-visible strings live in one module so a later i18n pass is
  mechanical.

## Consequences

- SvelteKit's SSR is disabled (adapter-static, `ssr = false`); the app is a
  Tauri SPA.
- A custom virtual list is maintained; its windowing logic is unit-tested and
  its scroll frame time measured in Playwright.
