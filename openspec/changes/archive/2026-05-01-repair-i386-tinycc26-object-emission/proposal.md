# Proposal: Repair i386 TinyCC 0.9.26 object emission

## Why

`bootstrap/diag-i386-tinycc26-emission.ncl` narrowed the i386 path blocker: the generated `tcc26-i386` prints its version but segfaults on both assembly and C object emission before producing any object. Until this works, Crunch cannot use the i386 path as evidence for `tcc-0.9.27 -> make-3.82 pass1`.

## What Changes

- Identify the TinyCC 0.9.26 host/target assumption that makes an x86_64-hosted i386-target compiler treat a small target value as a host pointer.
- Patch the sibling proof/diagnostic derivation so `tcc26-i386` can emit an object and produce/run the no-libc i386 `exit42` ELF.
- Preserve evidence showing the staged diagnostic results after repair.

## Non-Goals

- Do not mutate production `bootstrap/make-tcc.ncl`.
- Do not claim `tcc-0.9.27` or Make 3.82 works until those stages are separately proven.

## Verification

Run the repaired diagnostic/proof through Crunch under `nixpkgs#bubblewrap`, require `i386-exit42 rc=42`, save logs under this change, run OpenSpec helper verification, and commit.
