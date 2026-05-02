# Proposal: Spike i386 Mes runtime/header layout

## Why

The i386 TinyCC 0.9.26 predecessor can emit and run a no-libc i386 ELF, but the TinyCC 0.9.27 handoff failed before Make because it had no real i386 Mes include/CRT/libc/libtcc1 layout. A focused sibling proof should create that layout and record the next concrete compiler/runtime blocker without mutating production bootstrap derivations.

## What Changes

- Add `bootstrap/spike-i386-mes-runtime-layout.ncl` as a sibling diagnostic derivation.
- Assemble an i386 Mes header/CRT layout from Mes 0.27.1 sources and the proven `tcc26-i386` predecessor.
- Probe the TinyCC 0.9.27 object handoff using that layout, recording whether the blocker is headers, runtime object/archive generation, or TinyCC 0.9.27 compilation.

## Non-Goals

- Do not change production `bootstrap/make-tcc.ncl`.
- Do not claim Make 3.82 progress unless an actual Make binary and Makefile smoke pass.

## Verification

Run the sibling derivation through Crunch, copy its summary/logs into evidence, and archive the spike with the next blocker clearly identified.
