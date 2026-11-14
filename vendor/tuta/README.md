# Vendored Tuta SDK — origin and local changes

This directory contains the official Tuta Rust SDK from
`https://github.com/tutao/tutanota`, pinned at tag
`tutanota-release-362.261020.0` (commit `f5e2d8628f17be558fe63130396ff625bbe69c77`),
plus local patches. Both upstream and our changes are **GPL-3.0**; the full
license text is in `LICENSE.txt`.

## Why vendored

The published Rust SDK is incomplete for a full mail client: it lacks
attachment/body blob reads, AEAD decryption, batch loads, interactive 2FA and
a few protocol fixes. We vendor a pinned copy so builds are reproducible and
so security-sensitive protocol fixes are reviewable in this repository.

## Local changes

The initial patch stack was adapted from `spartanz51/tutabridge` (GPL-3.0),
then reviewed and modified for this project. Each patch is tracked in
`patches/` with a README. Files touched by patches carry a short header where
practical; the authoritative list is `PATCHES.md`.
