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
   git tag v0.2.0
   git push origin v0.2.0
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

## Signing

The workflow currently produces **unsigned** builds and says so in the release
body. This is deliberate: an unsigned build must not look signed.

To add signing later:

- **Tauri updater/signing:** create `TAURI_SIGNING_PRIVATE_KEY` and
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` repository secrets and add updater
  artifacts to the Tauri config.
- **Windows:** add a code-signing certificate and `signtool` configuration.
- **macOS:** add `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`,
  `APPLE_SIGNING_IDENTITY` and notarization credentials.

Do not enable signing without testing the resulting artifact on a clean
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
