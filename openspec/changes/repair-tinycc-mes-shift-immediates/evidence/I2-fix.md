Task-ID: I2
Covers: bootstrap.part.tinycc.0.9.26.shift-immediates

# TinyCC 0.9.26 immediate-shift repair

Result: PASS. `bootstrap/tinycc-mes.ncl` now applies the smallest x86_64 source normalization needed for Mes-built TinyCC 0.9.26 shift immediates.

Patch:

```sh
sed -i 's/g(vtop->c.i \& (ll ? 63 : 31));/g(vtop->c.i);/' x86_64-gen.c
```

Rationale: the predecessor compiler miscompiled the ternary/bitwise expression in the immediate shift count path and emitted zero-count shifts such as `shr $0x0,%eax` and `shl $0x0,%eax`. Emitting `vtop->c.i` directly lets `g()` write the byte count without exercising that broken expression. This leaves variable shifts, non-shift immediates, and unrelated codegen paths unchanged.

Build proof: pueue task 28 (`tinycc-mes-shift-fix-build`) succeeded in 7m37s.

Output:

```text
/home/brittonr/git/crunch/crunch/target/live-part-tinycc-0-9-27/run-current/store/idif5ffznlbrn0ji82q6fjix3pkw1n6s-tinycc-0.9.26
```

Build log proof lines:

```text
build succeeded drv=tinycc-0.9.26.drv outputs=["/home/brittonr/git/crunch/crunch/target/live-part-tinycc-0-9-27/run-current/store/idif5ffznlbrn0ji82q6fjix3pkw1n6s-tinycc-0.9.26"]
hermeticity: practical (no degraded facts)
tcc version 0.9.26 (x86_64 Linux)
```
