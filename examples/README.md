# crunch examples

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
| `build-crate-crc64.ncl` | build published `crc64` crate with crunch bootstrap Rust toolchain |
| `bootstrap-no-nix.ncl` | zero-Nix bootstrap with fetched musl toolchain |
| `project/` | project-aware `crunch build .#name` example |

Useful commands:

```bash
# Evaluate only
crunch eval examples/fetch-crate-crc64.ncl

# Build into a writable temp store
mkdir -p /tmp/crunch-examples-store /tmp/crunch-examples-state

# Fetch a real crate source tarball
crunch build examples/fetch-crate-crc64.ncl \
  --store /tmp/crunch-examples-store \
  --state-dir /tmp/crunch-examples-state

# Build a real Rust crate from crates.io
crunch build examples/build-crate-crc64.ncl \
  --store /tmp/crunch-examples-store \
  --state-dir /tmp/crunch-examples-state \
  --no-substitute
```
