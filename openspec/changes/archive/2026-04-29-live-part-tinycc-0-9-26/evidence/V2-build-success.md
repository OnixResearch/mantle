Task-ID: V2
Covers: bootstrap.part.tinycc.0.9.26,bootstrap.part.tinycc.0.9.26.selfcompile

# Tinycc 0.9.26 build transcript

Result: PASS.

Build command accepted for this part after the blocker fix:

```sh
CRUNCH_NO_FUSE=1 \
  ./target/debug/crunch build bootstrap/tinycc-mes.ncl \
    --store "$PWD/target/fix-tinycc-mes-bufferedfile-codegen/run-current-nofuse/store" \
    --state-dir "$PWD/target/fix-tinycc-mes-bufferedfile-codegen/run-current-nofuse/state" \
    --no-substitute -j 1 --verbose --log-level info
```

Provider selection: chain-local predecessors (`stage0-posix.ncl`, `mes.ncl`) through declared derivation inputs; no legacy-provider claim is made by this part.

Exit status: success, recorded in the archived blocker-fix transcript.

Output path:

```text
/home/brittonr/git/crunch/crunch/target/fix-tinycc-mes-bufferedfile-codegen/run-current-nofuse/store/miymhdmqink0c81drn4y1f1chc73vdf9-tinycc-0.9.26
```

Relevant transcript excerpt:

```text
build succeeded drv=tinycc-0.9.26.drv outputs=["/home/brittonr/git/crunch/crunch/target/fix-tinycc-mes-bufferedfile-codegen/run-current-nofuse/store/miymhdmqink0c81drn4y1f1chc73vdf9-tinycc-0.9.26"]
worker streaming finished completed=1 succeeded=1 failed=0 roots=1
tinycc-mes: compile tcc-boot0 with /build/tcc-build/prefix/bin/tcc-mes
tinycc-mes: mescc emit final tcc.s
tinycc-mes: mescc link final tcc
hermeticity: practical (no degraded facts)
```

Negative check result from the same evidence:

```text
no-segmentation-fault-in-task-18-log
```

Primary evidence: `openspec/changes/archive/2026-04-29-fix-tinycc-mes-bufferedfile-codegen/evidence/V2-full-build.md`.
