# V2 autoconf 2.52 build gate evidence

Task-ID: V2
Covers: bootstrap.part.autoconf.2.52

## Result

`bootstrap/autoconf-2.52.ncl` was source-hardened and audited, but a target build is prerequisite-gated. The derivation directly requires `make-3.82-tcc` and the archived runtime-validation evidence for Make records a remaining builder/runtime blocker rather than a promoted simple-Makefile runtime proof.

Relevant predecessor evidence:

- `openspec/changes/archive/2026-05-04-live-part-make-3-82-runtime-validation/evidence/V4-host-leakage-2026-05-02.md` records: "The remaining blocker is the `make-3.82-tcc` builder segfault."
- `openspec/changes/archive/2026-04-30-repair-make-tcc-amd64-varargs-runtime-validation/evidence/V1-build-full.log` records failed root `make-3.82-tcc` with build phase failure.
- `bootstrap/autoconf-2.52.ncl` also depends on early compiler/libc boundary outputs (`tcc-0.9.27-musl-v2`, `musl-1.1.24-tcc-musl`) and must not replace them with host/Nix tools.

Required transcript field status:

| Field | Status |
|---|---|
| command | deferred: target build gated on predecessor `make-3.82-tcc` runtime/build blocker |
| prerequisite | blocked: Make predecessor has archived blocker evidence and is not promotable as a runtime provider for this part |
| exit status | not run for target; prerequisite evidence blocks promotion |
| output path | none |
| failure class | prerequisite provider not validated/promoted for Autoconf 2.52 |
| fallback status | no host Autoconf/Make/TinyCC/Musl/Nix substitute accepted |

No `autoconf-2.52` build success is claimed here.
