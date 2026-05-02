# Proposal: Repair i386 Mes libtcc1 compile flags

## Why

The i386 Mes runtime layout spike proved headers and `crt1.o`, but `tcc26-i386` failed compiling Mes `lib/libtcc1.c` with `constant exceeds 32 bit` followed by a segfault. The failure is below TinyCC 0.9.27 and Make, so fix the runtime-library handoff first.

## What Changes

- Compile Mes `lib/libtcc1.c` for the i386 proof with 32-bit-safe feature flags: disable float stubs and long-long entrypoints for this predecessor-runtime archive.
- Preserve the sibling diagnostic/proof shape and continue to the TinyCC 0.9.27 object probe only after `libtcc1.a` is built from a real object.

## Non-Goals

- Do not mutate production `bootstrap/make-tcc.ncl`.
- Do not claim Make progress.

## Verification

Build `bootstrap/spike-i386-mes-runtime-layout.ncl`, save the output summary and logs, and verify the OpenSpec change.
