# mantle

Nickel build system on the Nix store protocol.

mantle evaluates `.ncl` files describing derivations, constructs store
paths using BLAKE3 hashing, and executes builds in a bwrap sandbox. No
Nix evaluator in the loop.

## Quick Start

```bash
# Build mantle (requires Rust nightly, clang, mold, openssl-dev)
cargo build --release

# Generate a seed toolchain from your Nix store
mantle bootstrap -o seed.ncl

# Write a derivation
cat > hello.ncl << 'EOF'
let mantle = import "lib.ncl" in
{
  name = "hello",
  builder = "/bin/sh",
  args = ["-c", "echo 'Hello, mantle!' > $out"],
} | mantle.Derivation
EOF

# Evaluate (print JSON, no build)
mantle eval hello.ncl

# Build (requires Linux + bwrap)
mantle build hello.ncl

# Machine-readable build summary for audit tooling
mantle --json build hello.ncl
```

`mantle --json build` writes a stable `crunch-build-report-v1` JSON
object to stdout. It includes per-root outcomes, cache hits, failure
records, output paths, hermeticity audit data, and per-output
`artifact_attestation` references (`logical_path` + sidecar `path`) so
tests and operators can assert on structured data instead of scraping
human text. The optional `log_file` fields are only present when the
corresponding log was actually written to disk.

For successful cache hits, each output may also include a `substitution`
object. `mode` is `delta` when mantle completed the hit through delta
reuse and `full` when it completed through ordinary full-artifact fetch.
`transferred_bytes` reports bytes fetched from the cache, `reused_bytes`
reports receiver-local reuse, and `fallback_reason` only appears when
mantle started delta negotiation but finished through full fetch.

Human output reports same facts inline on cached outputs, for example:

```text
…/result-path (cached, substitution=delta, transferred_bytes=12, reused_bytes=34)
…/result-path (cached, substitution=full, transferred_bytes=55, reused_bytes=0, fallback_reason=stream_application_failed)
```

## Operator diagnostics

Use `mantle doctor` before a fresh build host or self-build run:

```bash
# Default profile: build
mantle doctor

# Self-build prerequisites
mantle doctor --profile self-build
```

Current doctor profiles:

- `build` — default. Checks `bwrap`, sandbox shell resolution,
  `fusermount3`, writable state dir, and writable output store dir.
- `self-build` — everything in `build`, plus nightly `cargo +nightly`
  and `rustc +nightly` visibility.

Doctor is read-only: it does not start builds, download substitutes, or mutate
store state.

For bootstrap runtime-validation work, `mantle bootstrap validate` combines the
build doctor preflight, a no-substitute build, captured stdout/stderr logs, a
coarse host-path leakage scan, and OpenSpec-ready JSON/Markdown evidence:

```bash
mantle --json --store .mantle-drain/make-store \
  --state-dir .mantle-drain/make-state \
  bootstrap validate bootstrap/make-tcc.ncl \
  --evidence-dir openspec/changes/live-part-make-3-82-runtime-validation/evidence \
  --resume
```

Use `mantle build --plan` to preview what mantle will do per root without
building:

```bash
mantle build --plan hello.ncl
mantle --json build --plan hello.ncl
```

Planned action labels:

- `cached` — every output is already accepted from local PathInfo + castore
- `substitute` — at least one output is missing locally but remote narinfo says
  a cache hit is available, and no local build is needed
- `build` — no acceptable cache hit exists, but local build preflight passed
- `preflight-error` — mantle cannot perform the local build path for that root
  (for example missing output store dir, missing `bwrap`, or missing sandbox
  shell)

Recommended operator workflow:

1. Run `mantle doctor` for the intended workflow profile.
2. Run `mantle build --plan ...` to see which roots are cached,
   substitutable, buildable, or blocked by preflight.
3. Run the real build only after doctor and plan look sane.
4. If a build fails, read the structured failure summary first, then open the
   saved log path when one is present.

JSON build failures in `crunch-build-report-v1` now use a typed failure
envelope under `failed[]` with these fields:

- `root` — operator-facing root label
- `drv_key` — derivation store key
- `phase` — `preflight` or `build`
- `error_class` — typed class such as `preflight`,
  `fixed-output-hash-mismatch`, or `sandbox`
- `message` — original failure text
- `saved_log_path` — only present when mantle wrote a saved failure log

Human failure output prints the same facts in the same order before any deeper
error text.

Output lands in `/nix/store/<hash>-hello` by default (the `--store`
default). Derivation hashes are computed under the `/mantle/store`
logical prefix (`--store-prefix`). Use `--store /tmp/mystore` to
write outputs elsewhere, or `--nix-compat` to switch the logical
prefix to `/nix/store` for interop testing. See
[Store Paths and Prefixes](#store-paths-and-prefixes) for details.

Pass `--strict-hermetic` to `mantle build`, `mantle self-build`, or
`mantle bootstrap validate` when degraded hermetic behavior should fail instead
of warn. Pass the explicit Nix-style `--impure` escape hatch only for diagnostic
or compatibility work that intentionally permits ambient host dependencies;
`--impure` is mutually exclusive with `--strict-hermetic`. Human and JSON
reports both surface `hermeticity_mode` and any
`hermeticity_audit_events` recorded during the run, including the typed
`impure-mode-selected` event for impure runs.

## Operator workflows

Current operator loops:

- Plan and build: `mantle doctor`, `mantle build --plan`, `mantle build`,
  `mantle self-build --strict-hermetic`
- Develop and run: `mantle shell`, `mantle develop`, `mantle run`
- Inspect and publish: `mantle attest`, `mantle release`

Short examples:

```bash
# Shells and package execution from the compatibility-named crunch.ncl package root
mantle shell --command env
mantle develop            # deprecated alias for shell
mantle run .#hello -- --help
mantle run ./tool.ncl --bin tool -- --version

# Attestation and release evidence entry points
mantle attest show /nix/store/<hash>-hello
mantle release verify target/release-evidence/<release-id>
mantle release attest target/release-evidence/<release-id>
mantle attest release-verify target/release-verification/<release-id> --trusted-public-key <name:base64>
```

For the full command path, examples, and sidecar rules, see
[`docs/operator-workflows.md`](docs/operator-workflows.md).

## Validation tiers

Use the checked-in toolchain from [`rust-toolchain.toml`](rust-toolchain.toml)
for all validation commands.

**Ordinary first-party gate**

Run the checked-in wrapper from the repo root:

```bash
./scripts/check-first-party-quality.sh
```

That wrapper runs three ordinary edit-time checks in order:

```bash
cargo fmt --check \
  -p mantle \
  -p crunch-attestation \
  -p crunch-build \
  -p crunch-delta \
  -p crunch-eval \
  -p crunch-glue \
  -p crunch-pipeline \
  -p crunch-project \
  -p crunch-shell \
  -p crunch-store
./scripts/check-first-party-clippy.sh
cargo test --workspace --lib --tests
```

The root `-p mantle` rustfmt leg covers the root package's `src/`,
`examples/`, and `tests/`, including `tests/benchmark_harness.rs`.
The strict clippy helper excludes vendored workspace members
`fuse-backend-rs`, `nix-compat`, `nix-compat-derive`, `snix-build`,
`snix-castore`, `snix-store`, and `snix-tracing` so first-party warnings fail
cleanly.

**Tigerstyle lane**

Run the repo-pinned Tiger Style consumer check when you want the structural lint
pass:

```bash
./scripts/check-first-party-tigerstyle.sh
```

That wrapper delegates to the flake-pinned `cargo-tigerstyle` runner:

```bash
nix run .#tigerstyle -- check
```

Default scope comes from `[workspace.metadata.tigerstyle]` in `Cargo.toml`, so
vendored workspace members stay out of the lint pass. Workspace-specific lint
rollout config lives in `dylint.toml`.

**Heavyweight rails**

Keep these heavier checks separate from the ordinary gate:

```bash
cargo test -p crunch-pipeline --test integration_build \
  pipeline_determinism_probe_ -- --ignored --nocapture
./scripts/prove-self-hosting.sh --check
./scripts/check-release-determinism-quality.sh
./scripts/check-release-nix-witness-quality.sh
nix build .#checks.x86_64-linux.release-determinism-quality --no-link -L
nix build .#checks.x86_64-linux.release-nix-witness-quality --no-link -L
```

The release determinism quality rail runs the generated deterministic proof
smoke and validates its receipt/log BLAKE3 in one checked-in entry point. The
flake check is the Nix/CI-callable heavy gate for the same underlying generated
proof regression. The Nix witness rail exercises the bounded `mantle release
nix-witness` CLI path and is exposed as both
`packages.<system>.release-nix-witness-quality` and
`checks.<system>.release-nix-witness-quality`, keeping it opt-in rather than part
of ordinary developer builds. `bootstrap parity-report` consumes the checked compact
self-build descriptor in `bootstrap/evidence/real-self-build-proof-parity.json`
and surfaces bounded `crunch.self-build` proof details without marking Guix or
StageX parity complete. The determinism probe is an ignored integration rail,
not part of every edit. `./scripts/prove-self-hosting.sh --check` is only
self-hosting preflight. The full ignored proof run stays heavier:

```bash
./scripts/prove-self-hosting.sh
```

**Vendored maintenance lane**

Workspace-wide vendored upkeep stays separate from the first-party gate.
Do not treat vendored clippy debt as an ordinary edit blocker for tracked mantle
code.

If you bypass the checked-in wrappers and run direct compile-heavy `cargo`
commands, use the same PATH / `PKG_CONFIG_PATH` / `SNIX_BUILD_SANDBOX_SHELL`
prerequisites described in the self-hosting section below, and put `TMPDIR` and
`CARGO_TARGET_DIR` on disk-backed scratch before treating failures as code
failures.

## Examples

The repo ships runnable examples under [`examples/`](examples/):

- [`examples/fetch-crate-crc64.ncl`](examples/fetch-crate-crc64.ncl) — fetch a real crates.io source tarball (`crc64` 2.0.0)
- [`examples/build-crate-crc64.ncl`](examples/build-crate-crc64.ncl) — build that real crate with mantle's bootstrap Rust toolchain and shared reduced seed provider
- [`examples/build-from-source.ncl`](examples/build-from-source.ncl) — build a multi-file C project with `make`
- [`examples/bootstrap-no-nix.ncl`](examples/bootstrap-no-nix.ncl) — compile C with the shared reduced bootstrap seed provider
- [`examples/project/`](examples/project/) — project-aware `mantle build .#name` layout
- [`examples/README.md`](examples/README.md) — short index of the full example set

## Benchmark suite

Checked-in benchmark entry points live under [`examples/`](examples/):

- `cargo run --example benchmark_eval_smoke -- --bundle-out target/benchmarks/eval-smoke.json --repeat-count 2`
- `cargo run --example benchmark_suite -- --bundle-out target/benchmarks/suite.json --repeat-count 2`
- `cargo run --example benchmark_compare -- --baseline target/benchmarks/baseline.json --fresh target/benchmarks/suite.json --absolute-threshold-ns 1000 --percent-threshold 5`

The smoke path keeps local checks cheap. The full matrix covers evaluation,
conversion, substitution planning, and build-graph preparation using checked-in
fixtures only. The compare entry point matches workloads by stable name and
highlights the largest regressions or wins. See
[`docs/benchmark-suite.md`](docs/benchmark-suite.md).

## Fetchers

Download files, tarballs, and git repos as fixed-output derivations:

```nickel
let mantle = import "lib.ncl" in

# Single file
mantle.fetchurl {
  url = "https://example.com/foo.tar.gz",
  hash = "sha256-...",
}

# Tarball (auto-decompress + unpack + strip top-level dir)
mantle.fetchTarball {
  url = "https://github.com/user/repo/archive/v1.0.tar.gz",
  hash = "sha256-...",
}

# Git repository at a specific rev
mantle.fetchGit {
  url = "https://github.com/user/repo.git",
  rev = "abc123...",
  hash = "sha256-...",
}
```

Hash mismatches show the correct hash. Use `--fix` to auto-update:

```bash
mantle build --fix hello.ncl
```

Supported hash algorithms: sha256, sha512, sha1, md5, blake3.
Supported compression: gzip, xz, bzip2, zstd.

## Content-Addressed Derivations

The default `addressing_mode` is `'content-addressed`: output paths are
computed from the build output content (BLAKE3 hash of the NAR), not from
the derivation inputs. Identical outputs get identical paths.

```nickel
# CA is the default — no opt-in needed
{ name = "my-tool", builder = "...", ... } | mantle.Derivation

# Opt into input-addressed if needed
{ name = "my-tool", addressing_mode = 'input-addressed, ... } | mantle.Derivation
```

CA self-reference rewriting works: if a build output embeds its own
store path (RPATHs, shebangs), the provisional path is rewritten to
the final content-addressed path. Multi-output CA derivations use
two-pass rewriting to handle cross-output references.

## Multi-Output Derivations

Split a package into separate outputs (binary, headers, man pages):

```nickel
{
  name = "hello",
  outputs = ["out", "dev", "man"],
  builder = "/bin/bash",
  args = ["-c", m%"
    for output in $outputs; do
      mkdir -p "${!output}"
    done
    mkdir -p $out/bin && echo 'hello' > $out/bin/hello
    mkdir -p $dev/include && echo '#define V 1' > $dev/include/hello.h
    mkdir -p $man/share/man/man1 && echo '.TH HELLO 1' > $man/share/man/man1/hello.1
  "%],
  ...
} | mantle.Derivation
```

Each output gets a distinct store path. The builder receives `$outputs`
(space-separated list) and individual path variables (`$out`, `$dev`,
`$man`). Works with both input-addressed and content-addressed modes.

To depend on a single output of a multi-output package, use
`mantle.select`:

```nickel
let mantle = import "lib.ncl" in
# Only mount the `dev` output of libfoo (not `out` or `lib`)
{
  name = "consumer",
  inputs = [
    mantle.select libfoo "dev",
    mantle.select libfoo "lib",
  ],
  ...
} | mantle.Derivation
```

Multiple selections from the same dependency are coalesced into a single
`input_derivations` entry. A bare derivation in `inputs` mounts all its
outputs.

## Architecture

```
.ncl file
    │
    ▼
crunch-eval (nickel-lang)     Evaluate Nickel → record
    │
    ▼
crunch-glue (nix-compat)      Record → nix_compat::Derivation + BLAKE3 store paths
    │
    ▼  mpsc channel (EvalMessage per root)
    │
crunch-build (goal + worker)  Lazy goal scheduler → bwrap sandbox → PathInfo
    │                         (or builtin fetcher for fetchurl/fetchTarball/fetchGit)
    ▼
/nix/store/<hash>-<name>      Output on disk (--store default)
```

The scheduler is a lazy goal-based system (not an eager DAG). Each
derivation becomes a `Goal` with a state machine
(Pending→Waiting→Ready→Building→Done). The `Worker` creates goals on
demand, deduplicates by store path, and dispatches concurrent builds
via `JoinSet` + `Semaphore`. Roots arrive over an mpsc channel, so the
Worker can start building leaf deps while later roots are still being
processed.

The goal system is extensible: substitution goals, dynamic derivations
(build outputs that are .drv files), and remote build dispatch can be
added without restructuring the scheduler.

## Crate Layout

| Crate | Role |
|---|---|
| `mantle` | CLI binary — build, eval, bootstrap, store, log, project management, self-build |
| `crunch-eval` | Nickel evaluation, stdlib embedding |
| `crunch-glue` | `CrunchDerivation` → `nix_compat::Derivation`, ConversionCache |
| `crunch-build` | `Derivation` → `BuildRequest`, goal scheduler, build orchestration, fetchers |
| `crunch-pipeline` | eval → deserialize → convert → build wiring |
| `crunch-project` | Project manifest, lockfile, refresh, stale detection, upgrade |
| `crunch-store` | Store export, closure resolution, StoreHandle, signing, query |
| `vendor/nix-compat` | Store paths, ATerm, NAR (BLAKE3-modified) |
| `vendor/snix-build` | `BuildService` trait, bwrap sandbox |
| `vendor/snix-castore` | Blob and directory content-addressed storage |
| `vendor/snix-store` | `PathInfoService`, NAR calculation |

## Nickel Stdlib

The stdlib (`lib/`) defines the derivation schema and fetch helpers:

```nickel
let mantle = import "lib.ncl" in
{
  name = "myapp",              # | Name (validated)
  builder = "/bin/sh",         # | String
  system = 'x86_64-linux,      # | System enum (default)
  args = ["-c", "..."],        # | Array String (default [])
  outputs = ["out"],           # | Array String (default ["out"])
  env = { CC = "gcc" },        # | { _ : String } (default {})
  inputs = [                   # | Array Input (default [])
    "/nix/store/...-bash",     #   StorePath string → source input
    { name = "lib", ... },     #   Derivation record → built first
  ],
  fixed_output = {             # | FixedOutput | optional
    hash = "sha256-...",
    algo = 'sha256,            #   sha256 | sha512 | sha1 | md5 | blake3
    mode = 'flat,              #   flat | recursive
  },
  addressing_mode              # | default = 'content-addressed
    = 'content-addressed,      #   content-addressed | input-addressed
  sandbox                      # | default = 'native
    = 'native,                 #   'native today; 'oci and 'wasm are reserved for future backends
} | mantle.Derivation
```

Contracts catch errors at eval time:
- Missing required fields (`name`, `builder`)
- Invalid store paths, derivation names
- Wrong enum variants

## How It Differs From Nix

| | Nix | mantle |
|---|---|---|
| Config language | Nix | Nickel |
| Derivation hash | SHA-256 | BLAKE3 |
| Default addressing | Input-addressed | Content-addressed |
| Default store prefix | `/nix/store` | `/mantle/store` (`--nix-compat` for `/nix/store`; explicit `/crunch/store` remains legacy-compatible) |
| Inputs | String context (implicit) | Explicit `inputs` field |
| Hash algorithms | sha256/sha512/sha1/md5 | + blake3 (first-class) |
| Sandbox | Nix sandbox | bwrap (via snix-build) |
| Closure resolution | `nix-store -qR` | Own PathInfo walk (no Nix required) |
| Serialization | protobuf/gRPC | postcard |

## Store Paths and Prefixes

mantle separates two concepts:

- **Logical prefix** (`--store-prefix`, default `/mantle/store`): used for
  derivation hash computation, ATerm serialization, and output path names.
  Different prefixes produce different derivation hashes.
- **Physical directory** (`--store`, default `/nix/store`): where mantle
  writes build outputs on disk. Source inputs (seed packages) are always
  read from `/nix/store`.

```bash
# Default: logical paths under /mantle/store, outputs written to /nix/store
mantle build hello.ncl

# Write outputs to a custom directory
mantle build --store /tmp/mystore hello.ncl

# Nix-compatible hashes (for interop testing)
mantle build --nix-compat hello.ncl
```

`--nix-compat` is shorthand for `--store-prefix=/nix/store`.

## Signing and Trust

mantle signs PathInfo entries with ed25519 keys. If no `--signing-key` is
provided, an auto-generated key is created at
`$CRUNCH_CONFIG_DIR/signing-key` (or `$state_dir/signing-key` when
`CRUNCH_CONFIG_DIR` is unset; default state dir is
`~/.local/state/mantle`).

```bash
# Build with an explicit signing key
mantle build --signing-key ./my-key hello.ncl

# Verify signatures against trusted public keys
mantle store verify --trusted-public-keys "mykey-1:base64pubkey..."

# Sign all unsigned PathInfo entries (migration from unsigned stores)
mantle store sign --all

# Accept unsigned PathInfo on cache hits (escape hatch)
mantle build --trust-unsigned hello.ncl
```

By default, cached PathInfo must be signed by a trusted key. The default
trust set includes `cache.nixos.org-1`. Use `--trusted-public-keys` to
override.

### Retained roots and manual GC

Successful top-level outputs from `mantle build`, `mantle self-build`, and
`mantle bootstrap --fetch` are retained automatically as GC roots. Use:

```bash
mantle store roots
mantle store pin /nix/store/<hash>-name
mantle store unpin /nix/store/<hash>-name
mantle store gc --dry-run
mantle store gc
```

`mantle store gc --dry-run` reports the retained-root count, candidate path
count, reclaimable bytes, and each candidate logical store path without
mutating state. Real GC sweeps unreachable exported outputs, PathInfo rows,
attestation sidecars, and unreachable castore content. GC fails closed if a
retained root's reachability metadata is missing or unreadable.

Current safety bounds are internal and fail closed rather than partially
collect: exported-output byte walks stop at depth 128 or 100,000 visited
entries, and filesystem/blob scans stop at 200,000 entries.

## Binary Cache Sharing

Share build results between machines using the Nix binary cache directory
layout (`.narinfo` + `.nar` files). Push exports from the local store;
pull imports into it.

```bash
# Push all signed paths to a cache directory
mantle store push --all --to /srv/cache

# Push specific paths
mantle store push --to /srv/cache /mantle/store/<hash>-hello

# Include unsigned entries (normally skipped)
mantle store push --all --to /srv/cache --trust-unsigned

# Pull all paths from a cache directory
mantle store pull --all --from /srv/cache

# Pull specific paths from a cache directory
mantle store pull --from /srv/cache /mantle/store/<hash>-hello

# Pull specific paths from an HTTP cache
mantle store pull --from https://cache.example.com /mantle/store/<hash>-hello

# Pull from HTTP with explicit trust (signature verification)
mantle store pull --from https://cache.example.com \
  --trusted-public-keys "builder-1:base64pubkey..." \
  /mantle/store/<hash>-hello

# Accept unsigned narinfos
mantle store pull --all --from /srv/cache --trust-unsigned
```

Push writes `nix-cache-info` into the target directory if absent, so the
result is directly servable by `nix-serve`, `harmonia`, nginx, or S3
sync. Paths already present in the target are skipped.

Pull verifies narinfo signatures against the configured trusted keys
(same trust set as `mantle build --substituters`). Paths that fail
signature verification, have a store-directory prefix mismatch, or whose
NAR content does not match the declared hash are skipped with a warning.
HTTP pull is explicit-path only: `--all` works for directory caches, but
HTTP/HTTPS sources require one or more logical store paths on the command
line.

Both commands acquire the store mutation lock, so they are safe alongside
concurrent builds on the same state directory.

Typical CI workflow:

```bash
# Builder machine: build and publish
mantle build my-package.ncl
mantle store push --all --to /shared/cache

# Consumer machine: import pre-built results
mantle store pull --all --from /shared/cache \
  --trusted-public-keys "ci-builder-1:base64pubkey..."
mantle build my-package.ncl   # cache hit, no rebuild
```

## Bootstrap

mantle needs some trusted inputs to build anything. The repo tracks four
different bootstrap paths, and they do not all prove the same thing.

### Bootstrap maturity levels

- **Seed-assisted bootstrap**: mantle can start from declared external seeds
  and build later bootstrap stages.
- **Self-hosting proof**: a mantle-built `mantle` can rebuild `mantle` from
  the same staged source tree.
- **Packaged release evidence**: a release bundle carries the exact binary,
  source archive with tracked worktree files plus verified vendored Cargo inputs,
  proof bundle, and prerequisite inventory so
  later verification can check bundle-local integrity and proof linkage.
- **Full-source bootstrap**: the trusted root has been reduced to small,
  explicitly audited source or bootstrap seeds.
- **Reproducible release evidence**: independent rebuilds produce the same
  final artifact and can be compared or signed.

Today the repo has seed-assisted bootstrap, a checked-in self-hosting proof,
and packaged release evidence. It does not yet claim a full-source bootstrap
or bit-for-bit reproducible release outputs.

### StageX-class no-quorum target

The repo defines a `stagex-verified-no-quorum` release profile that requires
a fully auditable seed-to-binary chain without trusting host compilers.
It is stricter than seed-assisted or source-root evidence:

- The bootstrap provider must be `stagex-lineage` — the only accepted seed
  class is `hex0-seed` (a small hand-audited byte seed, max 4096 bytes).
- The self-build proof must include StageX lineage metadata: seed/lineage
  manifest/stage-graph/provider/staged-source/stage1/stage2/bootstrap-tool
  digests plus a protected-exec audit digest, all lowercase BLAKE3 hex.
- Protected execution must not observe undeclared host compiler, build tool,
  archive tool, Nix command, or legacy provider executables.
- A verified reproducibility report must be present and matched.
- Quorum policy is intentionally deferred: `quorum_status` is always
  `not_evaluated`. The profile never emits `quorum-satisfied`.

Seed-assisted bootstrap, source-root evidence, and self-proof-only evidence
do not satisfy this profile. External witness agreement alone does not
satisfy it either — StageX-class lineage proof is required.

Remaining environmental assumptions not yet eliminated:

- The Rust compiler used for stage0 is a fetched stable binary, not
  bootstrapped from the hex0 seed.
- The Linux kernel and FUSE/bwrap sandbox runtime are trusted host components.
- Network transport for bootstrap artifact fetches is trusted.

Verify with: `mantle release verify <bundle-dir> --require-stagex-no-quorum`

### Trust inventory by entry point

For the stricter stage0 view, see [`docs/bootstrap-stage0-inventory.md`](docs/bootstrap-stage0-inventory.md).
That inventory groups first-bootstrap dependencies into host prerequisites,
pinned fetched artifacts, mantle-built outputs, and host-convenience probes.

#### `mantle bootstrap`

- **Current claim**: seed-assisted bootstrap from an existing Nix installation.
- **Trusted inputs today**: the host's Nix tooling and the `/nix/store` paths
  returned by `nix-build` or `nix build`.
- **Host prerequisites**: Nix installed, selected packages available in
  `nixpkgs`, and a machine where those store paths remain reachable.
- **Evidence today**: `mantle bootstrap -o seed.ncl ...` writes a
  `StorePath`-validated seed file, and later builds mount the declared source
  closures in the sandbox.
- **Not yet proven**: the imported Nix seed is not reduced to an auditable
  minimal root inside mantle itself.

#### `mantle bootstrap --fetch`

- **Current claim**: seed-assisted bootstrap without Nix.
- **Trusted inputs today**: the pinned musl.cc native tarball declared in
  `bootstrap/seed.ncl`, plus the checked-in reducer that turns it into the
  normalized `musl-seed-toolchain` provider.
- **Host prerequisites**: Linux, network access to fetch the raw tarball,
  and a working mantle binary with enough local disk for fetched outputs.
- **Evidence today**: the fetch path evaluates `bootstrap/seed.ncl`, builds
  the reduced provider, writes `seed.ncl`, and installs provider provenance at
  `<seed>/share/crunch-bootstrap/provider.json`.
- **Not yet proven**: the reduced provider is still derived from a trusted
  binary tarball, not a source-built root.

#### `mantle self-build`

- **Current claim**: seed-assisted self-build from the current checkout.
- **Trusted inputs today**: the current source tree, the source-tree
  `vendor-deps/` directory and `.cargo/vendor-config.toml` validated against
  `Cargo.lock` and Cargo's `.cargo-checksum.json` metadata, the reduced
  `musl-seed-toolchain` provider derived from the pinned musl.cc tarball, and
  the host sandbox entry points used for the first bootstrap stage.
- **Host prerequisites**: `bwrap` and a static sandbox shell for the first
  bootstrap stage.
- **Evidence today**: `mantle self-build --store /tmp/mantle-store -j 4 --no-substitute`
  builds the bootstrap chain `seed -> make -> dash -> binutils -> musl -> gcc -> busybox -> bwrap -> rust -> mantle`.
- **Not yet proven**: this first build still relies on host tooling and the
  reduced seed provider, so it is not a host-tool-free or full-source bootstrap.

#### `./scripts/prove-self-hosting.sh`

- **Current claim**: checked-in self-hosting proof.
- **Trusted inputs today**: a checkout-built stage0 `mantle` binary, the same
  reduced seed provider and host-tool assumptions as `mantle self-build`, and
  the staged source tree recorded by stage0.
- **Host prerequisites**: Linux, the repo's nightly Rust toolchain, `clang`,
  `mold`, `pkg-config`, OpenSSL development files, `bwrap`, `git`, `cargo`,
  a static sandbox shell, and about 4 GiB free in the proof scratch
  filesystem (`target/self-hosting-proof/work/` by default, or
  `CRUNCH_PROOF_SCRATCH_DIR`).
- **Evidence today**: the helper prepares the environment and runs
  `cargo test -p mantle --test self_hosting -- --ignored --nocapture`, which
  drives the stage0 -> stage1 -> stage2 proof path.
- **Not yet proven**: the proof still stops short of a full-source bootstrap
  root, independent rebuild agreement, or reproducible release artifacts.

### Bootstrappable Builds checklist

| Best-practice item | Status | Evidence today | Gap that remains |
|---|---|---|---|
| Provide an alternative way to build the build system | Yes | `cargo build --release` builds the checkout binary, and `mantle self-build` provides the in-repo bootstrap path | The bootstrap path still starts from host tooling and a reduced fetched seed provider |
| Label where bootstrap binaries or tarballs came from | Partial | `mantle bootstrap` names the Nix-backed path, and `mantle bootstrap --fetch` reuses the checked-in `bootstrap/seed.ncl` metadata and writes provider provenance into `provider.json` inside the fetched store path | The repo still trusts that reduced provider; it does not yet derive it from a smaller source bootstrap |
| Reproduce bootstrap binaries from source end-to-end | Not yet | The repo can build `make`, `dash`, `binutils`, `musl`, `gcc`, `busybox`, `bwrap`, `rust`, and `mantle` from the reduced seed provider | The reduced provider itself still comes from a trusted musl.cc binary tarball |
| Automate bootstrap traceability or self-hosting checks | Partial | `./scripts/prove-self-hosting.sh` runs the checked-in stage0 -> stage1 -> stage2 proof, `--check` verifies prerequisites first, successful runs write a proof bundle with `manifest.json`, `summary.txt`, and per-stage logs under `target/self-hosting-proof/`, and `mantle release create` can package that bundle with the release binary and a source archive containing tracked worktree files plus verified vendored Cargo inputs for later bundle-local integrity and proof-context checks with `mantle release verify` | The proof and release bundle still stop short of independent reproducibility evidence for release outputs |

## Self-Build

mantle can rebuild itself from source once the stage0 prerequisites are already present:

```bash
mantle self-build --store /tmp/mantle-store -j 4 --no-substitute
```

Pass `--strict-hermetic` when degraded hermeticity should fail instead of
warn during the self-build path.

This builds the full bootstrap chain (`seed -> make -> dash ->
binutils -> musl -> gcc -> busybox -> bwrap -> rust -> mantle`) inside a
bwrap sandbox. Output is a statically linked musl binary.

This is still seed-assisted bootstrap. It does not yet prove that the first
bootstrap works on a host with Nix commands absent from `PATH`.

First bootstrap requires the source tree with `vendor-deps/` and
`.cargo/vendor-config.toml`; `mantle self-build` validates the staged vendored
inputs against `Cargo.lock` and Cargo checksum metadata before building them.
It also requires `bwrap` and a static sandbox shell on `PATH`.
After the first self-build, the mantle-built `bwrap` and `busybox` are used
for subsequent builds.

### Proving self-hosting

To verify the fixed-point self-hosting claim, run the checked-in helper from
the repo root:

```bash
./scripts/prove-self-hosting.sh
```

The helper sets the nightly Rust toolchain, `CC`, linker or tool lookup,
`pkg-config` or OpenSSL lookup, and `SNIX_BUILD_SANDBOX_SHELL` before it
invokes the canonical ignored proof test:

```bash
cargo test -p mantle --test self_hosting -- --ignored --nocapture
```

Use `./scripts/prove-self-hosting.sh --check` to validate prerequisites,
including temporary-disk headroom, and print the proof command without
starting the full build.

Use `./scripts/prove-self-hosting.sh --non-nix-host` for the stricter proof
mode. That mode keeps the same stage0 -> stage1 -> stage2 fixed-point check,
but it also scrubs `nix-build`, `nix-store`, `nix-shell`, and `nix` from the
proof runner `PATH` before it invokes `cargo test`, so hidden Nix-command
fallbacks fail loudly.

Use `./scripts/prove-self-hosting.sh --no-host-tools --stage0-inventory <file>`
for the host-tool-free stage0 proof mode. The inventory is a concrete Nickel
file matching `bootstrap/stage0-inventory.ncl`; it must predeclare absolute
operator-supplied seed artifacts with BLAKE3 digests. Required executable seed
roles are `sandbox-entry` and `sandbox-shell`; additional allowed seed roles are
`bootstrap-toolchain-tool` and `bootstrap-build-tool`. In this mode the helper
captures an absolute Cargo path before proof PATH poisoning, then removes
`git`, `tar`, `cp`, `sh`, `cargo`, `bwrap`, and Nix commands from the proof
PATH used by stage0. The stage0 `self-build` command passes `--no-host-tools
--stage0-inventory <file>` and installs the protected exec supervisor before
sandbox startup.

The protected stage begins at no-host-tools stage0 `mantle self-build` entry and
ends only after mantle-built `bootstrap/bwrap.ncl` and `bootstrap/busybox.ncl`
outputs have been built, exported, verified, selected, and recorded in the proof
audit. Linux direct kernel interfaces allowed in that protected phase are the
seccomp user-notification listener, `execve`/`execveat` interception,
`/proc/<pid>/mem` reads for syscall path bytes, file metadata and content reads
needed to compute BLAKE3 digests, and normal process wait/exit reporting. Any
undeclared executable, digest mismatch, unsupported supervisor setup, relative
or unreadable exec path, or forbidden host helper fails closed before execution.

This helper still does not prove a full-source bootstrap root or reproducible
release artifacts. It proves either a fixed-point self-hosting rebuild, the same
rebuild under the stricter non-Nix-host command-availability contract, or the
host-tool-free stage0 boundary against explicitly declared seed artifacts.

Successful runs write a proof bundle to `target/self-hosting-proof/run-...`
and refresh `target/self-hosting-proof/latest` to point at that bundle. Pass
`--bundle-dir <dir>` to choose a different destination. Relative bundle paths
are anchored to the repo root before the helper exports them to the proof test
or updates `latest`.

The helper does not inherit ambient `TMPDIR` or `CARGO_TARGET_DIR`. It picks a
proof scratch root from `CRUNCH_PROOF_SCRATCH_DIR` when set, else from the
repo-local default `target/self-hosting-proof/work/`, then rewrites both env
vars under that root before it launches `cargo test`. `--check` reports the
selected scratch root, provenance, and free-space preflight result.

Host prerequisites:
- Linux
- `rustup` with the repo's `nightly` toolchain installed
- `clang`, `mold`, `pkg-config`, and OpenSSL development files visible to `pkg-config`
- `bwrap`, `git`, and `cargo`
- a static `SNIX_BUILD_SANDBOX_SHELL`
- about 4 GiB free in the selected proof scratch filesystem (`target/self-hosting-proof/work/` by default, or `CRUNCH_PROOF_SCRATCH_DIR`)

If `pkg-config --exists openssl` does not work in your current shell, export
`CRUNCH_PROOF_OPENSSL_PKGCONFIG=/path/to/openssl/lib/pkgconfig` before running
the helper.

If no installed static busybox is discoverable, set `SNIX_BUILD_SANDBOX_SHELL`
explicitly before running the helper. The helper no longer realizes one through
hidden `nix-build` fallback.

If you bypass the helper for a compile-heavy non-proof Cargo command, start
from the prerequisite PATH / `PKG_CONFIG_PATH` / `SNIX_BUILD_SANDBOX_SHELL`
setup documented in `AGENTS.md`, then move both temp files and Cargo artifacts
onto disk-backed scratch yourself:

```bash
export TMPDIR="$PWD/target/manual-work/tmp"
export CARGO_TARGET_DIR="$PWD/target/manual-work/cargo-target"
mkdir -p "$TMPDIR" "$CARGO_TARGET_DIR"
cargo test -p mantle --test self_hosting -- --list
```

That example is scratch guidance only. For self-hosting evidence, keep using
`./scripts/prove-self-hosting.sh`.

This runs two stages: the checkout binary builds stage1, then the stage1
binary rebuilds mantle as stage2 from the same staged source tree. The test
asserts that stage2 selected mantle-built `bwrap` and `busybox`, not host
fallbacks. Expect about 30 minutes and about 4 GiB free in the selected proof
scratch filesystem.

Each proof bundle contains:
- `manifest.json` — stable machine-readable digests for the checkout, stage1,
  and stage2 binaries, plus the stage2 `bwrap` and `busybox` paths and digests,
  store or state locations, copied stage metadata, proof mode, and recorded
  stage0 prerequisite paths
- `summary.txt` — a short human-readable digest summary, including protected
  execution audit digest and protected transition summary when present
- `binaries/stage1-mantle` and `binaries/stage2-mantle` — durable copies of
  the fixed-point binaries, so release packaging and witness rebuilds do not
  depend on temporary proof scratch paths surviving after the proof exits
- `stage0/` and `stage2/` — copied `meta.json`, `stdout.txt`, `stderr.txt`, and
  `diagnostics.txt` files from the stage audit bundles
- `stage0-prerequisites/inventory.md` — copied stage0 trust inventory used by
  the proof bundle
- `protected-exec-audit.json` — machine-readable protected execution audit with
  no-host-tools mode, optional stage0 inventory digest, blocked host command
  set, fallback markers, mantle-built sandbox transition records, and final
  result

The proof does demonstrate a fixed point: stage1 and stage2 must match
byte-for-byte, and the stage0 and stage2 mantle-built `busybox` and `bwrap`
outputs must match too. In `--non-nix-host` mode it also proves that the stage0
command path completed with `nix-build`, `nix-store`, `nix-shell`, and `nix`
absent from `PATH`. In `--no-host-tools` mode it additionally proves that
stage0 used declared seed sandbox artifacts under protected exec supervision and
that common host helper names were absent from the stage0 PATH. It does not yet
demonstrate bit-for-bit reproducible release artifacts or a full-source
bootstrap root.

### Release evidence bundles

A successful proof bundle can be packaged as release evidence:

```bash
mantle release create \
  --release-id mantle-<version> \
  --binary target/self-hosting-proof/run-.../binaries/stage2-mantle \
  --proof-bundle target/self-hosting-proof/run-...
```

Repeat `--binary` when one release bundle should carry multiple executables.
If a separate byte-for-byte reproduction run has already produced a canonical
report, package it explicitly:

```bash
mantle release create \
  --release-id mantle-<version> \
  --binary target/self-hosting-proof/run-.../binaries/stage2-mantle \
  --proof-bundle target/self-hosting-proof/run-... \
  --reproducibility-report target/release-evidence/<release-id>/reproducibility/reproducibility-report.json
```

That command builds a staged-source tarball from the current tracked worktree
allowlist, the checked-in witness workflow driver (`scripts/prove-self-hosting.sh`),
the stage0 inventory document it consumes, the self-hosting test target/support
files needed by that driver, and verified vendored Cargo inputs, then copies the
release binary, proof bundle, and prerequisite inventory into a
new release-evidence bundle under `target/release-evidence/<release-id>/` by
default. The top-level `manifest.json` records BLAKE3 digests for the source
archive, bundled binary or binaries, proof-bundle directory, prerequisite
inventory, and the proof-linkage facts copied from the full self-hosting proof.

To produce the canonical byte-for-byte reproducibility report, run an explicitly
supported rebuild recipe into a clean output directory. The current supported
recipe identity is `mantle-release-reproducibility-v1`; unknown
`--workflow-version` values fail closed before execution.

```bash
mantle release reproduce target/release-evidence/<release-id> \
  --rebuild-output-dir /tmp/mantle-rebuild-out \
  --rebuild-command ./scripts/rebuild-release-artifacts.sh
```

For a stronger deterministic-build proof attempt, ask `release reproduce` to run
at least two additional clean proof runs. Each proof run is executed through a
recorded proof sandbox envelope instead of a direct host process. The envelope
uses `bwrap` when available, denies network by default, binds the release bundle
and rebuild recipe read-only, and exposes only the per-run output tree plus a
fresh store directory (`MANTLE_DETERMINISTIC_PROOF_STORE_DIR`) as writable proof
state. The proof directory must be separate from and not nested with the main
rebuild output directory. If the sandbox executor is unavailable or unsupported,
deterministic proof mode fails closed before writing a proof receipt.

```bash
mantle release reproduce target/release-evidence/<release-id> \
  --rebuild-output-dir /tmp/mantle-rebuild-out \
  --rebuild-command ./scripts/rebuild-release-artifacts.sh \
  --deterministic-proof-runs 2 \
  --deterministic-proof-dir /tmp/mantle-deterministic-proof
```

For a repo-maintained smoke rail that exercises the generated-proof path and
writes an operator receipt, run:

```bash
cargo -Zscript scripts/release-determinism-smoke.rs
```

The smoke rail runs the checked-in CLI regression that creates release evidence,
generates a `mantle-deterministic-proof-receipt-v1` receipt from two clean proof
stores, and verifies it with `mantle release verify
--require-deterministic-release`. It writes `receipt.json` and `test.log` under
`target/release-determinism-smoke/latest/` by default. Validate the rail receipt
schema and log BLAKE3 with:

```bash
cargo -Zscript scripts/check-release-determinism-smoke-receipt.rs \
  target/release-determinism-smoke/latest/receipt.json
```

The receipt is smoke evidence for the rail and does not replace a
release-specific proof receipt.

For a real release-specific rail, run the self-hosting proof, package the
stage2 mantle binary, produce two clean deterministic proof rebuilds under real
`bwrap`, and verify the generated proof in one command:

```bash
./scripts/prove-real-release-determinism.sh
```

Use `--proof-bundle target/self-hosting-proof/run-...` to reuse an existing full
self-hosting proof bundle. The script writes the release bundle,
`deterministic-build-proof.json`, sandbox evidence, verify receipt, and portable
`<release-id>-determinism-summary.{json,md}` under `target/release-evidence/`.
It validates the proof outputs with the checked-in real proof receipt checker
before writing the summary. Re-check a saved run with:

```bash
cargo -Zscript scripts/check-real-release-determinism-receipt.rs \
  target/release-evidence/<release-id>
```

Pass `--proof-dir` or `--verify-receipt` if the proof or verify JSON was moved
away from the default sibling paths. To write portable JSON and Markdown summary
artifacts for archival or review, run:

```bash
cargo -Zscript scripts/summarize-real-release-determinism.rs \
  target/release-evidence/<release-id>
```

That first runs the real proof receipt checker, then writes
`target/release-evidence/<release-id>-determinism-summary.{json,md}` with the
release id, provider kind, source/vendor BLAKE3, artifact digest set,
proof/sandbox/verify BLAKE3 evidence, `self-rebuild-match`, `eligible`, and
explicit non-claims. The successful claim is bounded to the packaged stage2
artifact rebuilding twice from the recorded inputs under recorded
`mantle-proof-sandbox-v1:*` profiles with matching BLAKE3 digest sets; it does
not claim full bootstrap reproducibility.

The report records a closed proof-class ladder:

- `bundle-consistent`: bundle-local digest/linkage checks only.
- `self-proof-valid`: the self-proof and report are valid, but the rebuild did
  not match or did not supply enough material for a rebuild-match claim.
- `self-rebuild-match`: a clean local rebuild produced the same named artifact
  set, sizes, and BLAKE3 digests.
- `external-witness-match`: an accepted external witness rebuild matched the
  published BLAKE3 artifact set.
- `policy-satisfied`: the configured witness/quorum policy was satisfied.

Rebuild-match classes require `comparison_verdict=matched`, matching per-artifact
BLAKE3 comparisons, clean rebuild store/output identities, replayable workflow
identity, environment assumptions, and BLAKE3 digests for the evidence artifacts
used to make the claim. A failed or missing comparison is represented as a failed
report with a weaker class; it is never promoted to a rebuild-match class.
Evidence marked `hermeticity-mode=impure` is also rejected for rebuild-match
proof classes by default, because explicit impure mode permits ambient host inputs
outside the declared proof boundary.

A stronger deterministic-release claim is separate from `self-rebuild-match`.
Mantle models it with `mantle-deterministic-proof-receipt-v1` receipts whose
closed verdicts include `self-rebuild-match`, `mismatch`, `missing-evidence`,
`reused-store`, `impure-mode`, `unsupported-workflow`, `unsupported-sandbox`,
`provider-kind-mismatch`, and `malformed-receipt`. A deterministic receipt
records the proof unit before execution: target artifact identity, selected
provider kind, source/vendor BLAKE3 inputs, toolchain/stage roots, selected
outputs, logical store prefix, and sandbox profile identity. It requires strict
hermetic mode, at least two clean proof runs, distinct fresh store and output
root identities, a recorded ambient host perturbation matrix (`HOME`, `PATH`,
`USER`, `LOGNAME`, `TZ`, `LANG`, `LC_ALL`, temp dirs, cwd, umask, and
environment noise), typed hermeticity audit events, and matching per-output
BLAKE3 digest sets. Each run must also name a supported canonical sandbox
profile identity (`mantle-proof-sandbox-v1:<blake3>`); receipts without that
profile evidence, or with a direct-host/unsupported profile, fail closed instead
of promoting the claim. The bounded claim is only: this artifact rebuilt twice
from these recorded inputs under this sandbox and matched. It is still scoped
release-artifact evidence; it does not claim full bootstrap reproducibility or
global Nix-like determinism for all Mantle builds.

Later verification is bundle-local:

```bash
mantle release verify target/release-evidence/<release-id>
```

`mantle release verify` checks that every required bundled artifact exists,
that the top-level manifest stays canonical, that bundled digests still match,
and that the nested proof bundle is a full `crunch-self-hosting-proof-v2`
artifact rather than prerequisite-only `--check` output. It reports
`reproducibility: absent`, `matched`, or `mismatched` separately from bundle
integrity. Use `--require-reproducible` when callers need a verified
byte-for-byte report:

```bash
mantle release verify target/release-evidence/<release-id> --require-reproducible
```

The bit-for-bit reproducible release label is reserved for bundles whose
canonical reproducibility report verifies and whose artifact set matches the
published release artifact set. Ordinary bundle-local integrity is still only
packaged integrity and proof-context evidence. It lets another operator inspect
exact artifacts and verify they are internally consistent. It does not, by
itself, prove a full-source bootstrap root or independent rebuild agreement.

### Release attestations and witness verification

After a release-evidence bundle verifies, the next layer is a signed release
attestation plus optional external witness attestations. The checked-in
cross-machine flow is publisher -> witness -> publisher:

```bash
# Publisher: sign the verified release bundle into a verification directory
mantle release attest target/release-evidence/<release-id>

# Publisher: export the verifier-ready public key string used by release-verify
RELEASE_TRUSTED_KEY=$(mantle attest key-show --signing-key /path/to/release.key)
RELEASE_SIGNER_NAME="${RELEASE_TRUSTED_KEY%%:*}"

# Publisher: scaffold verifier-local policy and empty revocations files
mantle attest policy-init target/release-verification/<release-id> \
  --profile single-witness \
  --trusted-release-signer "$RELEASE_SIGNER_NAME" \
  --trusted-witness-identity <witness-identity>

# Publisher: export a public-only witness-request directory for the witness
mantle release witness-export target/release-evidence/<release-id> \
  --verification-dir target/release-verification/<release-id> \
  --request-dir target/release-witness-requests/<release-id>

# Witness: replay the checked-in rebuild workflow from the exported request
./scripts/rebuild-witness-request.sh \
  target/release-witness-requests/<release-id> \
  --identity <witness-identity> \
  --system x86_64-linux \
  --toolchain rust-1.91.1 \
  --host-class nixos-25.05 \
  --signing-key /path/to/witness.key
WITNESS_TRUSTED_KEY=$(mantle attest key-show --signing-key /path/to/witness.key)

# Publisher: import the returned witness sidecars and verify the final status
mantle attest witness-import target/release-verification/<release-id> \
  target/release-witness-requests/<release-id>.work/release-verification/<release-id>
mantle attest release-verify target/release-verification/<release-id> \
  --trusted-public-key "$RELEASE_TRUSTED_KEY" \
  --trusted-public-key "$WITNESS_TRUSTED_KEY"
```

`mantle release attest` writes `release-attestation.json` plus a detached
`.sig` file. `mantle attest key-show` reads an existing signing keypair and
prints the exact `name:base64` token accepted by `--trusted-public-key`; omit
`--signing-key` to inspect the default configured signing key instead. The
release-signer name passed to `--trusted-release-signer` is the substring
before `:` in that token, so shell workflows can derive it with
`RELEASE_SIGNER_NAME="${RELEASE_TRUSTED_KEY%%:*}"` when keys were
auto-generated in a per-operator `CRUNCH_CONFIG_DIR`. `mantle attest
policy-init` writes verifier-local `policy.json` and `revocations.json` for
either `self-proof-only` or `single-witness` publication without hand-authoring
JSON. `mantle release witness-export` copies only public verification material:
a verified release-evidence bundle, the signed release attestation, and request
metadata. It does not copy signing keys or verifier-local policy files. The
checked-in witness-side wrapper `./scripts/rebuild-witness-request.sh` is the
operator-facing path for replaying that request. It preflights `bwrap`, resolves
an absolute static `SNIX_BUILD_SANDBOX_SHELL`, derives a controlled scratch
root (`<request-dir>.work/` by default or `$CRUNCH_WITNESS_SCRATCH_DIR`), and
then calls the machine-readable core command `mantle release witness-rebuild
<request-dir> ...`. The core replay preserves the published proof mode, so a
`non-nix-host` release proof is rebuilt with the helper's `--non-nix-host` path
scrub instead of silently weakening to the default fixed-point mode. The
request directory stays immutable after validation; the rebuild output lands
under the scratch verification directory together with
`witnesses/<identity>.json`, the matching `.sig`, and
`witness-rebuild-audit/meta.json` describing the replayed workflow, scratch
paths, timestamps, and rebuilt output digests. `mantle attest witness-import`
fails closed on missing signatures, release-digest mismatches, and conflicting
existing witness identities before touching the publisher verification
directory. `mantle attest release-verify` separates technical validity from
policy sufficiency, so a release can stay technically valid even when the
witness set is policy-insufficient. In JSON mode it also reports independent
agreement separately through `independent_agreement_status`,
`independent_agreement_class`, `independent_agreement_report_digest`, counted / skipped /
failed witness counts, and per-witness classification reasons. These fields
are derived from accepted witness files plus verifier-local policy,
revocations, and trusted keys; they are not publisher-authored release claims.
Unknown-key and invalid-signature witnesses are skipped, revoked witnesses are
skipped, duplicate independence domains are skipped, and signature-valid
witnesses with wrong release references or rebuilt digests are failed evidence.
A verifier-local `agreement-report.json` must be canonical and match the
derived report; duplicate report filenames are rejected. Release-evidence
bundles may also carry the optional report artifact at
`independent-agreement/agreement-report.json`, where the manifest digest check
covers it like any other bundled file. A successful single-witness workflow
therefore proves external witness agreement under the configured policy; a
satisfied independent-agreement status gives the stronger
`independent-rebuild-agreement` class for that policy-scoped witness set. It
still does not prove a full-source bootstrap root or globally reproducible
release artifacts.

### Bootstrap roadmap

The next trust-reduction steps are straightforward:

1. Replace the reduced musl.cc-derived provider with a smaller and more
   auditable bootstrap root.
2. Reduce or encapsulate the host-tool prerequisites still needed to stage the
   first self-build.
3. Add independent rebuild evidence for release outputs instead of stopping at
   the stage0 -> stage1 -> stage2 self-hosting proof.
4. Publish the trust inventory and proof outputs as first-class release
   artifacts instead of leaving them only in code and README text.

### Background reading

Related bootstrap work worth keeping handy:

- [Bootstrappable Builds](https://www.bootstrappable.org/) — broader reference material on reducing bootstrap seeds and building from source all the way down.
- [Guix blog: The full-source bootstrap: building from source all the way down](https://guix.gnu.org/en/blog/2023/the-full-source-bootstrap-building-from-source-all-the-way-down/) — a concrete walkthrough of Guix's bootstrap story and trust reduction work.
- [Stagex](https://codeberg.org/stagex/stagex) — a stage-by-stage bootstrap project with packaging and build recipes relevant to self-hosting discussions.

**Troubleshooting the proof:**

| Symptom | Cause | Fix |
|---|---|---|
| `error: nightly toolchain 'nightly' is not installed under ~/.rustup/toolchains` | nightly toolchain missing | install the repo's nightly toolchain under `~/.rustup/toolchains` |
| `error: pkg-config cannot find openssl` | OpenSSL `.pc` files are outside the default search path | export `CRUNCH_PROOF_OPENSSL_PKGCONFIG=/path/to/openssl/lib/pkgconfig` and rerun |
| `error: required tool 'mold' not found` | linker tools missing from the shell | enter the repo dev shell or add the tool directory to `PATH` before running the helper |
| `error: required tool 'bwrap' not found` | bubblewrap not installed | `nix-shell -p bubblewrap` or install bwrap from your distro |
| `error: only <n> MiB free in proof scratch root <dir>; need at least 4096 MiB for the proof. Set CRUNCH_PROOF_SCRATCH_DIR to a larger filesystem` | selected proof scratch filesystem is too small | free space under `target/self-hosting-proof/work/`, or point `CRUNCH_PROOF_SCRATCH_DIR` at a larger writable filesystem |
| `error: proof scratch root from CRUNCH_PROOF_SCRATCH_DIR is not usable:` | explicit scratch override points at a blocked or unwritable location | point `CRUNCH_PROOF_SCRATCH_DIR` at a writable directory |
| `error: default proof scratch root is not usable:` | repo-local `target/self-hosting-proof/work/` cannot be created or written | fix repo `target/` permissions or set `CRUNCH_PROOF_SCRATCH_DIR` to a writable directory |
| Stage0 fails with permission errors writing to store | unwritable output directory | The proof uses the selected scratch root for temp files, but build outputs still need writable store and state directories |
| Stage2 reports `bwrap-source=host-fallback:` | stage0 did not produce mantle-built bwrap | Clear the proof store and rerun; the bootstrap chain may have failed silently |
| Stage2 reports `busybox-path=none` | no mantle-built busybox in the proof store | Same as above — the bootstrap chain did not complete |
| Stale pathinfo.redb causes false cache hits | prior run left state in `~/.local/state/mantle/` | The proof uses per-stage state dirs to avoid this; if running manually, pass `--state-dir` to a fresh directory |
| `No space left on device` | selected proof scratch filesystem or later proof outputs still ran out of space | free about 4 GiB under `target/self-hosting-proof/work/` or set `CRUNCH_PROOF_SCRATCH_DIR` to a larger filesystem |

## Project Management

mantle has built-in dependency management for project inputs — git repos,
tarballs, and files declared in a Nickel manifest (`mantle-project.ncl`).

```bash
# Create a new project (manifest, lockfile, .mantle/ directory)
mantle init

# Validate manifest, lockfile, and generated inputs
mantle check

# Show resolved input state from the lockfile
mantle show

# Refresh all inputs (or specific ones)
mantle refresh
mantle refresh nixpkgs my-lib

# Check which inputs would change without modifying anything
mantle list-stale

# Migrate project files to the current schema version
mantle upgrade
```

The lockfile (`mantle.lock`) stores resolved revisions and NAR hashes.
`mantle refresh` resolves upstream references (git ls-remote, content
hashing) and updates both `mantle.lock` and `.mantle/inputs.ncl`
(generated Nickel bindings).

## System configuration

`mantle` now has a native system-configuration pipeline for Nickel module
inventories.

```bash
# Dry-run a checked-in example inventory to merged derivations
mantle system eval examples/system-config/inventory.ncl

# Stop after merged fragments instead of assembling derivations
mantle system eval examples/system-config/inventory.ncl --stop-after fragments

# Build the assembled machine derivations
mantle system build examples/system-config/inventory.ncl
```

The default module directory is `./modules` next to the inventory file, and the
checked-in example inventory uses `examples/system-config/modules/`. `mantle
system eval` supports `--machine <name>` filters, `--assembler <name>` backend
overrides, and `--format json|nickel` output selection (`nickel` is reserved but
not implemented yet). `mantle system build` reuses the standard mantle build
pipeline and, under `--json`, returns a machine-level envelope whose successful
machine entries embed the existing `crunch-build-report-v1` payloads.

For module shape, inventory schema, and authoring examples, see
[`docs/system-config.md`](docs/system-config.md) and
[`examples/system-config/README.md`](examples/system-config/README.md).

## CLI

### Commands

```
# Build, diagnostics, bootstrap
mantle doctor                    No-mutate preflight for build or self-build hosts
mantle build [file.ncl|.#name]   Evaluate and build
mantle build --plan <target>     Preview cached/substitute/build/preflight-error
mantle build --fix <file>        Build and rewrite FOD mismatches in source
mantle eval <file.ncl>           Evaluate and print JSON
mantle bootstrap [-o seed.ncl]   Generate a seed file (`--fetch` for Nix-free)
mantle self-build                Rebuild mantle from source

# Store, logs, attestations, release evidence
mantle store <subcommand>        List, inspect, verify, sign, pin, push, pull, or GC store state
mantle log [query]               Show a stored build log
mantle attest <subcommand>       Show, verify, diff, or synthesize attestations
mantle release <subcommand>      Create or verify a release-evidence bundle

# Project workflows
mantle init                      Initialize mantle-project.ncl, mantle.lock, .mantle/
mantle check                     Validate manifest, lockfile, and generated inputs
mantle show                      Show resolved input state
mantle refresh [names...]        Refresh selected or all project inputs
mantle list-stale                Report which inputs would change on refresh
mantle upgrade                   Migrate project files to the current schema
mantle shell [name]              Enter or execute inside a dev shell
mantle develop [name]            Deprecated alias for `mantle shell`
mantle run [target] [-- args...] Build and execute a package binary (`.#name`, bare project package, or .ncl file)

# System configuration
mantle system eval <inventory>   Evaluate a module inventory to fragments or derivations
mantle system build <inventory>  Build a module inventory through the system pipeline
```

### Global flags

```
--store <path>              Physical output directory (default: /nix/store)
--store-prefix <prefix>     Logical store prefix (default: /mantle/store)
--nix-compat                Shorthand for --store-prefix=/nix/store
--state-dir <path>          State directory for databases and blobs
                            (default: $CRUNCH_STATE_DIR or ~/.local/state/mantle)
--json                      Emit errors as JSON for tooling integration
-v, --verbose               Debug logging
--log-level <lvl>           trace|debug|info|warn|error
```

### Build flags

```
-j, --jobs <N>                   Max concurrent builds (default: CPU count, max 16)
--fix                            Auto-fix FOD hash mismatches in .ncl source
--plan                           Preview per-root action without building
-I, --import-path <path>         Additional Nickel import paths
--substituters <url>             Binary cache URLs (default: https://cache.nixos.org)
--no-substitute                  Disable binary cache substitution
--signing-key <path>             Path to ed25519 signing keypair file
--trusted-public-keys <keys>     Trusted public keys for signature verification
--trust-unsigned                 Accept unsigned PathInfo on cache hits
--strict-hermetic                Reject degraded hermetic behavior on build-entry commands
```

## Requirements

**Build time** (compiling mantle itself):
- Linux
- Rust nightly
- clang + mold (linker)
- pkg-config + openssl-dev

**Run time** (building derivations):
- Linux (bwrap sandbox requires user namespaces)
- bwrap (bubblewrap) in PATH

**Not required**:
- protoc — gRPC/protobuf replaced with postcard serialization
- Nix — closure resolution uses mantle's own PathInfo, not `nix-store`.
  A Nix installation is only needed for `mantle bootstrap` (without
  `--fetch`)
- Writable `/nix/store` — the castore is the primary store. Cache
  validation uses PathInfo + castore content probes, not filesystem
  existence. If the output directory is not writable, builds succeed
  with outputs in castore only

## Known Limitations

- **Manual GC only**: `mantle store gc` is implemented, but background or
  low-space-triggered collection is still future work.
- **Concurrent builds**: independent derivations run in parallel (up to
  `-j N`, default: CPU count, max 16). The lazy goal scheduler
  dispatches builds as their dependencies complete; sandbox execution
  is concurrent via `tokio::JoinSet`. Preparation and output processing
  are sequential.
- **Multi-output**: outputs work end-to-end. Output *selection* is
  supported via `mantle.select dep "dev"` to mount a single output of
  a multi-output dependency in the sandbox (like Nix's `pkg.dev`).

## References

- [fosslinux/live-bootstrap](https://github.com/fosslinux/live-bootstrap) — reference stage order and source provenance for the hex0 → mes → tinycc → GCC bootstrap ladder.
