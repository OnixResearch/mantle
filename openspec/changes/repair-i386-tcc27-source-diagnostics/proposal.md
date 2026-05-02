# Proposal: Narrow i386 TinyCC 0.9.27 source-emission blocker

## Why

After the i386 Mes `libtcc1.a` flag repair, the sibling runtime-layout proof now reaches TinyCC 0.9.27 pass1 source compilation. The remaining blocker is an opaque `tcc26-i386` segfault compiling `tcc.c`. Preserve focused evidence that separates preprocessing/line-marker diagnostics and individual translation-unit failures before touching production Make routing.

## What Changes

- Mirror the missing live-bootstrap `check-reloc-null` pass1 source edit in the sibling proof.
- Align the tcc27 pass1 compile/link probes with live-bootstrap-style flags by dropping unrelated Mes feature toggles.
- Add non-gating focused diagnostic steps for preprocessing and individual TinyCC 0.9.27 units, while keeping the hard blocker on the real `tcc27_compile_object` pass1 probe.

## Non-Goals

- Do not mutate production `bootstrap/make-tcc.ncl`.
- Do not claim TinyCC 0.9.27 or Make are repaired.

## Verification

Build `bootstrap/spike-i386-mes-runtime-layout.ncl`, inspect the output summary/logs, and validate this OpenSpec change.
