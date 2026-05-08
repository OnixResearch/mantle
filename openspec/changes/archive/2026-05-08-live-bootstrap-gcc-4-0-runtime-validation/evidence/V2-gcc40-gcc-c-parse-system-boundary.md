# GCC 4.0 c-parse system-header boundary diagnostic

Focused command:

```sh
cargo run -- build --store .crunch-drain/gcc40-system-combo/store \
  bootstrap/diag-gcc40-c-parse-boundary.ncl
```

Result: expected failure (`rc=1`) at the existing `gcc/c-parse.c` TinyCC segmentation boundary, after a refined diagnostic matrix.

New findings in `V2-gcc40-gcc-c-parse-system-boundary-build.diag.log`:

- After forcing `make -C gcc tree-check.h config.h`, generated `config.h` is available before the diagnostic probes.
- `cparse_inc_config` now compiles (`rc=0`), correcting the earlier missing/generated-config ambiguity.
- `cparse_inc_system_only` compiles (`rc=0`).
- Any prior include before `system.h` reproduces the first TinyCC crash:
  - `auto-host.h` then `system.h`: `rc=139`
  - `ansidecl.h` then `system.h`: `rc=139`
  - manual config wrapper then `system.h`: `rc=139`
  - generated `config.h` then `system.h`: `rc=139`
- Normalizing generated `config.h` to the manual include form does not advance full `c-parse.o`; the final make-level boundary remains `gcc/c-parse.c -> Segmentation fault`.

Interpretation: the boundary is not generated `config.h` content. It is a TinyCC frontend/preprocessor state issue triggered when `system.h` is parsed after an earlier GCC header under the real c-parse flags. Next work should reduce the `system.h`-after-prior-include interaction (for example by identifying the earliest `system.h` construct that crashes after `ansidecl.h` or `auto-host.h`) before touching parser tables.
