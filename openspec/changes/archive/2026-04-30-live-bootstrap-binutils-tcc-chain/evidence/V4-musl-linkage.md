Task-ID: V4
Covers: bootstrap.binutils.tcc.chain

Status: blocked.

## Blocker

Depends on V2 build outputs. No `crunch` binary available to build post-musl
derivations and inspect their linkage.

## Required when unblocked

For post-musl tools (m4-1.4.7, flex-2.5.11, flex-2.6.4, bison-2.3,
bison-3.4.1, grep-2.4), verify:
- `readelf -d <binary>` shows NEEDED entries referencing musl libc
- No references to mes libc or host glibc
- Dynamic linker is musl `ld-musl-x86_64.so.1`

Verified: 2026-04-27 (blocker recorded)
