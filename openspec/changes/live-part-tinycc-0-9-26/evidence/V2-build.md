Task-ID: V2
Covers: bootstrap.part.tinycc.0.9.26

# Build validation

Command:

```sh
./target/debug/crunch build bootstrap/tinycc-mes.ncl \
  --store target/live-part-tinycc-0-9-26/store \
  --state-dir target/live-part-tinycc-0-9-26/state \
  --no-substitute -j 1 --verbose --log-level info
```

Environment:

- `SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox`
- `CRUNCH_CONFIG_DIR=target/live-part-tinycc-0-9-26/config`

Result: BLOCKED / FAIL.

Pueue task: `17` (`tinycc-0-9-26-build-evidence`)

Elapsed: `17m 3s`

Failure class:

- Mes and stage0 predecessors built successfully.
- `tcc-mes -version` succeeded twice.
- The first self-compile `tcc-boot0` segfaulted after mescc emitted `BufferedFile` type diagnostics.

Key excerpt:

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

Transcripts:

- `evidence/V2-build-full.log`
- `evidence/V2-builder-log.log`

Next action:

- Fix or isolate mescc `BufferedFile` codegen/type handling before this part can satisfy the required successful `crunch build bootstrap/tinycc-mes.ncl` evidence.
- `tcc-mes -version` is explicitly not enough evidence for this part.
