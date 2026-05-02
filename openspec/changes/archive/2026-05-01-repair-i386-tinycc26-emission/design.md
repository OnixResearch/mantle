# Design: i386 TinyCC 0.9.26 emission diagnostics

## Context

`bootstrap/spike-i386-tinycc26-cross-smoke.ncl` currently combines several facts into one failing command: the generated `tcc26-i386` parses assembly, emits objects, links static i386 ELF output, and possibly writes the executable. A single `Segmentation fault` does not identify which boundary failed.

## Decisions

### 1. Add a sibling diagnostic derivation

**Choice:** Add `bootstrap/diag-i386-tinycc26-emission.ncl` instead of changing the archived proof target or production bootstrap derivations.

**Rationale:** This keeps the proof reproducible and isolates diagnostics from production Make/TinyCC derivations.

### 2. Record failures without failing the derivation immediately

**Choice:** The diagnostic runs each stage with `set +e`, records exit codes and output existence, then installs logs and any generated artifacts.

**Rationale:** If assemble-only segfaults, later checks can still record that no object exists; if linking creates a broken ELF, the derivation can capture mode/size/header evidence and run result.

### 3. Stay on TinyCC 0.9.26 for this slice

**Choice:** Do not reintroduce the failed 0.9.27 exploratory derivation yet.

**Rationale:** The 0.9.26 handoff is the nearest proven boundary and should be understood before adding another compiler transition.

## Risks

- A diagnostic derivation may hide a nonzero command by design. Mitigation: write an explicit summary file with each step's rc and whether expected outputs exist.
- Warm-store reuse can contaminate interpretation. Mitigation: use unique derivation name and preserve the exact build command in evidence.
