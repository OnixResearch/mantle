# Fetched and patched source

This project fetches the published `crc64` 2.0.0 crate as a fixed-output tarball, applies a BLAKE3-fixed local patch with BusyBox, and checks the patched source tree. It also exposes an intentionally invalid patch derivation for fail-closed testing.

```sh
cd examples/projects/fetched-and-patched
mantle build .#upstream
mantle build
mantle build .#patched
mantle build .#checks.patch

# Expected to fail because the digest or patch context is wrong:
mantle build .#invalid-hash
mantle build .#invalid-patch
```

The first build requires network access. A repeated build with the same store and state reuses the verified fixed-output source.

The source hash proves the identity of the unpacked tree. It does not vouch for the upstream project or prove the patch is correct.
