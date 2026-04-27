Task-ID: V2
Covers: bootstrap.gcc40.transition

Status: blocked.

## Blocker

No `crunch` binary available. `~/.cargo-target/debug/crunch` does not exist.
Building crunch requires nightly Rust + clang + mold + pkg-config + openssl-dev.

Also depends on binutils-tcc-chain V2 (all 48 chain dependencies must build
first, and their hashes need NAR correction).

## Required when unblocked

Command: `crunch build bootstrap/gcc-4.0.ncl`
Record: command, exit status, output path, build duration, provider selection,
fallback status, languages built (c,c++ or c only).

Verified: 2026-04-27 (blocker recorded)
