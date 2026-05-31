# Cargo-free fixed-point command completion validation

## Focused tests

Command run in pueue task 99 from repo root:

```sh
export PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
cargo test -p mantle --bin mantle cargo_free -- --nocapture
cargo test -p mantle --test cargo_free_self_build_cli -- --nocapture
```

Observed output summary:

```text
running 14 tests
...
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 576 filtered out; finished in 0.00s

running 7 tests
...
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
```

Covered positive and negative tests include:

- `cargo_free_fixed_point_builds_tiny_mantle_fixture`
- `cargo_free_fixed_point_fails_if_stage_invokes_cargo`
- `cargo_free_fixed_point_reports_missing_rustc_toolchain`
- `cargo_free_fixed_point_rejects_inside_source_output_dir`
- `cargo_free_fixed_point_reports_stage_digest_mismatch`
- `rustc_wrapper_script_strips_link_self_contained_runtime_args`

## Real first-class command proof

Command run in pueue task 101 from repo root:

```sh
set -euo pipefail
bundle=/tmp/mantle-fixed-point-command-real
rm -rf "$bundle" /tmp/mantle-fixed-point-command-real.stdout /tmp/mantle-fixed-point-command-real.stderr
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
nix shell nixpkgs#rustc nixpkgs#gcc nixpkgs#clang -c sh -lc '
  set -euo pipefail
  export SNIX_BUILD_SANDBOX_SHELL="$SNIX_BUILD_SANDBOX_SHELL"
  rustc_path=$(command -v rustc)
  /home/brittonr/.cargo-target/debug/mantle --json self-build --cargo-free --fixed-point --out /tmp/mantle-fixed-point-command-real --rustc "$rustc_path" > /tmp/mantle-fixed-point-command-real.stdout 2> /tmp/mantle-fixed-point-command-real.stderr
'
cat /tmp/mantle-fixed-point-command-real.stdout
cat /tmp/mantle-fixed-point-command-real.stderr >&2
```

Observed output summary:

```json
{
  "schema": "mantle-cargo-free-fixed-point-proof-v1",
  "status": "success",
  "bundle_dir": "/tmp/mantle-fixed-point-command-real",
  "fixed_point": true,
  "stage1": {
    "execution_status": "success",
    "cargo_marker_absent": true,
    "unit_count": 599,
    "failed_unit_count": 0,
    "binary_blake3": "03c8422d8927a3ae0bc4246e70959c6f819b637ca411b945ee44ce4772707283",
    "smoke_status_code": 0,
    "blocker": null
  },
  "stage2": {
    "execution_status": "success",
    "cargo_marker_absent": true,
    "unit_count": 599,
    "failed_unit_count": 0,
    "binary_blake3": "03c8422d8927a3ae0bc4246e70959c6f819b637ca411b945ee44ce4772707283",
    "smoke_status_code": 0,
    "blocker": null
  },
  "rustc_compatibility": {
    "requested_rustc": "/nix/store/ph8jb0mw89p4lfshpp36z70l6r4kg3vh-rustc-wrapper-1.94.1/bin/rustc",
    "stage_rustc": "/tmp/mantle-fixed-point-command-real/toolchain/rustc-normalized",
    "normalization": "strip-link-self-contained-no",
    "wrapper": "/tmp/mantle-fixed-point-command-real/toolchain/rustc-normalized",
    "wrapper_blake3": "1ec2e85c08391fb3c96add6df5f1daf3ddb7e49ce60f5149de6fef4084538870"
  },
  "blocker": null,
  "non_claims": [
    "not-crunch-bootstrap",
    "not-release-reproducibility",
    "not-source-built-toolchain-closure",
    "not-full-cargo-compatibility"
  ]
}
```

## Interpretation

- Stage execution is now behind the first-class command, not the standalone proof script.
- The command selected a bundle-local normalized rustc wrapper and recorded `strip-link-self-contained-no` compatibility handling.
- Stage1 and stage2 both executed 599 units, reported zero failed units, kept Cargo guards absent, smoke-checked successfully, and produced identical BLAKE3 digests.
- The command reported bounded non-claims in the final JSON summary.

## Cairn validation and gates

Commands run from repo root after marking tasks complete:

```sh
CAIRN=$(ls /nix/store/*-cairn-0.1.0/bin/cairn 2>/dev/null | sort | tail -1)
"$CAIRN" validate --root .
"$CAIRN" gate proposal cargo-free-fixed-point-command --root .
"$CAIRN" gate design cargo-free-fixed-point-command --root .
"$CAIRN" gate tasks cargo-free-fixed-point-command --root .
```

Observed output:

```json
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 3,
  "valid": true
}
{
  "change": "cargo-free-fixed-point-command",
  "input_hash": "c7e4fa64c28cb772a21db6f84dd3984840130db462ea672a2c6cc9d3015da4de",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "cargo-free-fixed-point-command",
  "input_hash": "fb07ec6755fe8fdf6cd7df44dcbbba3fcc7d99a80ccfb67a6486d4c4ae25b98d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "cargo-free-fixed-point-command",
  "input_hash": "2248e53a1f0a74fe7922b4636ce9be8babe95fbd47e509b4cacd632cf2f0b028",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
