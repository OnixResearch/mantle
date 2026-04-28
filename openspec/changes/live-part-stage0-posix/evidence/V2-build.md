Task-ID: V2
Covers: bootstrap.part.stage0.posix

# V2: Build `bootstrap/stage0-posix.ncl`

Command:

```sh
./target/debug/crunch build bootstrap/stage0-posix.ncl \
  --store target/live-part-stage0-posix/store \
  --state-dir target/live-part-stage0-posix/state \
  --no-substitute -j 1 --verbose --log-level info
```

Environment summary:

- `PATH` included `target/debug`, static `bubblewrap`, nightly Cargo/Rust, clang, mold, and pkg-config.
- `SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox`.
- `CRUNCH_CONFIG_DIR=target/live-part-stage0-posix/config`.

Pueue task: 15 (`stage0-posix-build-evidence`)
Exit status: 0
Elapsed: 11s
Provider selection: direct source build of `stage0-posix`; legacy musl.cc provider not selected.
Output path: `target/live-part-stage0-posix/store/bkq165k4gddjp3m6mk1qvr6h0mfrhvli-stage0-posix`
Fallback status/event marker: `hermeticity: practical (no degraded facts)`; no provider fallback marker for this ordinary stage build.
Placeholder rejection result: no `ERROR: ... is a placeholder` line appears in the build transcript.

Transcript anchors:

```text
build succeeded drv=stage0-posix.drv outputs=["target/live-part-stage0-posix/store/bkq165k4gddjp3m6mk1qvr6h0mfrhvli-stage0-posix"]
worker streaming finished completed=8 succeeded=1 failed=0 roots=1
stage0-posix build complete
hermeticity: practical (no degraded facts)
target/live-part-stage0-posix/store/bkq165k4gddjp3m6mk1qvr6h0mfrhvli-stage0-posix
```

Full transcript: `evidence/V2-build-full.log`.
