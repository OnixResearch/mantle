# Source-built closure fixed-point proof attempt

Task-ID: V4-attempt
Covers: rust_package_planning.source_built_toolchain_closure
Date: 2026-06-01
Owner: agent

## Scope

This is an end-to-end proof attempt using the materialized source-root musl GCC provider for target linking. It did **not** complete the source-built toolchain closure fixed-point proof. The V4 task remains unchecked.

## Inputs

- Source-root provider: `.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain`
- Provider manifest digest: `54ca36ac9eacc0193929cf20e457ccc065866839028146ad13d552346505f900`
- Provider output digest: `70674e2e762f22ccf6512ad36d90bc07b608ae0495c216be300a60e61e74c41b`
- Attempt run dir: `.pi/source-built-closure-proof-20260601T004426Z`
- Toolchain closure manifest: `.pi/source-built-closure-proof-20260601T004426Z/toolchain-closure.json`
- Durable manifest copy: `evidence/source-built-closure-attempt-toolchain-closure.json`
- Durable run metadata: `evidence/source-built-closure-attempt-summary.env`
- Rust wrapper: `.pi/source-built-closure-proof-20260601T004426Z/rustc-source-root-target-wrapper`
- Self-build output dir: `/tmp/mantle-source-built-closure-self-build-source-built-closure-proof-20260601T004426Z`
- Durable summary copy: `evidence/source-built-closure-attempt-self-build-summary.json`
- Durable stderr copy: `evidence/source-built-closure-attempt-self-build-stderr.txt`

The closure manifest deliberately used:

- source-built members: source-root musl GCC as `linker` and `c-compiler`
- seed exceptions: Rust compiler wrapper/prebuilt Nix rustc 1.94.1, Rust sysroot/std, host pkg-config, and host linker for host proc-macro/build-script units

## Supporting code repair

The attempt exposed a real aliasing issue before the proof command could honestly use source-root tools: source-root provider executables are shell wrappers that locate their runtime relative to `$0`. Symlinking `cc` to those wrappers changes `$0` to the guard-dir symlink and breaks runtime discovery.

`src/cargo_free_self_build.rs` now writes executable alias wrappers instead of symlinks and exposes a declared `ld` alias for the Linker role. Focused unit test evidence is pueue task 39:

```text
running 22 tests
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_declared_linker_for_collect2 ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 628 filtered out; finished in 0.00s
```

A direct smoke check with the generated Rust wrapper, `CRATE_KIND=bin`, source-root `cc`, and `--target x86_64-unknown-linux-musl` produced and ran a static-pie hello binary before the Mantle proof attempt:

```text
accepts
/tmp/tmp.8FekdXFxNw/probe: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), static-pie linked, with debug_info, not stripped
hi
```

## Proof attempt command

Pueue task `44`:

```sh
run_dir=$(ls -td /home/brittonr/git/mantle/.pi/source-built-closure-proof-* | head -1)
. "$run_dir/summary.env"
out="/tmp/mantle-source-built-closure-self-build-$(basename "$run_dir")"
summary="$run_dir/self-build-summary.json"
stderr="$run_dir/self-build-stderr.txt"
export PATH=/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/bin:/usr/bin
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
timeout 1200 /home/brittonr/.cargo-target/debug/mantle --json self-build --cargo-free --out "$out" --rustc "$wrapper" --toolchain-closure "$manifest" >"$summary" 2>"$stderr"
```

## Result

Pueue task `44` failed because Mantle correctly returned a blocked proof summary:

```json
{
  "schema": "mantle-cargo-free-self-build-v1",
  "status": "blocked",
  "execution_status": "blocked",
  "cargo_marker_absent": true,
  "unit_count": 11,
  "failed_unit_count": 1,
  "blocker": "topology execution status was blocked",
  "source_built_toolchain_closure": {
    "schema": "mantle-source-built-toolchain-closure-v1",
    "status": "validated-enforced",
    "claim": false,
    "non_claim": "not-source-built-toolchain-closure",
    "policy_digest_blake3": "ef742e9e26018b287f4c26335abc4be7843141261a6379266f9812dff94c5a15",
    "member_count": 6,
    "source_built_member_count": 2,
    "seed_exception_count": 4
  }
}
```

The first failed unit was `aws-lc-sys`'s custom-build unit:

```text
unit_id: 11:registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.1:build-script-build:custom-build:build
execution_status: failed
blocker.class: rustc-failed
blocker.message: error E0461: couldn't find crate `cc` with expected target triple x86_64-unknown-linux-gnu
note: crate `cc`, target triple x86_64-unknown-linux-musl: <redacted-temp-path>
```

## Decision

The real source-built closure fixed-point task cannot be checked from this attempt. Mantle can enforce the receipt-bound closure and can drive a source-root musl linker for target binaries, but the native Rust topology planner currently builds build-script dependency libraries for the target triple. Host custom-build/proc-macro units then expect those dependencies as `x86_64-unknown-linux-gnu`, while the attempted source-root target path produced `x86_64-unknown-linux-musl` artifacts.

This is a real host/target split blocker, not a manifest validation issue.

## Validation after recording blocker

Pueue task `46` passed after recording this blocked attempt and the alias-wrapper repair:

```text
cargo test -p mantle --bin mantle cargo_free_self_build -- --nocapture
running 22 tests
...
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 628 filtered out; finished in 0.00s

cargo fmt --check -p mantle -v
git diff --check
cairn validate --root .
{
  "valid": true
}
cairn gate tasks source-built-toolchain-closure --root .
{
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Next action

Add a follow-up implementation slice for cross-target native topology execution: host units and their dependency libraries must compile for the host triple/linker, while target lib/bin units compile for the selected target triple/source-root linker. Then rerun the source-built closure fixed-point proof and keep V4 unchecked until the proof reaches stage1/stage2 fixed point.
