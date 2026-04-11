# crunch

Nickel build system on the Nix store protocol.

crunch evaluates `.ncl` files describing derivations, constructs store
paths using BLAKE3 hashing, and executes builds in a bwrap sandbox. No
Nix evaluator in the loop.

## Quick Start

```bash
# Build crunch (requires Rust nightly, clang, mold, openssl-dev)
cargo build --release

# Generate a seed toolchain from your Nix store
crunch bootstrap -o seed.ncl

# Write a derivation
cat > hello.ncl << 'EOF'
let crunch = import "lib.ncl" in
{
  name = "hello",
  builder = "/bin/sh",
  args = ["-c", "echo 'Hello, crunch!' > $out"],
} | crunch.Derivation
EOF

# Evaluate (print JSON, no build)
crunch eval hello.ncl

# Build (requires Linux + bwrap)
crunch build hello.ncl

# Machine-readable build summary for audit tooling
crunch --json build hello.ncl
```

`crunch --json build` writes a stable `crunch-build-report-v1` JSON
object to stdout. It includes per-root outcomes, cache hits, failure
records, and output paths so tests can assert on structured data instead
of scraping human text. The optional `log_file` fields are only present
when the corresponding log was actually written to disk.

Output lands in `/nix/store/<hash>-hello` by default (the `--store`
default). Derivation hashes are computed under the `/crunch/store`
logical prefix (`--store-prefix`). Use `--store /tmp/mystore` to
write outputs elsewhere, or `--nix-compat` to switch the logical
prefix to `/nix/store` for interop testing. See
[Store Paths and Prefixes](#store-paths-and-prefixes) for details.

## Fetchers

Download files, tarballs, and git repos as fixed-output derivations:

```nickel
let crunch = import "lib.ncl" in

# Single file
crunch.fetchurl {
  url = "https://example.com/foo.tar.gz",
  hash = "sha256-...",
}

# Tarball (auto-decompress + unpack + strip top-level dir)
crunch.fetchTarball {
  url = "https://github.com/user/repo/archive/v1.0.tar.gz",
  hash = "sha256-...",
}

# Git repository at a specific rev
crunch.fetchGit {
  url = "https://github.com/user/repo.git",
  rev = "abc123...",
  hash = "sha256-...",
}
```

Hash mismatches show the correct hash. Use `--fix` to auto-update:

```bash
crunch build --fix hello.ncl
```

Supported hash algorithms: sha256, sha512, sha1, md5, blake3.
Supported compression: gzip, xz, bzip2, zstd.

## Content-Addressed Derivations

The default `addressing_mode` is `'content-addressed`: output paths are
computed from the build output content (BLAKE3 hash of the NAR), not from
the derivation inputs. Identical outputs get identical paths.

```nickel
# CA is the default — no opt-in needed
{ name = "my-tool", builder = "...", ... } | crunch.Derivation

# Opt into input-addressed if needed
{ name = "my-tool", addressing_mode = 'input-addressed, ... } | crunch.Derivation
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
} | crunch.Derivation
```

Each output gets a distinct store path. The builder receives `$outputs`
(space-separated list) and individual path variables (`$out`, `$dev`,
`$man`). Works with both input-addressed and content-addressed modes.

To depend on a single output of a multi-output package, use
`crunch.select`:

```nickel
let crunch = import "lib.ncl" in
# Only mount the `dev` output of libfoo (not `out` or `lib`)
{
  name = "consumer",
  inputs = [
    crunch.select libfoo "dev",
    crunch.select libfoo "lib",
  ],
  ...
} | crunch.Derivation
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
| `crunch` | CLI binary — build, eval, bootstrap, store, log, project management, self-build |
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
let crunch = import "lib.ncl" in
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
} | crunch.Derivation
```

Contracts catch errors at eval time:
- Missing required fields (`name`, `builder`)
- Invalid store paths, derivation names
- Wrong enum variants

## How It Differs From Nix

| | Nix | crunch |
|---|---|---|
| Config language | Nix | Nickel |
| Derivation hash | SHA-256 | BLAKE3 |
| Default addressing | Input-addressed | Content-addressed |
| Default store prefix | `/nix/store` | `/crunch/store` (`--nix-compat` for `/nix/store`) |
| Inputs | String context (implicit) | Explicit `inputs` field |
| Hash algorithms | sha256/sha512/sha1/md5 | + blake3 (first-class) |
| Sandbox | Nix sandbox | bwrap (via snix-build) |
| Closure resolution | `nix-store -qR` | Own PathInfo walk (no Nix required) |
| Serialization | protobuf/gRPC | postcard |

## Store Paths and Prefixes

crunch separates two concepts:

- **Logical prefix** (`--store-prefix`, default `/crunch/store`): used for
  derivation hash computation, ATerm serialization, and output path names.
  Different prefixes produce different derivation hashes.
- **Physical directory** (`--store`, default `/nix/store`): where crunch
  writes build outputs on disk. Source inputs (seed packages) are always
  read from `/nix/store`.

```bash
# Default: logical paths under /crunch/store, outputs written to /nix/store
crunch build hello.ncl

# Write outputs to a custom directory
crunch build --store /tmp/mystore hello.ncl

# Nix-compatible hashes (for interop testing)
crunch build --nix-compat hello.ncl
```

`--nix-compat` is shorthand for `--store-prefix=/nix/store`.

## Signing and Trust

crunch signs PathInfo entries with ed25519 keys. If no `--signing-key` is
provided, an auto-generated key is created at
`$CRUNCH_CONFIG_DIR/signing-key` (or `$state_dir/signing-key` when
`CRUNCH_CONFIG_DIR` is unset; default state dir is
`~/.local/state/crunch`).

```bash
# Build with an explicit signing key
crunch build --signing-key ./my-key hello.ncl

# Verify signatures against trusted public keys
crunch store verify --trusted-public-keys "mykey-1:base64pubkey..."

# Sign all unsigned PathInfo entries (migration from unsigned stores)
crunch store sign --all

# Accept unsigned PathInfo on cache hits (escape hatch)
crunch build --trust-unsigned hello.ncl
```

By default, cached PathInfo must be signed by a trusted key. The default
trust set includes `cache.nixos.org-1`. Use `--trusted-public-keys` to
override.

## Bootstrap

crunch needs some trusted inputs to build anything. The repo tracks four
different bootstrap paths, and they do not all prove the same thing.

### Bootstrap maturity levels

- **Seed-assisted bootstrap**: crunch can start from declared external seeds
  and build later bootstrap stages.
- **Self-hosting proof**: a crunch-built `crunch` can rebuild `crunch` from
  the same staged source tree.
- **Full-source bootstrap**: the trusted root has been reduced to small,
  explicitly audited source or bootstrap seeds.
- **Reproducible release evidence**: independent rebuilds produce the same
  final artifact and can be compared or signed.

Today the repo has seed-assisted bootstrap and a checked-in self-hosting
proof. It does not yet claim a full-source bootstrap or bit-for-bit
reproducible release outputs.

### Trust inventory by entry point

#### `crunch bootstrap`

- **Current claim**: seed-assisted bootstrap from an existing Nix installation.
- **Trusted inputs today**: the host's Nix tooling and the `/nix/store` paths
  returned by `nix-build` or `nix build`.
- **Host prerequisites**: Nix installed, selected packages available in
  `nixpkgs`, and a machine where those store paths remain reachable.
- **Evidence today**: `crunch bootstrap -o seed.ncl ...` writes a
  `StorePath`-validated seed file, and later builds mount the declared source
  closures in the sandbox.
- **Not yet proven**: the imported Nix seed is not reduced to an auditable
  minimal root inside crunch itself.

#### `crunch bootstrap --fetch`

- **Current claim**: seed-assisted bootstrap without Nix.
- **Trusted inputs today**: the pinned `musl-gcc` tarball listed in
  `src/bootstrap.rs` (`FETCH_SEEDS`), plus its recursive hash.
- **Host prerequisites**: Linux, network access to fetch the seed tarball,
  and a working crunch binary with enough local disk for fetched outputs.
- **Evidence today**: the fetch path persists the tarball as a fixed-output
  derivation, writes `seed.ncl`, and the checked-in `bootstrap/*.ncl` chain
  consumes that seed.
- **Not yet proven**: the fetched `musl-gcc` seed is still a trusted binary
  bootstrap input, not a source-built root.

#### `crunch self-build`

- **Current claim**: seed-assisted self-build from the current checkout.
- **Trusted inputs today**: the current source tree, vendored Cargo deps, the
  fetched `musl-gcc` seed, and host tools used to stage and launch the first
  build.
- **Host prerequisites**: `git`, `cargo`, `tar`, `xz`, `cp`, `bwrap`, and a
  static sandbox shell for the first bootstrap stage.
- **Evidence today**: `crunch self-build --store /tmp/crunch-store -j 4 --no-substitute`
  builds the bootstrap chain `musl-gcc -> make -> dash -> binutils -> musl -> gcc -> busybox -> bwrap -> rust -> crunch`.
- **Not yet proven**: this first build still relies on host tooling and the
  fetched seed, so it is not a host-tool-free or full-source bootstrap.

#### `./scripts/prove-self-hosting.sh`

- **Current claim**: checked-in self-hosting proof.
- **Trusted inputs today**: a checkout-built stage0 `crunch` binary, the same
  fetched seed and host-tool assumptions as `crunch self-build`, and the
  staged source tree recorded by stage0.
- **Host prerequisites**: Linux, the repo's nightly Rust toolchain, `clang`,
  `mold`, `pkg-config`, OpenSSL development files, `bwrap`, `git`, `cargo`,
  `tar`, `xz`, `cp`, a static sandbox shell, and about 4 GiB free in
  `${TMPDIR:-/tmp}`.
- **Evidence today**: the helper prepares the environment and runs
  `cargo test -p crunch --test self_hosting -- --ignored --nocapture`, which
  drives the stage0 -> stage1 -> stage2 proof path.
- **Not yet proven**: the proof does not claim stage1 == stage2 identity,
  full-source bootstrap, or reproducible release artifacts.

### Bootstrappable Builds checklist

| Best-practice item | Status | Evidence today | Gap that remains |
|---|---|---|---|
| Provide an alternative way to build the build system | Yes | `cargo build --release` builds the checkout binary, and `crunch self-build` provides the in-repo bootstrap path | The bootstrap path still starts from host tooling and a fetched seed |
| Label where bootstrap binaries or tarballs came from | Partial | `crunch bootstrap` names the Nix-backed path, and `crunch bootstrap --fetch` pins the fetched `musl-gcc` URL and hash in `src/bootstrap.rs` | The repo still trusts those seeds; it does not yet reduce them to a smaller audited root |
| Reproduce bootstrap binaries from source end-to-end | Not yet | The repo can build `make`, `dash`, `binutils`, `musl`, `gcc`, `busybox`, `bwrap`, `rust`, and `crunch` from the fetched seed | The fetched `musl-gcc` seed itself is not yet rebuilt from a smaller source bootstrap inside crunch |
| Automate bootstrap traceability or self-hosting checks | Partial | `./scripts/prove-self-hosting.sh` runs the checked-in stage0 -> stage1 -> stage2 proof and `--check` verifies prerequisites first | The proof does not yet provide independent reproducibility evidence for release outputs |

## Self-Build

crunch can build itself from source with zero Nix runtime dependency:

```bash
crunch self-build --store /tmp/crunch-store -j 4 --no-substitute
```

This builds the full bootstrap chain (`musl-gcc -> make -> dash ->
binutils -> musl -> gcc -> busybox -> bwrap -> rust -> crunch`) inside a
bwrap sandbox. Output is a statically linked musl binary.

First bootstrap requires `git`, `cargo`, `tar`, `xz`, `cp`, `bwrap`, and a
static sandbox shell on `PATH`.
After the first self-build, the crunch-built `bwrap` and `busybox` are used
for subsequent builds.

### Proving self-hosting

To verify that a crunch-built binary can rebuild crunch, run the checked-in
helper from the repo root:

```bash
./scripts/prove-self-hosting.sh
```

The helper sets the nightly Rust toolchain, `CC`, linker or tool lookup,
`pkg-config` or OpenSSL lookup, and `SNIX_BUILD_SANDBOX_SHELL` before it
invokes the canonical ignored proof test:

```bash
cargo test -p crunch --test self_hosting -- --ignored --nocapture
```

Use `./scripts/prove-self-hosting.sh --check` to validate prerequisites,
including temporary-disk headroom, and print the proof command without
starting the full build.

Host prerequisites:
- Linux
- `rustup` with the repo's `nightly` toolchain installed
- `clang`, `mold`, `pkg-config`, and OpenSSL development files visible to `pkg-config`
- `bwrap`, `git`, `cargo`, `tar`, `xz`, and `cp`
- a static `SNIX_BUILD_SANDBOX_SHELL`
- about 4 GiB free in `${TMPDIR:-/tmp}`

If `pkg-config --exists openssl` does not work in your current shell, export
`CRUNCH_PROOF_OPENSSL_PKGCONFIG=/path/to/openssl/lib/pkgconfig` before running
the helper.

This runs two stages: the checkout binary builds stage1, then the stage1
binary rebuilds crunch as stage2 from the same staged source tree. The test
asserts that stage2 selected crunch-built `bwrap` and `busybox`, not host
fallbacks. Expect about 30 minutes and about 4 GiB free in `/tmp`.

The proof does not demonstrate bit-for-bit reproducibility, stage1 == stage2
identity, or freedom from all host tools. It proves that a crunch-built
`crunch` can drive another self-build to completion.

### Bootstrap roadmap

The next trust-reduction steps are straightforward:

1. Replace the fetched `musl-gcc` seed with a smaller and more auditable
   bootstrap root.
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
| `error: only <n> MiB free in <tmpdir>; need at least 4096 MiB for the proof` | insufficient temporary disk space | free space in `${TMPDIR:-/tmp}` or point `TMPDIR` at a larger filesystem |
| `error: temporary directory does not exist:` | `TMPDIR` points at a missing location | unset `TMPDIR` or point it at a real directory before running the helper |
| `error: temporary directory is not writable:` | proof tempdir is not usable | pick a writable `TMPDIR` or fix permissions before running the helper |
| Stage0 fails with permission errors writing to store | unwritable output directory | The proof uses a tempdir under `${TMPDIR:-/tmp}`; check that filesystem has space and write permissions |
| Stage2 reports `bwrap-source=host-fallback:` | stage0 did not produce crunch-built bwrap | Clear the proof store and rerun; the bootstrap chain may have failed silently |
| Stage2 reports `busybox-path=none` | no crunch-built busybox in the proof store | Same as above — the bootstrap chain did not complete |
| Stale pathinfo.redb causes false cache hits | prior run left state in `~/.local/state/crunch/` | The proof uses per-stage state dirs to avoid this; if running manually, pass `--state-dir` to a fresh directory |
| `No space left on device` | insufficient `/tmp` space | Free about 4 GiB in `/tmp`; the proof stores two full bootstrap chains |

## Project Management

crunch has built-in dependency management for project inputs — git repos,
tarballs, and files declared in a Nickel manifest (`crunch-project.ncl`).

```bash
# Create a new project (manifest, lockfile, .crunch/ directory)
crunch init

# Validate manifest, lockfile, and generated inputs
crunch check

# Show resolved input state from the lockfile
crunch show

# Refresh all inputs (or specific ones)
crunch refresh
crunch refresh nixpkgs my-lib

# Check which inputs would change without modifying anything
crunch list-stale

# Migrate project files to the current schema version
crunch upgrade
```

The lockfile (`crunch.lock`) stores resolved revisions and NAR hashes.
`crunch refresh` resolves upstream references (git ls-remote, content
hashing) and updates both `crunch.lock` and `.crunch/inputs.ncl`
(generated Nickel bindings).

## CLI

### Commands

```
crunch build <file.ncl>          Evaluate and build
crunch build -j 4 <file>         Build with max 4 concurrent jobs
crunch build --fix <file>        Build, auto-fix FOD hash mismatches in .ncl source
crunch eval <file.ncl>           Evaluate and print JSON
crunch bootstrap [-o seed.ncl]   Generate seed from Nix store (or --fetch for Nix-free)
crunch self-build                Build crunch from its own source
crunch store list                List all known store paths
crunch store info <path>         Show PathInfo for a store path
crunch store verify [<path>]     Verify NAR hashes and signatures
crunch store sign [<path>]       Sign PathInfo entries (or --all)
crunch log [query]               Show a stored build log
crunch log --list                List all stored logs
crunch init                      Initialize a new project
crunch check                     Validate project manifest and lockfile
crunch show                      Show resolved input state
crunch refresh [names...]        Refresh project inputs
crunch list-stale                List inputs that would change on refresh
crunch upgrade                   Migrate project files to current schema
```

### Global flags

```
--store <path>              Physical output directory (default: /nix/store)
--store-prefix <prefix>     Logical store prefix (default: /crunch/store)
--nix-compat                Shorthand for --store-prefix=/nix/store
--state-dir <path>          State directory for databases and blobs
                            (default: $CRUNCH_STATE_DIR or ~/.local/state/crunch)
--json                      Emit errors as JSON for tooling integration
-v, --verbose               Debug logging
--log-level <lvl>           trace|debug|info|warn|error
```

### Build flags

```
-j, --jobs <N>                   Max concurrent builds (default: CPU count, max 16)
--fix                            Auto-fix FOD hash mismatches in .ncl source
-I, --import-path <path>         Additional Nickel import paths
--substituters <url>             Binary cache URLs (default: https://cache.nixos.org)
--no-substitute                  Disable binary cache substitution
--signing-key <path>             Path to ed25519 signing keypair file
--trusted-public-keys <keys>     Trusted public keys for signature verification
--trust-unsigned                 Accept unsigned PathInfo on cache hits
```

## Requirements

**Build time** (compiling crunch itself):
- Linux
- Rust nightly
- clang + mold (linker)
- pkg-config + openssl-dev

**Run time** (building derivations):
- Linux (bwrap sandbox requires user namespaces)
- bwrap (bubblewrap) in PATH

**Not required**:
- protoc — gRPC/protobuf replaced with postcard serialization
- Nix — closure resolution uses crunch's own PathInfo, not `nix-store`.
  A Nix installation is only needed for `crunch bootstrap` (without
  `--fetch`)
- Writable `/nix/store` — the castore is the primary store. Cache
  validation uses PathInfo + castore content probes, not filesystem
  existence. If the output directory is not writable, builds succeed
  with outputs in castore only

## Known Limitations

- **No garbage collection**: `crunch store gc` is not implemented.
- **Concurrent builds**: independent derivations run in parallel (up to
  `-j N`, default: CPU count, max 16). The lazy goal scheduler
  dispatches builds as their dependencies complete; sandbox execution
  is concurrent via `tokio::JoinSet`. Preparation and output processing
  are sequential.
- **Multi-output**: outputs work end-to-end. Output *selection* is
  supported via `crunch.select dep "dev"` to mount a single output of
  a multi-output dependency in the sandbox (like Nix's `pkg.dev`).
