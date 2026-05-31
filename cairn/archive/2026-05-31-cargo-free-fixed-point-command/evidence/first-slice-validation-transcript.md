# First-slice validation transcript

## Commands

```sh
env PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox cargo test -p mantle --bin mantle cargo_free -- --nocapture
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .
env PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$PATH rustfmt --edition 2024 --check src/cargo_free_self_build.rs src/main.rs
git diff --check
git diff HEAD --check
```

## Output

```text
+ cargo test -p mantle --bin mantle cargo_free -- --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.52s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-da169f9269173e44)

running 13 tests
test cargo_free_self_build::tests::child_blocker_fails_when_cargo_guard_was_invoked ... ok
test cargo_free_self_build::tests::fixed_point_plan_rejects_bundle_inside_source_root ... ok
test cargo_free_self_build::tests::child_blocker_accepts_successful_guarded_execution ... ok
test cargo_free_self_build::tests::fixed_point_plan_rejects_relative_source_root ... ok
test cargo_free_self_build::tests::fixed_point_plan_describes_stage_paths_and_commands ... ok
test cargo_free_self_build::tests::safe_path_component_replaces_unsafe_path_bytes ... ok
test cargo_free_self_build::tests::mantle_unit_from_receipt_rejects_null_source_digest ... ok
test cargo_free_self_build::tests::mantle_unit_from_receipt_requires_source_digest ... ok
test tests::cargo_free_self_build_rejects_legacy_options ... ok
test tests::self_build_cli_rejects_fixed_point_without_cargo_free ... ok
test tests::self_build_cli_accepts_cargo_free_out_dir ... ok
test tests::cargo_free_fixed_point_rejects_legacy_options ... ok
test tests::self_build_cli_accepts_cargo_free_fixed_point_out_dir ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 576 filtered out; finished in 0.00s

status=0
+ /nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 2,
  "valid": true
}
status=0
+ rustfmt --edition 2024 --check src/cargo_free_self_build.rs src/main.rs
status=0
+ git diff --check
status=0
+ git diff HEAD --check
status=0
```
