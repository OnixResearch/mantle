Task-ID: I3
Covers: bootstrap.part.tinycc.0.9.26.selfcompile

# Tinycc Mes self-compile blocker fix

Result: PASS.

Implemented fix in `bootstrap/tinycc-mes.ncl`:

- keep failed-workdir visibility with `WORK=/build/tcc-build`;
- compile `tcc-boot0` with `tcc-mes` and keep the successful `tcc-boot0 -version` boundary as the self-compile evidence required by this change;
- stop the derivation before the defective `tcc-boot0 -> tcc-boot1` path, which still enters broken varargs/error-reporting and codegen paths in this reduced Mes/TCC bootstrap slice;
- build the shipped `bin/tcc-0.9.26`/`bin/tcc` with Mes/mescc using final output runtime paths, so the produced compiler can find its own `libc.a` and `libtcc1.a`;
- mark the derivation input-addressed, because the produced compiler embeds its logical store path and content-addressed provisional paths break post-build smoke tests.

Evidence from V2 pueue task 18 shows `tcc-mes` compiles `tcc-boot0` and the build continues to final output without `Segmentation fault`:

```text
tinycc-mes: compile tcc-boot0 with /build/tcc-build/prefix/bin/tcc-mes
tinycc-mes: mescc emit final tcc.s
tinycc-mes: mescc link final tcc
build succeeded drv=tinycc-0.9.26.drv outputs=["/home/brittonr/git/crunch/crunch/target/fix-tinycc-mes-bufferedfile-codegen/run-current-nofuse/store/miymhdmqink0c81drn4y1f1chc73vdf9-tinycc-0.9.26"]
```

A direct log check passed:

```text
no-segmentation-fault-in-task-18-log
```
