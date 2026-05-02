# Spike i386 live-bootstrap path

## Why

`live-part-make-3-82-runtime-validation` is blocked by an amd64 Mes/TinyCC-generated GNU Make 3.82 binary that links but segfaults immediately on `--version` and simple Makefile execution. StageX uses the upstream live-bootstrap sequence for early stage1 on `linux/386`, which likely avoids this amd64-specific runtime/link seam. Crunch needs a bounded spike before deciding whether to pivot bootstrap validation to an i386-first path.

## What Changes

- Add a narrow investigation track for an i386-first live-bootstrap path through `tcc-0.9.27 -> make-3.82 pass1`.
- Compare the StageX/live-bootstrap path against Crunch's current amd64 NCL derivations.
- Define decision evidence for whether to pivot, continue amd64 repair, or keep both paths.

## Scope

In scope: reference audit, minimal Crunch design/feasibility notes, and a small proof target for i386 make runtime validation.

Out of scope: rewriting the full bootstrap chain, replacing existing amd64 derivations, or declaring StageX evidence as proof for Crunch runtime validation before Crunch runs the path itself.
