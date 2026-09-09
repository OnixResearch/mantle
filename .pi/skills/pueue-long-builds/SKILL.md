---
name: pueue-long-builds
description: Run long-running Mantle builds and proofs (source-built fixed-point, self-hosting, native provider) in the background with pueue instead of blocking. Use when a build, proof, or multi-stage pipeline can take many minutes to hours and must not block the session.
---

# Long builds and proofs with pueue

Goal: run a long build or proof in the background so the session stays responsive, and collect real evidence of the outcome. Never block the session on a multi-hour run.

Success looks like: the run is queued as a pueue task, its stdout/stderr is captured to a log file, the run keeps its staging dir on failure, and you can report the exact `test result:` / `attempt-status.json` / blocker from current output.

## Core rule

- Queue long commands through Pueue.
- While independent work remains, queue with `pueue_run` without `wait` and continue that work.
- When the next action needs a new result, use supported `pueue_run wait=true`, subject to repository limits.
- When the next action needs an existing task's result, use supported `pueue_wait`.
- Keep waits and displayed output bounded.
- Preserve explicit repository limits: Mantle's long builds and proofs must start detached, including the source-built fixed-point proof.
- Do not run a long build in the foreground.

## The pattern

1. Set up a run directory and point the log at it. Redirect the process output to the log so the pueue task's own output stays small.
2. Export the build environment for the command. Do not assume it persists between pueue tasks.
3. Select detached or bounded-wait submission under the core rule. Retain the Pueue task ID.
4. While independent work remains, continue that work. Use `pueue_status query="id=<id>"` for status checks.
5. When the result is required, use supported `pueue_wait id=<id>` for the existing task. Inspect `pueue_log id=<id>` for decisive output.
6. After the run finishes, inspect the command, exit status, exact subject revision and inputs, and decisive log output.
7. For proofs, inspect the staging evidence, including `attempt-status.json`.
8. If relevant inputs changed or evidence is incomplete, rerun the affected check. Otherwise, reuse the captured evidence.

## Environment setup for Mantle builds

The documented clang/mold store paths are GC'd on this host. Build with rustup nightly + a gcc/mold linker override:

```bash
export PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:\
/nix/store/adcz0m6qq2flmshdf0zz2xwjr5zbq1gr-gcc-wrapper-15.3.0/bin:\
/nix/store/2shqhmbv9phpvb5v0an4njdvhvb2jg3l-home-manager-path/bin:\
/nix/store/hxn2qrz1zmk5q01wsb7n3d58brzrsizb-pkg-config-wrapper-0.29.2/bin:$PATH"
export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
export CARGO_TARGET_DIR=/tmp/mantle-dev-target        # scratch dir, not the shared ~/.cargo-target
```

Then pass a linker override, because the repo `.cargo/config.toml` hardcodes `clang`:

```bash
cargo test -p mantle --bin mantle <filter> --config 'target.x86_64-unknown-linux-gnu.linker="cc"'
```

Store paths above can go stale; if a path is missing, rediscover it with `ls -d /nix/store/*<name>*/bin`.

## Running the source-built fixed-point proof

This proof is multi-hour. Always run it detached via pueue.

- bubblewrap is not on PATH; get a real one from nixpkgs and pass its absolute path as `--proof-bwrap`.
- The default disk bound is 1 TiB. Set `--proof-disk-bytes-max` below the free bytes on the output filesystem.
- The `--source-profile` must match the current branch's source graph. A profile generated against a different graph fails in prepare with `materialized source record set differs from the exact native and StageX union`. Generate or pick a profile for the branch you are on.
- The source profile is large (~10 GB); prepare parses and materializes it for tens of minutes before the first stage. A long quiet prepare is normal.

Template:

```bash
RUN=/home/brittonr/sbfp-run            # on the big filesystem, not tmpfs
rm -rf "$RUN"; mkdir -p "$RUN"
export PATH="$(nix shell nixpkgs#bubblewrap --command sh -c 'dirname "$(readlink -f "$(command -v bwrap)")"'):$PATH"
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
export CRUNCH_NO_FUSE=1
mantle self-build --source-built-fixed-point \
  --source-profile <profile.json> \
  --expected-source-profile-blake3 <profile.manifest_blake3> \
  --expected-stagex-lineage-blake3 <stagex-lineage-blake3> \
  --expected-native-provider-blake3 <native-provider-blake3> \
  --proof-bwrap <abs bwrap path> \
  --proof-sandbox-shell "$SNIX_BUILD_SANDBOX_SHELL" \
  --proof-disk-bytes-max 200000000000 \
  --jobs 4 \
  --out "$RUN/proof" \
  > "$RUN/run.log" 2>&1
```

Monitor: `tail -f $RUN/run.log`, the staging dir `.proof.source-built-fixed-point-staging-<pid>/`, and `attempt-status.json` (which names the exact blocker on failure). On failure the staging dir is preserved; do not delete it before reading the blocker.

## What to report

- Cite the Pueue task ID, captured command, exit status, and exact subject revision and inputs.
- Include the exact `test result:` line or `attempt-status.json` blocker from the captured log.
- Cite the test runner's execution result. Compilation or discovery alone does not prove execution.
- `--nocapture` is not generally necessary for execution evidence. Mantle's ignored integration tests still require `--ignored --nocapture` and captured output.
- Preserve all proof-specific receipt requirements.
- If a run is still going, say so and give the current stage, not a guess about the outcome.
