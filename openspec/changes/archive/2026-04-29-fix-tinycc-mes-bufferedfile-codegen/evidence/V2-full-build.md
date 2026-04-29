Task-ID: V2
Covers: bootstrap.part.tinycc.0.9.26.selfcompile

# Full tinycc Mes build

Command:

```sh
CRUNCH_NO_FUSE=1 \
  ./target/debug/crunch build bootstrap/tinycc-mes.ncl \
    --store "$PWD/target/fix-tinycc-mes-bufferedfile-codegen/run-current-nofuse/store" \
    --state-dir "$PWD/target/fix-tinycc-mes-bufferedfile-codegen/run-current-nofuse/state" \
    --no-substitute -j 1 --verbose --log-level info
```

Result: PASS via pueue task 18.

Output path:

```text
/home/brittonr/git/crunch/crunch/target/fix-tinycc-mes-bufferedfile-codegen/run-current-nofuse/store/miymhdmqink0c81drn4y1f1chc73vdf9-tinycc-0.9.26
```

Required output files checked in V3 preflight:

```text
bin/tcc
bin/tcc-0.9.26
lib/mes/libc.a
lib/mes/tcc/libtcc1.a
```

Relevant transcript from `pueue_log 18`:

```text
build succeeded drv=tinycc-0.9.26.drv outputs=["/home/brittonr/git/crunch/crunch/target/fix-tinycc-mes-bufferedfile-codegen/run-current-nofuse/store/miymhdmqink0c81drn4y1f1chc73vdf9-tinycc-0.9.26"]
worker streaming finished completed=1 succeeded=1 failed=0 roots=1
--- build log: tinycc-0.9.26 ---
tcc version 0.9.26 (x86_64 Linux)
tcc version 0.9.26 (x86_64 Linux)
tcc version 0.9.26 (x86_64 Linux)

tinycc-mes: mescc emit tcc.s
tinycc-mes: mescc link tcc-mes
tinycc-mes: rebuild-mes-runtime compiler=/build/tcc-build/prefix/bin/tcc-mes mode=full
tinycc-mes: compile crt1.o
tinycc-mes: compile libtcc1.o
tinycc-mes: compile unified-libc.o
unified-libc.c:2425: warning: SYS_exit redefined
unified-libc.c:2644: warning: SYS_write redefined
tinycc-mes: compile getopt.o
tinycc-mes: compile tcc-boot0 with /build/tcc-build/prefix/bin/tcc-mes
tinycc-mes: mescc emit final tcc.s
tinycc-mes: mescc link final tcc
--- end log ---
hermeticity: practical (no degraded facts)
```

Negative check:

```sh
grep -q 'Segmentation fault' /home/brittonr/.local/share/pueue/task_logs/18.log
```

Result:

```text
no-segmentation-fault-in-task-18-log
```
