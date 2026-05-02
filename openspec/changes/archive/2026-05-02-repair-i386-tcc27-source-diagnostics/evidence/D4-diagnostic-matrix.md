# D4 evidence: i386 tcc27 source diagnostics

Task-ID: D4
Covers: bootstrap.i386-tcc27-source-diagnostics.evidence

Command:

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute \
  --store "$PWD/.crunch-drain/make-diag-stage2-store" \
  --state-dir "$PWD/.crunch-drain/make-diag-stage2-state" \
  bootstrap/spike-i386-mes-runtime-layout.ncl
```

Exit status: 0

Output path:

```text
.crunch-drain/make-diag-stage2-store/wxgf0351sfrh1vyg36p5vqni6nrv6xz4-spike-i386-mes-runtime-layout
```

Summary:

```text
status=blocked
blocked_step=tcc27_compile_object
blocked_rc=139
note=The i386 Mes runtime/header layout proof found the first concrete blocker before tcc27 can use the layout.
```

Diagnostic matrix:

| step | rc | stderr excerpt |
| --- | ---: | --- |
| `runtime_libtcc1_object` | 0 | `` |
| `runtime_libtcc1_archive` | 0 | `` |
| `tcc27_preprocess_no_lines` | 0 | `` |
| `tcc27_preprocess_line_markers` | 139 | `Segmentation fault (core dumped) ` |
| `tcc27_compile_tccpp_unit` | 0 | `` |
| `tcc27_compile_tccelf_unit` | 0 | `` |
| `tcc27_compile_i386_gen_unit` | 0 | `` |
| `tcc27_compile_libtcc_unit` | 0 | `` |
| `tcc27_compile_tccgen_unit` | 139 | `Segmentation fault (core dumped) ` |
| `tcc27_compile_object` | 139 | `Segmentation fault (core dumped) ` |

Interpretation: the Mes runtime-library blocker is cleared (`runtime_libtcc1_*` rc 0). `tcc26-i386` can preprocess without line markers and compile several independent units, but still segfaults on line-marker preprocessing, `tccgen.c`, and the full `ONE_SOURCE=1` pass1 object. The next repair should target predecessor diagnostic/source-emission behavior around line markers and tccgen/full-source paths, not Make source edits.
