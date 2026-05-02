# Proposal: Repair i386 TinyCC 0.9.26 emission proof

## Why

The archived i386 live-bootstrap spike proved native i386 execution inside Crunch's sandbox and proved Crunch can build an x86_64-hosted/i386-targeting TinyCC 0.9.26 from the Mes-built TinyCC 0.9.26. The proof now blocks at a narrower failure: the generated `tcc26-i386` segfaults when asked to assemble/link the no-libc i386 `exit42.s` smoke.

This blocks any evidence-backed pivot toward the StageX/live-bootstrap i386 path for `tcc-0.9.27 -> make-3.82 pass1`.

## What Changes

- Add a focused diagnostic derivation that splits `tcc26-i386` output generation into version, assemble-only, link-from-assembly, link-from-object, and run-output stages.
- Capture the diagnostic transcript as OpenSpec evidence.
- Use the evidence to decide the smallest next repair target before attempting `tcc-0.9.27` or `make-3.82`.

## Non-Goals

- Do not mutate production `bootstrap/make-tcc.ncl`.
- Do not switch production bootstrap to i386-first in this change.
- Do not claim Make 3.82 runtime validation is complete.

## Verification

Run the diagnostic derivation through `crunch build` under `nixpkgs#bubblewrap`, save the derivation log, and run OpenSpec helper verification for this change.
