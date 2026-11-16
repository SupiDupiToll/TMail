# Releasing TMail

Releases are built by GitHub Actions from a version tag. Nothing is published
without a tag, and every release starts as a **draft** for review.

## Create a release

1. Make sure the version is bumped in `Cargo.toml`, `src-tauri/tauri.conf.json`
   and `ui/package.json` (the Tauri action uses the tag for the release name,
   so keep them in sync).
2. Run the local gate:
   ```bash
   cargo test -p tmail-core
   cargo clippy --workspace --all-targets --no-deps -- -D warnings
   pnpm -C ui check
   pnpm -C ui test
   pnpm -C ui exec playwright test --reporter=line
   ```
3. Commit and push, then tag:
   ```bash
   git tag v0.2.4
   git push origin v0.2.4
   ```
4. Open the **Actions → Release** run. It builds in parallel:
   - Ubuntu 22.04: `.deb`, `.rpm`, `.AppImage` (x86_64, glibc 2.35+)
   - Windows: NSIS `.exe` and `.msi`
   - macOS: universal `.dmg`
5. Review the generated **draft release**, verify the artifacts and install
   notes, then publish it.

The workflow is in [`.github/workflows/release.yml`](../.github/workflows/release.yml).

## Artifacts and runtime requirements

| Artifact | Runtime |
|---|---|
| `.deb` | Debian/Ubuntu with `libwebkit2gtk-4.1-0` and `libgtk-3-0` |
| `.rpm` | Fedora/openSUSE with `webkit2gtk4.1` and `gtk3` |
| `.AppImage` | Any x86_64 Linux with WebKitGTK 4.1; `libfuse2` on older Ubuntu |
| `.exe` / `.msi` | Windows 10/11 with WebView2 |
| `.dmg` | macOS; universal binary |

The AppImage does not bundle WebKitGTK. It uses the system engine.

**Screenshots are not release assets.** Keep review screenshots in
`docs/review-shots/` for the README and documentation; do not upload them or a
screenshot ZIP to a GitHub release.

## Signing and in-app updates

The workflow produces **unsigned OS installers** and says so in the release
body, but update artifacts are signed with the Tauri updater key:

- Public key: in `src-tauri/tauri.conf.json` under `plugins.updater.pubkey`.
- Private key: generated locally at `~/.tauri/tmail-updater.key` (mode 0600);
  keep a secure backup and never commit it.
- Password: `~/.tauri/tmail-updater.password`.
- GitHub repository secrets:
  - `TAURI_SIGNING_PRIVATE_KEY`
  - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

The workflow builds `latest.json` and the `.sig` files, uploads them to the
release, and TMail checks:

```text
https://github.com/SupiDupiToll/TMail/releases/latest/download/latest.json
```

Only **published** releases are visible to the updater. Draft releases are not.
The updater is enabled for **AppImage, Windows and macOS**; `.deb`, `.rpm` and
distribution packages are updated by the package manager. The settings sheet
detects this and disables in-app updates for package-managed Linux installs.

OS code signing (Windows Authenticode, macOS notarization) remains separate:

- **Windows:** add a code-signing certificate and `signtool` configuration.
- **macOS:** add `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`,
  `APPLE_SIGNING_IDENTITY` and notarization credentials.

Do not enable OS signing without testing the resulting artifact on a clean
machine.

## Local packaging check

On Linux, verify the configuration without producing a full bundle:

```bash
pnpm tauri build --debug --no-bundle
```

To build the host bundles locally:

```bash
pnpm tauri build
ls target/release/bundle/
```

If RPM or AppImage tooling is missing, install the distribution packages for
`rpm`/`linuxdeploy` or rely on the GitHub workflow, which has the toolchain.
If `linuxdeploy` complains about foreign `libgdk_pixbuf`/`libcroco` paths, unset
`LD_LIBRARY_PATH` before bundling (`env -u LD_LIBRARY_PATH pnpm tauri build`).

## Version bumps and the SDK

TMail vendors and patches the official Tuta SDK. A version bump may also
require updating the SDK base and patch stack; see
[`docs/adr/0001-protocol-and-crypto.md`](adr/0001-protocol-and-crypto.md) and
[`vendor/tuta/PATCHES.md`](../vendor/tuta/PATCHES.md). Run the smoke checklist
in [`docs/SMOKE.md`](SMOKE.md) against the dedicated test account before a
public release.
