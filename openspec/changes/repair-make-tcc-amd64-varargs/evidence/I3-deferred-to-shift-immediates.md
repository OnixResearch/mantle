Task-ID: I3
Covers: bootstrap.part.make.3.82.amd64.execution

# Deferred to TinyCC immediate-shift repair

Result: DEFERRED.

After `repair-tinycc-0-9-27-amd64-compile` archived, `bootstrap/make-tcc.ncl` progressed to the repaired TinyCC 0.9.27 but still failed while compiling GNU make sources.

Observed facts:

- `crunch build bootstrap/make-tcc.ncl` failed quickly in `make-3.82-tcc.drv` with `Segmentation fault (core dumped)`.
- Outside the sandbox, the repaired TinyCC 0.9.27 compiled trivial `hello.c`, but compiling GNU make `getopt.c` either segfaulted in TinyCC relocation formatting or hung under a bounded timeout after reading the source.
- Debugging the predecessor showed current Mes-built `tinycc 0.9.26` compiles:
  - `x >> 8` as `shr $0x0,%eax`
  - `x << 3` as `shl $0x0,%eax`
- This explains why source-level workaround patches in TinyCC 0.9.27 do not scale to GNU make: the predecessor miscompiles general constant shifts.

New scoped change:

- `repair-tinycc-mes-shift-immediates`

This make repair remains open until the predecessor emits correct shift immediates and TinyCC 0.9.27 can compile GNU make `getopt.c`.
