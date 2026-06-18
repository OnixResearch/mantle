# Focused validation evidence (2026-06-18)

Task-ID: V2
Covers: r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_static_crt_normalization]

## Formatting and focused tests

Environment notes:

- `CARGO_BUILD_RUSTC_WRAPPER=` was set to bypass the global rustc wrapper.
- `RUSTC=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc`.
- `PATH` included live `mold` at `/nix/store/0gg3sqxf5jbfdzk7m483yj5biqn39dp3-mold-unwrapped-2.40.4/bin`; the previously documented mold wrapper path was absent on this host.
- `SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox`.

Command transcript (pueue task `50`):

```text
--- rustfmt check ---
--- focused route-plan test ---
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.84s

--- diff check ---
```

Additional focused wrapper tests (pueue task `43`):

```text
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.89s

running 1 test
test rust_source_provider::tests::musl_target_tool_candidates_include_source_root_aliases_only ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.00s
```

## Cairn gates

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-static-crt-normalization --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-static-crt-normalization --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-static-crt-normalization --root .
```

Result after evidence files were written and task boxes were checked (pueue task `51`, final gate excerpt):

```json
{
  "input_hash": "264e219c6edf1db20881d5e97bf180303d8c152a22b7a9b48788d49481adee8e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "receipt_hash": "b1237361bb09d0c2b1c3ab29db66d1d4f460d52baa675e347bfa705b0e816fa0",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
