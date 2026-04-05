# crunch

Nickel build system on the Nix store protocol.

crunch evaluates `.ncl` files describing derivations, constructs store
paths using BLAKE3 hashing, and executes builds in a bwrap sandbox. No
Nix evaluator in the loop.

## Quick Start

```bash
# Build crunch (requires Rust nightly, protoc, clang, mold, openssl-dev)
cargo build --release

# Generate a seed toolchain from your Nix store
crunch bootstrap -o seed.ncl

# Write a derivation
cat > hello.ncl << 'EOF'
let crunch = import "lib.ncl" in
let seed = import "seed.ncl" in
{
  name = "hello",
  builder = "%{seed.bash}/bin/bash",
  args = ["-c", "echo 'Hello!' > $out"],
  inputs = [seed.bash],
} | crunch.Derivation
EOF

# Evaluate (print JSON, no build)
crunch eval hello.ncl

# Build (requires Linux + bwrap + writable /nix/store)
crunch build hello.ncl
```

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
/nix/store/<hash>-<name>      Output in the store
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
| `crunch` | CLI binary — `build`, `eval`, `bootstrap`, `store`, `log` |
| `crunch-eval` | Nickel evaluation, stdlib embedding |
| `crunch-glue` | `CrunchDerivation` → `nix_compat::Derivation`, KnownPaths |
| `crunch-build` | `Derivation` → `BuildRequest`, goal scheduler, build orchestration, fetchers |
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
} | crunch.Derivation
```

Contracts catch errors at eval time:
- Missing required fields (`name`, `builder`)
- Extra fields (closed contract)
- Invalid store paths, derivation names
- Wrong enum variants

## How It Differs From Nix

| | Nix | crunch |
|---|---|---|
| Config language | Nix | Nickel |
| Derivation hash | SHA-256 | BLAKE3 |
| Default addressing | Input-addressed | Content-addressed |
| Inputs | String context (implicit) | Explicit `inputs` field |
| Store paths | Compatible with Nix | Different (BLAKE3) |
| Hash algorithms | sha256/sha512/sha1/md5 | + blake3 (first-class) |
| Sandbox | Nix sandbox | bwrap (via snix-build) |
| Builder templates | stdenv, mkDerivation | None (separate packages) |
| FOD hash fix | Manual copy-paste | `--fix` auto-rewrites .ncl |

## Bootstrap

crunch needs existing binaries to build anything. The `bootstrap` command
queries your Nix installation for tool paths:

```bash
crunch bootstrap -o seed.ncl bash coreutils gcc gnumake binutils
```

This generates a `seed.ncl` with `StorePath`-validated entries. Pin them
as GC roots so Nix doesn't garbage-collect them:

```bash
nix-store --add-root /nix/var/nix/gcroots/crunch-seed -r /nix/store/...-bash
```

Source inputs automatically resolve their Nix runtime closure — listing
`seed.bash` as an input mounts bash and all its dependencies (glibc,
ncurses, etc.) in the sandbox.

## CLI

```
crunch build <file.ncl>     Evaluate and build
crunch build -j 4 <file>    Build with max 4 concurrent jobs
crunch build --fix <file>   Build, auto-fix FOD hash mismatches in .ncl source
crunch eval <file.ncl>      Evaluate and print JSON
crunch bootstrap [-o seed.ncl] [packages...]   Generate seed from Nix store
crunch store list            List all known store paths
crunch store info <path>     Show PathInfo for a store path
crunch store verify [<path>] Verify NAR hashes
crunch log [query]           Show stored build logs
crunch log --list            List all stored logs

Flags:
  --store <path>        Store directory (default: /nix/store)
  --substituters <url>  Binary cache URL (default: cache.nixos.org)
  --no-substitute       Disable binary cache substitution
  -v, --verbose         Debug logging
  --log-level <lvl>     trace|debug|info|warn|error
  -I <path>             Additional Nickel import paths

Exit codes:
  0  success
  1  build failure
  2  evaluation error
  3  internal error
```

## Requirements

- Linux (bwrap sandbox requires user namespaces)
- Rust nightly + protoc + clang + mold + openssl-dev (build time)
- bwrap (bubblewrap) in PATH (run time)
- Writable `/nix/store` for build outputs to land on disk
- Nix installation (for `bootstrap` and runtime closure resolution)

## Custom Output Directory

`--store` controls where crunch writes build outputs on disk. Source
inputs (seed packages) are always read from `/nix/store`. Derivation
path computation always uses `/nix/store` as the logical store prefix,
matching Nix convention.

```bash
mkdir -p /tmp/mystore
crunch --store /tmp/mystore build hello.ncl
# Output lands at /tmp/mystore/<hash>-hello
# Source inputs (bash, coreutils, ...) read from /nix/store
```

## Known Limitations

- **Read-only `/nix/store`**: works. The castore is the primary store.
  Cache validation uses PathInfo + castore content probes, not
  filesystem existence. Only root outputs (derivations you asked to
  build) are exported to disk. Intermediate deps stay in castore.
  If the output dir is not writable, the build succeeds with a warning.
- **Concurrent builds**: independent derivations run in parallel (up to
  `--jobs N`, default: CPU count, max 16). The lazy goal scheduler
  dispatches builds as their dependencies complete; sandbox execution
  is concurrent via `tokio::JoinSet`. Preparation and output processing
  are sequential.
- **No garbage collection**: `crunch store gc` is not implemented.
- **Multi-output**: outputs work end-to-end. Output *selection* is
  supported via `crunch.select dep "dev"` to mount a single output of
  a multi-output dependency in the sandbox (like Nix's `pkg.dev`).
