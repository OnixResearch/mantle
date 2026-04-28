Task-ID: V4
Covers: bootstrap.part.stage0.posix

# V4: Host leakage scan

Command:

```sh
awk '/--- build log: stage0-posix ---/{keep=1; next} /--- end log ---/{keep=0} keep {print}' \
  evidence/V2-build-full.log > evidence/V4-build-transcript-only.log
rg -n '/usr|nix-(build|store|shell)|\bnix\b|musl\.cc|/run/current-system|/home/brittonr|/\.cargo|clang|gcc' \
  bootstrap/stage0-posix.ncl evidence/V4-build-transcript-only.log
```

Exit status: 0 for the wrapper; both forbidden-reference `rg` checks returned no matches.
Provider selection: direct source build output from V2; legacy musl.cc provider not selected.
Fallback status/event marker: inherited from V2 build: `hermeticity: practical (no degraded facts)`.
Placeholder rejection result: not applicable to scan command; V2 transcript contains no placeholder error.

Output:

```text
derivation scan: forbidden host refs
ok: no forbidden host refs in derivation (allowed sandbox /bin/sh and /bin/busybox are present)

build transcript scan: forbidden host refs
ok: no forbidden host refs in stage build transcript

allowed sandbox shell refs in derivation:
68:  builder = "/bin/sh",
73:      BB=/bin/busybox
127:f7c0fcae688ec01050d836771392f4d81f5ad6788590278ec76ccd17cdd89e48  AMD64/bin/sha256sum

build transcript line count:
858 openspec/changes/live-part-stage0-posix/evidence/V4-build-transcript-only.log
```

Notes:

- `/bin/sh` and `/bin/busybox` are the declared sandbox shell/utility boundary for this derivation.
- The `AMD64/bin/sha256sum` line is from upstream checksum data, not a host shell path.
- Outer crunch logs contain expected host paths for the worktree, state dir, and bwrap/fusermount setup; this scan intentionally checks the derivation and the stage-local build transcript only.

Raw output: `evidence/V4-host-leakage-output.txt`.
Stage-local transcript slice: `evidence/V4-build-transcript-only.log`.
