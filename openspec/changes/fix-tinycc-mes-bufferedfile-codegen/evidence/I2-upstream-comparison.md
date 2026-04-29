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

2. Mes/mescc setup bucket: not the leading cause for module discovery, but the pass1 runtime refresh was incomplete.
   - Mes and stage0 predecessors build successfully.
   - `mescc` emits `tcc.s` and links `tcc-mes`.
   - `tcc-mes -version` runs, so module/load-path and link-root setup are sufficient for startup.
   - Upstream immediately rebuilds `crt1.o`, empty amd64 `crti.o`/`crtn.o`, `libc.a` from `unified-libc.c`, `libtcc1.a`, and `libgetopt.a` with `tcc-mes` before compiling `tcc-boot0`.
   - Crunch only rebuilt `crt1.o` and `libtcc1.a` in `bootstrap` mode before `tcc-boot0`, leaving the first self-compile on the Mes-built `libc.a` instead of the upstream tcc-readable runtime.

3. Local source-normalization / mescc type-handling bucket: contributing but not sufficient alone.
   - Crunch already has a custom `normalize_buffered_file_typedef()` that is not in upstream `pass1.kaem`.
   - The normalization must keep all remaining `file` declarations on the `BufferedFile` typedef after removing the struct tag; otherwise `struct BufferedFile *file` names a separate incomplete type.
   - A standalone `mescc -S` probe still emits:
     `->type--: not a <type>: (typename "BufferedFile")`, `rank--: not a pointer`, and `unexpected size:42` even after the declaration cleanup, so the diagnostic text alone is not the acceptance boundary.
   - The linked `tcc-mes` starts but the real boundary is whether the first self-compile survives after upstream-equivalent runtime refresh.

Classification:

The next fix should restore upstream pass1 runtime refresh before `tcc-boot0` and keep the `BufferedFile` declaration cleanup local to Crunch's existing normalization, while preserving the upstream `tcctools.c` patch and source pins.
