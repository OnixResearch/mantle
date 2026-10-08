# T4.2 Tiger Style on finish/add-bootstrap-source-pins (2026-10-08 UTC)

Head `5dcf9180ea395693ee1bbc94f76e3b35371692e6`; the branch is on published main `9a76abe641b7`.
The job ran under `mcf-guard.sh --devshell add-bootstrap-source-pins tiger-fix` with `bin/jobs/bsp-tiger-fix.sh`.
Log: `/home/brittonr/mantle-cairn-finish/logs/add-bootstrap-source-pins/tiger-fix-20261008T041742Z/output.log`.

## Step results

```text
rustfmt_leaf_exit=0 rustfmt_leaf_wall_s=0
fmt_touched_exit=0 fmt_touched_wall_s=5
core_pins_exit=0 core_pins_wall_s=1
core_lib_exit=0 core_lib_wall_s=0
clippy_core_platform_exit=0 clippy_core_platform_wall_s=1
tigerstyle_core_exit=0 tigerstyle_core_wall_s=10
cli_pins_exit=0 cli_pins_wall_s=1
clippy_mantle_exit=0 clippy_mantle_wall_s=86
tigerstyle_lib_keep_going_exit=1 tigerstyle_lib_keep_going_wall_s=153
phase=final exit=0 group_survivors=0 end=2026-10-08T04:22:29Z elapsed_s=287 free=958139072512 max_drop=26687946752
```

## Finding locations from `./scripts/check-first-party-tigerstyle.sh -- --lib --keep-going`

```text
26 crates/crunch-android/src/lib.rs
```

`tigerstyle_core` is the scoped run `./scripts/check-first-party-tigerstyle.sh -p crunch-project-core -- --lib --keep-going`.
It exits 0, so the crates this change touches are clean. Every remaining finding of the workspace `--lib`
run is in `crates/crunch-android/src/lib.rs`. That is pre-existing baseline debt. The integrator measured
the same 26 findings on published main `9a76abe6`, and `mantle-apk-proof-resume` owns them under
`add-apk-build-adapter`. This change does not touch that crate. The integrator reruns the full gate
after the Android fix lands. The Tiger Style repair itself is commit `5dcf9180`, which fixes 11 findings
in `crates/crunch-project-core/src/bootstrap_pins.rs`. Its tests, fmt, and strict Clippy steps above all exit 0.
