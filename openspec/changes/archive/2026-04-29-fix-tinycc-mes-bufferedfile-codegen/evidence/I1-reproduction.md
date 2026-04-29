Task-ID: I1
Covers: bootstrap.part.tinycc.0.9.26.selfcompile

# Reproduce `BufferedFile` / first-self-compile failure

Result: PASS (failure reproduced and minimized to first self-compile boundary).

Reproduction source:

- Parent evidence: `openspec/changes/live-part-tinycc-0-9-26/evidence/V2-build.md`
- Full transcript: `openspec/changes/live-part-tinycc-0-9-26/evidence/V2-build-full.log`
- Builder log: `openspec/changes/live-part-tinycc-0-9-26/evidence/V2-builder-log.log`

Command:

```sh
./target/debug/crunch build bootstrap/tinycc-mes.ncl \
  --store target/live-part-tinycc-0-9-26/store \
  --state-dir target/live-part-tinycc-0-9-26/state \
  --no-substitute -j 1 --verbose --log-level info
```

Observed boundary:

- `stage0-posix.drv` succeeded.
- `mes.drv` succeeded.
- `tinycc-0.9.26.drv` linked `tcc-mes`.
- `tcc-mes -version` succeeded twice.
- `tcc-boot0` compile then segfaulted.

Key minimized failure:

```text
tinycc-mes: mescc emit tcc.s
->type--: not a <type>: (typename "BufferedFile")
->type--: not a <type>: (typename "BufferedFile")
rank--: not a pointer:
rank--: not a pointer:
unexpected size:42
tinycc-mes: mescc link tcc-mes
tinycc-mes: rebuild-mes-runtime compiler=/tmp/tcc-build/prefix/bin/tcc-mes mode=bootstrap
tinycc-mes: compile crt1.o
tinycc-mes: compile libtcc1.o
tinycc-mes: compile tcc-boot0 with /tmp/tcc-build/prefix/bin/tcc-mes
Segmentation fault (core dumped)
```

Negative evidence:

`tcc-mes -version` is not acceptance evidence. It succeeds before the failing first self-compile, so this change must prove `tcc-boot0` and the final output contract instead.
