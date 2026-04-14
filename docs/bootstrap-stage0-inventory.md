# Stage0 Bootstrap Inventory

This file tracks the stricter first-bootstrap view.

It is not the same thing as the default checked-in self-hosting proof.
Today the repo has two proof modes:
- default fixed-point proof: stage1 -> stage2 identity
- stricter `--non-nix-host` proof: same fixed point, with the proof runner
  `PATH` scrubbed of `nix-build`, `nix-store`, `nix-shell`, and `nix`

Neither mode yet proves a full-source bootstrap root or reproducible release
artifacts.

## Contract boundary

For the planned non-Nix-host first-bootstrap path, every stage0 dependency must
fit one of these buckets:

- **Host prerequisite**: installed or supplied by the operator before the long
  bootstrap starts.
- **Pinned fetched artifact**: downloaded from an explicit URL with a checked
  hash.
- **Crunch-built output**: produced later in the bootstrap chain.

Anything outside those buckets is a hidden trust edge.

## Current stage0 buckets

### Host prerequisites

| Item | Used by | Why it is trusted today | Notes |
|---|---|---|---|
| Linux host | `crunch bootstrap --fetch`, `crunch self-build`, `./scripts/prove-self-hosting.sh` | bwrap build service and current proof run on Linux only | non-Linux first bootstrap not in scope yet |
| Checkout source tree | `crunch self-build`, `./scripts/prove-self-hosting.sh` | stage0 still starts from the current repo checkout before crunch can rebuild itself | source staging now copies a fixed allowlist of top-level entries with Rust filesystem calls |
| Checked-in `vendor-deps/` tree + `.cargo/vendor-config.toml` | `crunch self-build`, `./scripts/prove-self-hosting.sh` | stage0 reuses the repo's checked vendored Cargo inputs instead of running host `cargo vendor` | staging currently checks that `vendor-deps/` exists and that `.cargo/vendor-config.toml` points at `vendor-deps`; it does not yet prove vendor-tree/Cargo.lock freshness |
| Host `bwrap` | `crunch self-build`, `./scripts/prove-self-hosting.sh` | first sandboxed build needs a working bubblewrap before crunch has built its own | later self-build stages switch to crunch-built `bwrap` |
| Static `SNIX_BUILD_SANDBOX_SHELL` | `crunch self-build`, `./scripts/prove-self-hosting.sh` | first sandbox stage needs a static shell that also exposes busybox applets | must be explicit; hidden realization is not acceptable for the stronger claim |
| Host Rust nightly + `cargo` + `clang` + `mold` + `pkg-config` + OpenSSL dev files | `./scripts/prove-self-hosting.sh` | stage0 helper builds the checkout test binary and prepares the proof env | proof-only prerequisites, not required by `crunch bootstrap --fetch` |
| About 4 GiB free in the selected proof scratch filesystem (`target/self-hosting-proof/work/` by default, or `CRUNCH_PROOF_SCRATCH_DIR`) | `./scripts/prove-self-hosting.sh` | proof stores two full bootstrap chains plus audit bundles | capacity requirement, not a trust root, but still a stage0 prerequisite |

### Pinned fetched artifacts

| Item | Used by | Why it is trusted today | Notes |
|---|---|---|---|
| musl.cc native tarball `https://musl.cc/x86_64-linux-musl-native.tgz` | `crunch bootstrap --fetch`, `crunch self-build`, `./scripts/prove-self-hosting.sh` | fetched by crunch with recursive hash `sha256-XpcI34j9YwAQj7qw4DpvXqT1CX00vHcUQbAk/do46jw=` and reduced to the normalized `musl-seed-toolchain` provider | still a trusted binary bootstrap seed, but smaller than the full raw tarball surface; inspect `<seed>/share/crunch-bootstrap/provider.json` for provenance and dropped payload |

### Crunch-built outputs

| Output | First producer | Later role |
|---|---|---|
| `make` | `bootstrap/make.ncl` | drives later source builds |
| `dash` | `bootstrap/dash.ncl` | provides POSIX shell for later stages |
| `binutils` | `bootstrap/binutils.ncl` | assembler, linker, archive tools for later stages |
| `musl` | `bootstrap/musl.ncl` | libc and sysroot for later stages |
| `gcc` | `bootstrap/gcc.ncl` | source-built compiler for later C and Rust work |
| `busybox` | `bootstrap/busybox.ncl` | static shell and applets for later sandbox stages |
| `bwrap` | `bootstrap/bwrap.ncl` | replaces host `bwrap` after first bootstrap stage |
| `rust` | `bootstrap/rust.ncl` | toolchain used to compile crunch itself |
| `crunch` | `crunch self-build` derivation | rebuilt stage1 and stage2 proof outputs |

## Host-convenience discovery only

Current helper scripts still probe NixOS- and Nix-specific locations such as
`/run/wrappers/bin`, `/run/current-system/sw/bin`, and `/nix/store/*`.
Those probes are host-convenience discovery only. They are not proof that the
first-bootstrap contract is already Nix-free.

`./scripts/prove-self-hosting.sh` no longer realizes `pkgsStatic.busybox`
through `nix-build`. If no installed static busybox is available, the helper
now fails fast and requires an explicit `SNIX_BUILD_SANDBOX_SHELL`.

## Current proof boundary

What the checked-in self-hosting proof demonstrates today:

- stage0 builds stage1 and records the staged `*-crunch-src` tree
- stage1 rebuilds stage2 from that same staged source tree
- stage1 and stage2 crunch binaries must match byte-for-byte
- the stage0 and stage2 crunch-built `busybox` and `bwrap` outputs must match
- in `--non-nix-host` mode, the stage0 command path completes with
  `nix-build`, `nix-store`, `nix-shell`, and `nix` absent from `PATH`
- successful proof bundles copy this inventory and record the resolved stage0
  prerequisite paths they used

What it does not demonstrate yet:

- a full-source bootstrap root smaller than the current reduced musl.cc-derived seed provider
- bit-for-bit reproducible release artifacts from independent rebuilders
- removal of remaining stage0 proof-helper host-tool edges such as the checkout-built Rust toolchain and host `bwrap`
