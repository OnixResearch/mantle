Task-ID: V2
Covers: bootstrap.part.mes.0.27

# Mes 0.27 build evidence

Command: pueue task 16 (`mes-0-27-build-evidence`)

```sh
./target/debug/crunch build bootstrap/mes.ncl \
  --store target/live-part-mes-0-27/store \
  --state-dir target/live-part-mes-0-27/state \
  --no-substitute -j 1 --verbose --log-level info
```

Environment summary:

- `PATH` included `target/debug`, nightly Rust/Cargo, bubblewrap, clang, mold, and pkg-config.
- `SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox`
- `CRUNCH_CONFIG_DIR=target/live-part-mes-0-27/config`

Result: PASS.

Elapsed: 14m25s (`2026-04-28 19:44:10 -0400` → `2026-04-28 19:58:36 -0400`).

Output path:

```text
target/live-part-mes-0-27/store/anqp2lm1qgppznmndhgj7ighp8fbx6wn-mes
```

Hermeticity:

```text
hermeticity: practical (no degraded facts)
```

Transcript: `evidence/V2-build-full.log`.
