Task-ID: V3
Covers: bootstrap.binutils.tcc.chain

Status: blocked.

## Blocker

Depends on V2 build transcripts. No `crunch` binary available to produce
build logs for host leakage analysis.

## Required when unblocked

Scan V2 build transcripts for:
- References to `/nix/store` paths not declared as inputs
- References to host compiler/libc (`/usr/lib`, `/lib/x86_64-linux-gnu`)
- References to host shell (`/bin/bash` outside sandbox)
- References to legacy provider paths
- Undeclared network access

Verified: 2026-04-27 (blocker recorded)
