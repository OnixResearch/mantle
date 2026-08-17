# Rust source final rustdoc install-goal evidence (2026-06-15)

## Scope

This records route progress only. It does not claim a real source-built Rust provider, provider-backed self-build, or fixed-point proof.

## Prior full-route blocker

Transcript: `target/rust-source-provider-host-target-stage1-probe/transcript-2026-06-15.txt`

The fresh full route moved through `rust-1.91.1-stage1`, `rust-1.92.0-stage1`, and `rust-1.93.1-stage1`, then reached the `rust-1.94.0-final` generated x.py boundary. It failed closed because Rust 1.94.0 bootstrap no longer exposes a direct `install rustdoc` rule even though `tools = ["cargo", "rustdoc"]` is set in `bootstrap.toml`:

```text
442:  acquired source rust-1.94.0 sha256=b83f921cd3f321ff614f9c06a8b870d89299fc02888b48a5549683a36823474c extracted=/home/brittonr/git/mantle/target/rust-source-provider-host-target-stage1-probe/tmp/mantle-rust-source-provider-i1gXSd/rustc-final/rustc-final-sources/rust-1.94.0 entries=304388
443:  rustc_final_build_script: /home/brittonr/git/mantle/target/rust-source-provider-host-target-stage1-probe/tmp/mantle-rust-source-provider-i1gXSd/rustc-final/run-rustc-final.sh
444:  rustc_final_build_log: /home/brittonr/git/mantle/target/rust-source-provider-host-target-stage1-probe/tmp/mantle-rust-source-provider-i1gXSd/rustc-final/rustc-final-build.log
445:error: build failed
446:Rust source provider materialization failed closed: build: rustc final build failed with status exit status: 1; log=/home/brittonr/git/mantle/target/rust-source-provider-host-target-stage1-probe/tmp/mantle-rust-source-provider-i1gXSd/rustc-final/rustc-final-build.log; tail="...\nERROR: no `install` rules matched [rustdoc]\nHELP: run `x.py install --help --verbose` to show a list of available paths\nNOTE: if you are adding a new Step to bootstrap itself, make sure you register it with `describe!`\nBuild completed unsuccessfully in 0:00:17\n" preserved_scratch=/home/brittonr/git/mantle/target/rust-source-provider-host-target-stage1-probe/tmp/mantle-rust-source-provider-i1gXSd
```

## Implemented fix

`RUSTC_FINAL_XPY_GOALS` now uses:

```text
install rustc cargo library/std
```

The final generated bootstrap configuration still sets:

```text
tools = ["cargo", "rustdoc"]
```

The post-build validation still requires `bin/rustdoc`, so this only removes the invalid direct x.py install goal; it does not weaken the provider artifact contract.

The synthetic generated-x.py fixture now always writes `bin/rustdoc`, matching the Rust bootstrap behavior where rustdoc is produced as an extended tool selected by configuration rather than by a separate direct install path.

## Focused validation

Focused formatter/test gate (pueue task 20) ran:

```text
rustfmt src/rust_source_provider.rs
rustfmt --check src/rust_source_provider.rs
git diff --check
cargo test -p mantle --bin mantle rust_source_provider -- --nocapture --test-threads=1
cargo test -p mantle --bin mantle source_toolchain_closure -- --nocapture
```

The combined command exited successfully. The visible source-toolchain leg reported:

```text
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 734 filtered out; finished in 0.05s
```

Additional exact generated-adapter regression coverage (pueue task 24):

```text
cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_generates_xpy_adapters_when_rust_sources_lack_stage_scripts -- --exact --nocapture
cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_rejects_rust_source_without_stage_script_or_xpy_without_output -- --exact --nocapture
```

Result:

```text
test rust_source_provider::tests::materializer_generates_xpy_adapters_when_rust_sources_lack_stage_scripts ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 778 filtered out; finished in 1.99s
...
test rust_source_provider::tests::materializer_rejects_rust_source_without_stage_script_or_xpy_without_output ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 778 filtered out; finished in 0.70s
```

The positive test asserts the generated final route invokes `install rustc cargo library/std` and does not invoke `install rustc cargo rustdoc`. The negative test preserves the fail-closed behavior when a verified Rust source lacks both a Mantle stage script and `x.py`.

## Fresh rerun

A fresh full route is running as pueue task 22 with:

```text
ROOT=/home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe
OUTPUT_DIR=$ROOT/provider-out
TMPDIR=$ROOT/tmp
cargo run -p mantle --bin mantle -- -v bootstrap rust-source-provider --recipe bootstrap/rust-source.ncl --output-dir "$OUTPUT_DIR"
```

Current transcript path:

```text
target/rust-source-provider-final-rustdoc-goal-probe/transcript-2026-06-15.txt
```

Do not mark the provider task complete unless that or a later route produces a validated final `provider-out`, durable smoke evidence, provider-backed self-build, and fixed-point proof.

## Cairn validation

Command (pueue task 23):

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Result:

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
```
