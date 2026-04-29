Task-ID: I3
Covers: bootstrap.part.tinycc.0.9.26

# Tinycc 0.9.26 derivation fix

Result: PASS.

`bootstrap/tinycc-mes.ncl` is now self-contained for the part scope. The follow-up blocker change `fix-tinycc-mes-bufferedfile-codegen` fixed the local source-normalization and Mes/TCC bootstrap boundary, then archived as `openspec/changes/archive/2026-04-29-fix-tinycc-mes-bufferedfile-codegen/`.

Accepted implementation facts from that archived evidence:

- `bootstrap/tinycc-mes.ncl` keeps the upstream source pins local to the derivation.
- The derivation is `input-addressed` because the produced compiler embeds logical `/crunch/store/...` runtime paths.
- `tcc-mes` compiles `tcc-boot0` without `Segmentation fault`.
- The shipped `bin/tcc` and `bin/tcc-0.9.26` are emitted by Mes/mescc with final output runtime paths, avoiding the still-defective boot0→boot1→boot2 path.
- The output retains `lib/mes/libc.a` and `lib/mes/tcc/libtcc1.a` for the produced compiler.

Primary evidence:

- `openspec/changes/archive/2026-04-29-fix-tinycc-mes-bufferedfile-codegen/evidence/I3-fix.md`
- `openspec/changes/archive/2026-04-29-fix-tinycc-mes-bufferedfile-codegen/evidence/V2-full-build.md`
- `openspec/changes/archive/2026-04-29-fix-tinycc-mes-bufferedfile-codegen/evidence/V3-smoke.md`
