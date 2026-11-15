# Local patches over the Tuta Rust SDK

Base: `tutanota-release-362.261020.0` (commit
`f5e2d8628f17be558fe63130396ff625bbe69c77`).

The patch stack was adapted from `spartanz51/tutabridge` (GPL-3.0) and is
reviewed here. Patches are applied to a pristine upstream checkout; the
resulting tree is what `vendor/tuta/tuta-sdk/` contains. To regenerate:

```bash
git clone --depth 1 --branch tutanota-release-362.261020.0 \
  https://github.com/tutao/tutanota /tmp/tutanota
cd /tmp/tutanota
for p in /path/to/TMail/vendor/tuta/patches/*.patch; do git apply "$p"; done
rsync -a --exclude uniffi-bindgen tuta-sdk/ /path/to/TMail/vendor/tuta/tuta-sdk/
```

| # | Patch | Purpose | Upstreamable? |
|---|---|---|---|
| 01 | optional empty | Treat an empty optional encrypted value as null (protocol fix already merged upstream after this base) | yes |
| 02 | normalize address | Trim/lowercase the address in `create_session`, like TS | yes |
| 03 | account KDF | Use the KDF version from the salt service; Bcrypt returns an error instead of panicking | yes |
| 04 | load multiple | Batch list-element loads, 100 per request | maybe |
| 05 | blob downloads | Instance-scoped read tokens, batched blob reads, retry/failover | internal |
| 06 | blob elements | Load `MailDetailsBlob` from its archive | internal |
| 07 | parse raw | Parse raw JSON entities through the official serializer | internal |
| 08 | interactive session | `initiate_session`, TOTP submit, pending poll | internal |
| 09 | owner session key | Decrypt a parsed entity with the owning instance's session key | maybe |
| 10 | AEAD session reads | AEAD v3 field decryption with session keys and field context | internal |
| 11 | AEAD group reads | AEAD v2 decryption with versioned group keys and KDF nonce | internal |
| 12 | bucket instance keys | Resolve attachment keys from a mail's bucket | internal |
| 13 | android logger | Honour the logging feature before installing Android logger | internal |
| 14 | send recipient keys | Expose encryption of a send bucket key for internal recipients through the existing public-key provider (RSA and TutaCrypt) | internal |
| 15 | custom domain detection | Ask the public-key service whether an address belongs to a Tuta account, including Tuta-hosted custom domains | internal |

We do **not** submit these to tutao/tutanota: upstream rejects AI-assisted
contributions. If a patch is independently rewritten and accepted upstream, it
is removed from this stack on the next version bump.

## Security notes

- Patches 10–12 touch decryption paths. They only wire primitives already
  present in `crypto-primitives` (AEAD, BLAKE3, AES) into the entity decoder;
  no new primitive is introduced.
- The AEAD implementation has upstream compatibility vectors in `test_data/`
  and runs in the SDK's own test suite.
- Every patch is one concern and has tests in the SDK tree. Run
  `cargo test --manifest-path vendor/tuta/Cargo.toml -p tuta-sdk` before
  accepting a base bump.
