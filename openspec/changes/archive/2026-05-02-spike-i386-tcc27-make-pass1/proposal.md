# Proposal: Spike i386 TinyCC 0.9.27 to Make 3.82 pass1

## Why

The amd64 TinyCC/Mes route for `bootstrap/make-tcc.ncl` repeatedly reached Make-oriented runtime failures. The i386 route now has a proven `tcc26-i386` handoff that can emit and run a native i386 no-libc ELF, so the next highest-ROI feature is a bounded sibling proof for the StageX/live-bootstrap ordering: i386 TinyCC 0.9.26 -> TinyCC 0.9.27 -> Make 3.82 pass1.

## What Changes

- Add a sibling diagnostic/proof derivation that uses `spike-i386-tinycc26-cross-smoke.ncl` as the predecessor compiler.
- Attempt to build an i386 TinyCC 0.9.27 and use it to compile/link GNU Make 3.82 pass1.
- Record the first concrete blocker if the full Make smoke does not pass.

## Non-Goals

- Do not mutate production `bootstrap/make-tcc.ncl`.
- Do not replace the amd64 production bootstrap path until the i386 proof reaches a real Makefile smoke.

## Verification

Run the sibling derivation through Crunch under `nixpkgs#bubblewrap`, save stdout/stderr and any output summaries under this change, then either archive on a positive Make smoke or record the narrow next blocker.
