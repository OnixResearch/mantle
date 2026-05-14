# Stage0 Bootstrap Inventory

This file tracks the stricter first-bootstrap view.

It is not the same thing as the default checked-in self-hosting proof.
Today the repo has three proof modes:
- default fixed-point proof: stage1 -> stage2 identity
- stricter `--non-nix-host` proof: same fixed point, with the proof runner
  `PATH` scrubbed of `nix-build`, `nix-store`, `nix-shell`, and `nix`
- `--no-host-tools --stage0-inventory <file>` proof: same fixed point, with
  stage0 launched through a concrete declared seed inventory and protected exec
  supervision while common host helpers are absent from the proof PATH

Neither mode by itself proves a full-source bootstrap root or reproducible
release artifacts. A later `mantle release create` bundle can package the proof
bundle, release binary, source archive containing tracked worktree files plus
verified vendored Cargo inputs, this inventory, and an optional canonical
reproducibility report for bundle-local checks. Only a verified report whose
artifact set matches the published release artifact set supports the
bit-for-bit reproducible release label; the packaged proof bundle alone still
does not widen the underlying bootstrap claim.

## Contract boundary

For the protected no-host-tools first-bootstrap path, the protected phase starts
at stage0 `mantle self-build --no-host-tools --stage0-inventory <file>` entry
before source staging, fetch, sandbox, or build work. It ends only after the
exact mantle-built `bootstrap/bwrap.ncl` and `bootstrap/busybox.ncl` outputs are
built, exported to disk, executable-checked, BLAKE3-hashed, selected as the
later-stage sandbox entry and shell, and recorded in `protected-exec-audit.json`.

Every protected-phase dependency must fit one of these buckets:

- **Host prerequisite**: installed or supplied by the operator before the long
  bootstrap starts.
- **Pinned fetched artifact**: downloaded from an explicit URL with a checked
  hash.
- **Mantle-built output**: produced later in the bootstrap chain.

Anything outside those buckets is a hidden trust edge.

Allowed direct Linux kernel interfaces in the protected phase are intentionally
narrow: seccomp user-notification setup/listener handling, interception of
`execve` and `execveat`, `/proc/<pid>/mem` reads needed to copy syscall path
bytes from the trapped task, file metadata and content reads needed for BLAKE3
digest verification, and normal process wait/exit reporting. Unsupported
seccomp setup, unsupported audit architecture, missing listener inheritance,
relative or unreadable exec paths, undeclared executables, digest mismatch, or
forbidden host helper names fail closed before execution.

The concrete no-host-tools inventory is a Nickel file using the schema in
`bootstrap/stage0-inventory.ncl`. The checked-in file is the typed empty schema;
real proof runs pass a generated or operator-supplied concrete inventory with
absolute paths and digests. `executable_entries` and `source_entries` share these
fields: `schema_version`, `id`, `role`, `phase`, `digest`,
`provenance_category`, `provenance`, `allowed_reason`, `owner`, and `required`.
Executable entries add `executable_path`; source entries add `urls` and
`extraction_rules`. Mantle-owned fingerprints use `digest.algorithm = "blake3"`.
Non-BLAKE3 hashes are accepted only for interoperability and require an explicit
`interoperability_reason`.

Allowed protected executable seed roles are exactly `sandbox-entry`,
`sandbox-shell`, `bootstrap-toolchain-tool`, and `bootstrap-build-tool`.
`sandbox-entry` and `sandbox-shell` are required and must be unique. Nix commands
are never valid seed roles. Protected source entries are allowlists: only listed
URLs may be fetched, extraction rules must be explicit, and downloaded bytes or
extracted trees must match the declared digest.

## Current stage0 buckets

### Host prerequisites

| Item | Used by | Why it is trusted today | Notes |
|---|---|---|---|
| Linux host | `mantle bootstrap --fetch`, `mantle self-build`, `./scripts/prove-self-hosting.sh` | bwrap build service and current proof run on Linux only | non-Linux first bootstrap not in scope yet |
| Checkout source tree | `mantle self-build`, `./scripts/prove-self-hosting.sh` | stage0 still starts from the current repo checkout before mantle can rebuild itself | source staging now copies a fixed allowlist of top-level entries with Rust filesystem calls |
| Source-tree `vendor-deps/` directory + `.cargo/vendor-config.toml` | `mantle self-build`, `./scripts/prove-self-hosting.sh` | stage0 reuses the repo's vendored Cargo inputs instead of running host `cargo vendor` | staging validates `Cargo.lock` registry/git packages against `vendor-deps/`, verifies Cargo's `.cargo-checksum.json` file digests, and checks Cargo-format SHA-256 package checksums where Cargo.lock provides them |
| Host `bwrap` | default `mantle self-build`, default/non-Nix `./scripts/prove-self-hosting.sh` | first sandboxed build needs a working bubblewrap before mantle has built its own | not accepted by no-host-tools mode unless it is explicitly declared as the `sandbox-entry` seed with a matching BLAKE3 digest |
| Static `SNIX_BUILD_SANDBOX_SHELL` | default `mantle self-build`, default/non-Nix `./scripts/prove-self-hosting.sh` | first sandbox stage needs a static shell that also exposes busybox applets | no-host-tools mode uses the declared `sandbox-shell` seed and fails closed on digest drift |
| Host Rust nightly + `cargo` + `clang` + `mold` + `pkg-config` + OpenSSL dev files | `./scripts/prove-self-hosting.sh` | stage0 helper builds the checkout test binary and prepares the proof env | proof-only prerequisites, not required by `mantle bootstrap --fetch` |
| About 4 GiB free in the selected proof scratch filesystem (`target/self-hosting-proof/work/` by default, or `CRUNCH_PROOF_SCRATCH_DIR`) | `./scripts/prove-self-hosting.sh` | proof stores two full bootstrap chains plus audit bundles | capacity requirement, not a trust root, but still a stage0 prerequisite |

### Pinned fetched artifacts

| Item | Used by | Why it is trusted today | Notes |
|---|---|---|---|
| musl.cc native tarball `https://musl.cc/x86_64-linux-musl-native.tgz` | `mantle bootstrap --fetch`, `mantle self-build`, `./scripts/prove-self-hosting.sh` | fetched by mantle with recursive hash `sha256-ZtQZncMvugqmS7OMvMtdhY5MMtem4KXBhamxWbHUDkY=` and reduced to the normalized `musl-seed-toolchain` provider | still a trusted binary bootstrap seed, but smaller than the full raw tarball surface; inspect `<seed>/share/mantle-bootstrap/provider.json` for provenance and dropped payload |

### Mantle-built outputs

| Output | First producer | Later role |
|---|---|---|
| `make` | `bootstrap/make.ncl` | drives later source builds |
| `dash` | `bootstrap/dash.ncl` | provides POSIX shell for later stages |
| `binutils` | `bootstrap/binutils.ncl` | assembler, linker, archive tools for later stages |
| `musl` | `bootstrap/musl.ncl` | libc and sysroot for later stages |
| `gcc` | `bootstrap/gcc.ncl` | source-built compiler for later C and Rust work |
| `busybox` | `bootstrap/busybox.ncl` | static shell and applets for later sandbox stages |
| `bwrap` | `bootstrap/bwrap.ncl` | replaces host `bwrap` after first bootstrap stage |
| `rust` | `bootstrap/rust.ncl` | toolchain used to compile mantle itself |
| `mantle` | `mantle self-build` derivation | rebuilt stage1 and stage2 proof outputs |

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

- stage0 builds stage1 and records the staged `*-mantle-src` tree
- stage1 rebuilds stage2 from that same staged source tree
- stage1 and stage2 mantle binaries must match byte-for-byte
- the stage0 and stage2 mantle-built `busybox` and `bwrap` outputs must match
- in `--non-nix-host` mode, the stage0 command path completes with
  `nix-build`, `nix-store`, `nix-shell`, and `nix` absent from `PATH`
- in `--no-host-tools` mode, stage0 receives `--no-host-tools
  --stage0-inventory <file>`, common host helpers (`git`, `tar`, `cp`, `sh`,
  `cargo`, `bwrap`, and Nix commands) are absent from the proof PATH, the
  declared `sandbox-entry` and `sandbox-shell` seeds are digest-checked, and the
  seccomp supervisor denies undeclared child exec attempts before execution
- successful proof bundles copy this inventory, durable stage1/stage2 binary
  artifacts, and the resolved stage0 prerequisite paths they used
- successful proof bundles also write `protected-exec-audit.json` with the
  stage0 inventory digest when no-host-tools mode is active, blocked host
  command set, fallback-event markers, protected transition records, and final
  result
- `mantle release create` can copy a full proof bundle plus this inventory into
  a release-evidence bundle, and `mantle release verify` can later re-check
  bundle-local integrity and proof-context using bundle-local contents only
- a prerequisite-only `./scripts/prove-self-hosting.sh --check` result is not
  release proof evidence and does not satisfy the release-bundle proof slot

### StageX-class no-quorum release profile

The `stagex-verified-no-quorum` profile is the strictest release-verification
target. It requires:

- Provider kind `stagex-lineage` with `hex0-seed` seed class (max 4096 bytes)
- Complete StageX lineage metadata in the self-build proof: seed, lineage
  manifest, stage graph, normalized provider, staged source, stage1/stage2
  binary, bootstrap tool, and protected-exec audit digests
- All Mantle-owned digests lowercase BLAKE3 hex
- No undeclared host compiler, build tool, archive tool, Nix command, or
  legacy provider execution observed by the protected-exec supervisor
- A verified byte-identical reproducibility report
- Quorum is intentionally `not_evaluated`: the profile never emits
  `quorum-satisfied` as a success label

This profile rejects seed-assisted, source-root, self-proof-only, and
prerequisite-only evidence. External witness agreement alone does not
satisfy it. Verify with:
`mantle release verify <bundle-dir> --require-stagex-no-quorum`

Remaining environmental assumptions:

- Stage0 Rust compiler is a fetched stable binary, not hex0-bootstrapped
- Linux kernel and bwrap/FUSE sandbox runtime are trusted host components
- Network transport for bootstrap fetches is trusted

## Trust-root separation: full-source chain impact

When the live-bootstrap source chain (`bootstrap/seed-full.ncl`) passes
validation and replaces the legacy musl.cc seed, the trust-root picture
changes:

### Eliminated by full-source chain

| Former trust root | Eliminated by | Notes |
|---|---|---|
| musl.cc native tarball (`x86_64-linux-musl-native.tgz`) | `bootstrap/seed-full.ncl` normalizing gcc-10 + musl-1.2.5 + binutils-2.41 outputs built from source through the live-bootstrap ladder | The ~56-stage chain from `stage0-posix` through `seed-full` replaces the single fetched binary provider |
| Implicit trust in musl.cc binary provenance | Source-pin audit (`scripts/check-bootstrap-source-pins.rs`) covering all 84 fetch blocks | Each source is individually pinned with URL + SRI hash |

### Remaining trust roots (not eliminated by this chain)

| Trust root | Why it remains | Path to elimination |
|---|---|---|
| Host Linux kernel | bwrap sandbox and FUSE mounts require kernel interfaces | Out of scope for application-level bootstrap |
| Host `bwrap` (first stage only) | First sandbox needs a working bubblewrap before mantle builds its own | `--no-host-tools` mode with declared seed narrows this to a single digest-checked executable |
| Static `SNIX_BUILD_SANDBOX_SHELL` (first stage only) | First sandbox stage needs a static shell | Same as bwrap: declared seed with digest |
| Stage0 Rust compiler | Fetched stable binary, not hex0-bootstrapped | Requires a Rust-from-C bootstrap chain (future work) |
| Checkout source tree | stage0 starts from the current repo checkout | Source staging copies a fixed allowlist and validates vendored inputs |
| Host Rust/clang/mold/pkg-config/OpenSSL (proof-only) | Proof helper builds the test binary | Not required by `mantle bootstrap --fetch` or `mantle self-build` |
| Network transport for bootstrap fetches | Downloads use HTTPS but transport is trusted | Content-addressed hashes verify integrity post-fetch |

### Status promotion rules

Bootstrap maturity status reads proof fields from `SelfBuildReport`:
- `provider_mode` must be `source-root` or `stagex-lineage` for full-source claims
- `stagex_metadata` must include lineage manifest digest, stage graph digest,
  normalized provider digest, and all mantle-built tool digests
- Stage transcript index must cover all stages from `stage0-posix` through `seed-full`
- Placeholder, deferred, or archived partial-scaffolding evidence is always rejected
- Missing proof fields or transcripts block promotion and name the missing item

What it does not demonstrate yet:

- a full-source bootstrap root smaller than the current reduced musl.cc-derived seed provider
- independent rebuild agreement by itself; that status is derived later by
  `mantle attest release-verify --json` from accepted witness sidecars,
  verifier-local policy, revocations, and trusted keys
- more than packaged integrity, proof-context evidence, any separately verified
  reproducibility report, and any separately satisfied independent-agreement
  report; release verification keeps these evidence classes separate so one
  label does not silently imply another
- removal of remaining default proof-helper host-tool edges such as the checkout-built Rust toolchain and host `bwrap`; no-host-tools mode narrows the stage0 execution boundary but still requires operator-supplied seed artifacts and does not make them source-built
