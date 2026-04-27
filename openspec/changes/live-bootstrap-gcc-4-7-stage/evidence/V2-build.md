Task-ID: V2
Covers: bootstrap.gcc47.transition

Status: blocked.

## Blocker

No `crunch` binary available. Also depends on gcc-4.0 stage (V2 blocked)
and binutils-tcc-chain (V2 blocked, 48 deps need hash correction).

## Required when unblocked

Command: `crunch build bootstrap/gcc-4.7.ncl`
Record: command, exit status, output path, build duration, provider selection,
fallback status, languages built.

Verified: 2026-04-27 (blocker recorded)
