# Tasks: Narrow i386 TinyCC 0.9.27 source-emission blocker

## Diagnostics

- [x] D1 Add the missing live-bootstrap tcc-0.9.27 `check-reloc-null` pass1 source edit to the sibling proof. [covers=bootstrap.i386-tcc27-source-diagnostics.pass1-parity]
- [x] D2 Align tcc27 compile/link probe flags with live-bootstrap pass1 by removing unrelated Mes feature toggles. [covers=bootstrap.i386-tcc27-source-diagnostics.pass1-parity]
- [x] D3 Add non-gating diagnostic steps that distinguish preprocessing, line-marker, and per-unit compile outcomes. [covers=bootstrap.i386-tcc27-source-diagnostics.narrowing]
- [x] D4 Run Crunch evidence showing the real Mes libtcc1 archive still succeeds and the hard blocker remains `tcc27_compile_object`. [covers=bootstrap.i386-tcc27-source-diagnostics.evidence]
- [x] D5 Verify OpenSpec change. [covers=bootstrap.i386-tcc27-source-diagnostics.evidence]
