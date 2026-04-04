# crunch

Nickel build system on the Nix store protocol.

crunch evaluates `.ncl` files describing derivations, constructs store
paths using BLAKE3 hashing, and executes builds in a bwrap sandbox. No
Nix evaluator in the loop.

## Quick Start

```bash
# Build crunch (requires Rust nightly, protoc, clang, mold, openssl-dev)
cargo build -p crunch

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

# Build (requires Linux + bwrap)
crunch build hello.ncl
```

## Architecture

```
.ncl file
    │
    ▼
crunch-eval (nickel-lang)     Evaluate Nickel → JSON
    │
    ▼
crunch-glue (nix-compat)      JSON → nix_compat::Derivation + BLAKE3 store paths
    │
    ▼
crunch-build (snix-build)     Derivation → BuildRequest → bwrap sandbox → PathInfo
    │
    ▼
/nix/store/<hash>-<name>      Output in the store
```

## Crate Layout

| Crate | Role |
|---|---|
| `crunch` | CLI binary — `build`, `eval`, `bootstrap` |
| `crunch-eval` | Nickel evaluation, stdlib embedding |
| `crunch-glue` | `CrunchDerivation` → `nix_compat::Derivation`, KnownPaths |
| `crunch-build` | `Derivation` → `BuildRequest`, build orchestration |
| `vendor/nix-compat` | Store paths, ATerm, NAR (BLAKE3-modified) |
| `vendor/snix-build` | `BuildService` trait, bwrap sandbox |
| `vendor/snix-castore` | Blob and directory content-addressed storage |
| `vendor/snix-store` | `PathInfoService`, NAR calculation |

## Nickel Stdlib

The stdlib (`lib/`) defines the derivation schema:

```nickel
let crunch = import "lib.ncl" in
{
  name = "myapp",           # | Name (validated)
  builder = "/bin/sh",      # | String
  system = 'x86_64-linux,   # | System enum (default)
  args = ["-c", "..."],     # | Array String (default [])
  outputs = ["out"],         # | Array String (default ["out"])
  env = { CC = "gcc" },     # | { _ : String } (default {})
  inputs = [                 # | Array Input (default [])
    "/nix/store/...-bash",   #   StorePath string → source input
    { name = "lib", ... },   #   Derivation record → built first
  ],
  fixed_output = {           # | FixedOutput | optional
    hash = "sha256-...",
    algo = 'sha256,
    mode = 'flat,
  },
} | crunch.Derivation
```

Contracts catch errors at eval time:
- Missing required fields (`name`, `builder`)
- Extra fields (closed contract)
- Invalid store paths, derivation names
- Wrong enum variants (`'x86_64_linux` vs `'x86_64-linux`)

## How It Differs From Nix

| | Nix | crunch |
|---|---|---|
| Config language | Nix | Nickel |
| Derivation hash | SHA-256 | BLAKE3 |
| Inputs | String context (implicit) | Explicit `inputs` field |
| Store paths | Compatible with Nix | Different (BLAKE3) |
| Sandbox | Nix sandbox | bwrap (via snix-build) |
| Builder templates | stdenv, mkDerivation | None (separate packages) |

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

## CLI

```
crunch build <file.ncl>     Evaluate and build
crunch eval <file.ncl>      Evaluate and print JSON
crunch bootstrap [-o seed.ncl] [packages...]   Generate seed from Nix store

Flags:
  --store <path>     Store directory (default: /nix/store)
  -v, --verbose      Debug logging
  --log-level <lvl>  trace|debug|info|warn|error
  -I <path>          Additional Nickel import paths

Exit codes:
  0  success
  1  build failure
  2  evaluation error
  3  internal error
```
