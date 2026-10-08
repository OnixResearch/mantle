# fmt / strict first-party Clippy / diff-check on the rebased branch (2026-10-08 UTC)

Guarded Leviathan run `/home/brittonr/mantle-cairn-finish/logs/add-bootstrap-source-pins/quality-20261008T013843Z` (mcf-guard.sh; full log retained on Leviathan).
Job script: `/home/brittonr/mantle-cairn-finish/bin/jobs/bsp-quality.sh`.

## guard.log

```text
phase=preflight start=2026-10-08T01:38:43Z free=604081086464 floor=214748364800 peak_reserve=53687091200 dev=66305
cwd=/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins head=0f80aa9a4e6e412737df9065ddebcec3afdeb519 CARGO_TARGET_DIR=/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins TMPDIR=/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins CARGO_BUILD_JOBS=32
command=nix develop /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins --command /home/brittonr/mantle-cairn-finish/bin/jobs/bsp-quality.sh
phase=running launcher=2777787 pgid=2777787
phase=child_exited exit=0 elapsed_s=242 free=619591278592
phase=final exit=0 group_survivors=0 end=2026-10-08T01:42:45Z elapsed_s=242 free=619606343680
```

## exit-code

```text
0
```

## output.log (cargo Compiling/Checking progress lines elided)

```text
head=0f80aa9a4e6e412737df9065ddebcec3afdeb519
+ cargo fmt --check -p mantle -p crunch-project-core -p mantle-portable-client-core
fmt_touched_exit=0 fmt_touched_wall_s=6
+ rustfmt --edition 2024 --check src/bootstrap_pin_cmd.rs crates/crunch-project-core/src/bootstrap_pins.rs
rustfmt_leaf_exit=0 rustfmt_leaf_wall_s=0
+ cargo clippy -p crunch-project-core -p mantle-portable-client-core --all-targets --no-deps -- -D warnings
warning: /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins/Cargo.toml: file `/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.35s
clippy_core_platform_exit=0 clippy_core_platform_wall_s=11
+ cargo clippy -p mantle --all-targets --no-deps -- -D warnings
warning: /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins/Cargo.toml: file `/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3m 39s
clippy_mantle_exit=0 clippy_mantle_wall_s=221
+ git diff --check main
diff_check_exit=0 diff_check_wall_s=0
quality_done
```
