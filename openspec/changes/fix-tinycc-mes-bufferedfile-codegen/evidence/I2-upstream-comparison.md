Task-ID: I2
Covers: bootstrap.part.tinycc.0.9.26.selfcompile

# Upstream comparison and defect classification

Result: PASS (comparison complete; defect classified).

Inputs compared:

- Crunch: `bootstrap/tinycc-mes.ncl`
- Upstream: `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/steps/tcc-0.9.26/pass1.kaem`
- Upstream simple patches:
  - `simple-patches/remove-fileopen.before`
  - `simple-patches/remove-fileopen.after`
  - `simple-patches/addback-fileopen.before`
  - `simple-patches/addback-fileopen.after`

Findings:

1. Missing upstream patch bucket: not the leading cause.
   - Upstream only carries the `tcctools.c` file-open removal/addback simple patches for this step.
   - Crunch has `apply_tcctools_ar_patch()` that removes and reinserts the same `fopen(argv[i_lib], "wb")` block shape.
   - No upstream `tcc.h`/`BufferedFile` simple patch exists in this part.

2. Mes/mescc setup bucket: possible but not primary from this transcript.
   - Mes and stage0 predecessors build successfully.
   - `mescc` emits `tcc.s` and links `tcc-mes`.
   - `tcc-mes -version` runs, so module/load-path and link-root setup are sufficient for startup.
   - The failing boundary is generated compiler semantics during `tcc-boot0`, not basic Mes module discovery.

3. Local source-normalization / mescc type-handling bucket: leading classification.
   - Crunch already has a custom `normalize_buffered_file_typedef()` that is not in upstream `pass1.kaem`.
   - Despite that normalization, mescc still emits:
     `->type--: not a <type>: (typename "BufferedFile")`, `rank--: not a pointer`, and `unexpected size:42`.
   - The linked `tcc-mes` starts but produces a defective first self-compile, ending in `Segmentation fault`.

Classification:

The next fix should focus on local `BufferedFile` source normalization or mescc type handling around the generated `tcc.s`, while preserving the upstream `tcctools.c` patch and existing Mes module setup.
