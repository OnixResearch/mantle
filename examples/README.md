# mantle examples

Small gallery for `examples/`.

| File | What it shows |
|---|---|
| `hello.ncl` | smallest derivation |
| `mk-hello.ncl` | `mkDerivation` wrapper |
| `build-from-source.ncl` | multi-file C build with `make` |
| `multi-output.ncl` | split outputs (`out`, `dev`, `man`) |
| `fetch-file.ncl` | fixed-output single file fetch |
| `fetch-tarball.ncl` | fixed-output tarball fetch |
| `fetch-git.ncl` | fixed-output git checkout |
| `fetch-crate-crc64.ncl` | fetch published `crc64` crate source from crates.io |
| `build-crate-crc64.ncl` | build published `crc64` crate with mantle bootstrap Rust toolchain and shared reduced seed provider |
| `bootstrap-no-nix.ncl` | zero-Nix bootstrap with the shared reduced seed provider |
| `project/` | project-aware `mantle build .#name` example |

Useful commands:

```bash
# Evaluate only
mantle eval examples/fetch-crate-crc64.ncl

# Build into a writable temp store
mkdir -p /tmp/mantle-examples-store /tmp/mantle-examples-state

# Fetch a real crate source tarball
mantle build examples/fetch-crate-crc64.ncl \
  --store /tmp/mantle-examples-store \
  --state-dir /tmp/mantle-examples-state

# Build a real Rust crate from crates.io
mantle build examples/build-crate-crc64.ncl \
  --store /tmp/mantle-examples-store \
  --state-dir /tmp/mantle-examples-state \
  --no-substitute
```
