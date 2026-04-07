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
```

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
    = 'native,                 #   native | oci | wasm
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

crunch needs existing binaries to build anything. Two modes:

### From Nix (default)

The `bootstrap` command queries your Nix installation for tool paths:

```bash
crunch bootstrap -o seed.ncl bash coreutils gcc gnumake binutils
```

This generates a `seed.ncl` with `StorePath`-validated entries. Source
inputs automatically have their runtime closure resolved and mounted in
the sandbox.

### Without Nix (`--fetch`)

Downloads a static musl-gcc toolchain — no Nix installation required:

```bash
crunch bootstrap --fetch -o seed.ncl
```

This fetches pre-built tarballs, persists them as fixed-output
derivations, and writes `seed.ncl`.

## Self-Build

crunch can build itself from source with zero Nix runtime dependency:

```bash
crunch self-build --store /tmp/crunch-store -j 4 --no-substitute
```

This builds the full bootstrap chain (musl-gcc → make → dash →
binutils/musl → gcc → busybox/bwrap/rust → crunch) inside a bwrap
sandbox. Output is a statically-linked musl binary.

First bootstrap requires `git`, `cargo`, `tar`, `xz`, `bwrap` on PATH.
After the first self-build, the crunch-built bwrap and busybox are used
for subsequent builds.

### Proving self-hosting

To verify that a crunch-built binary can rebuild crunch:

```bash
cargo test -p crunch --test self_hosting -- --ignored --nocapture
```

This runs two stages: the checkout binary builds stage1, then the stage1
binary rebuilds crunch (stage2) using only crunch-built sandbox tools.
The test asserts that stage2 selected crunch-built bwrap and busybox,
not host fallbacks. Expect ~30 min and ~4 GiB free in `/tmp`.

The proof does not demonstrate bit-for-bit reproducibility or freedom
from all host tools (git, cargo, tar, xz are still needed). It proves
that a crunch-built crunch can drive another self-build to completion.

**Troubleshooting the proof:**

| Symptom | Cause | Fix |
|---|---|---|
| `SKIP: bwrap not on PATH` | bubblewrap not installed | `nix-shell -p bubblewrap` or install bwrap from your distro |
| `SKIP: not in crunch source tree` | test run from wrong directory | `cd` into the crunch workspace root (where `Cargo.toml` + `bootstrap/` live) |
| Stage0 fails with permission errors writing to store | unwritable output directory | The test uses a tempdir; check `/tmp` has space and write permissions |
| Stage2 reports `bwrap-source=host-fallback:` | stage0 did not produce crunch-built bwrap | Clear the proof store and rerun; the bootstrap chain may have failed silently |
| Stage2 reports `busybox-path=none` | no crunch-built busybox in the proof store | Same as above — the bootstrap chain did not complete |
| Stale pathinfo.redb causes false cache hits | prior run left state in `~/.local/state/crunch/` | The proof uses per-stage state dirs to avoid this; if running manually, pass `--state-dir` to a fresh directory |
| `No space left on device` | insufficient `/tmp` space | Free ~4 GiB in `/tmp`; the proof stores two full bootstrap chains |

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
