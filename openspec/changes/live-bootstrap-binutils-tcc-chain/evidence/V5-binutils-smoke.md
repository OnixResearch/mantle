Task-ID: V5
Covers: bootstrap.binutils.tcc.chain

Status: blocked.

## Blocker

Depends on V2 build of binutils-tcc.ncl. No `crunch` binary available.

## Required when unblocked

From the binutils-tcc.ncl output, verify:
- `as --version` reports GNU assembler 2.30
- `ld --version` reports GNU ld 2.30
- `ar`, `ranlib`, `nm`, `objcopy` are present and executable
- Assembler smoke test: assemble a minimal x86_64 exit syscall ELF object
  (already embedded in the derivation build script)

Verified: 2026-04-27 (blocker recorded)
