# Verification checkpoint: cargo-free-self-build-command

## Baseline before code changes

- pueue task 81: `cargo test -p mantle --test rust_plan_cli rust_plan_cli_no_cargo_oracle_executes_path_workspace_without_invoking_cargo -- --nocapture`
  - Result: PASS, `1 passed; 0 failed; 55 filtered out`.

## Focused tests after implementation

- pueue task 83: `cargo test -p mantle --test cargo_free_self_build_cli -- --nocapture`
  - Result: PASS, `2 passed; 0 failed`.
  - Positive coverage: tiny package named `mantle` builds through `self-build --cargo-free --out <outside-root>` and emits binary, receipt, source digest, meta, and absent Cargo marker.
  - Negative coverage: fake `rustc` invokes ambient `cargo`; path guard records `cargo-was-invoked`, command fails, summary reports `status=blocked` and `blocker="cargo guard was invoked"`.
- Review-fix rerun: `cargo test -p mantle --test cargo_free_self_build_cli -- --nocapture`
  - Result: PASS, `2 passed; 0 failed`.
  - Negative coverage now also asserts blocked runs write `smoke-stdout.txt` and `smoke-stderr.txt` placeholders.
- `cargo test -p mantle --bin mantle cargo_free_self_build -- --nocapture`
  - Initial result: PASS, `4 passed; 0 failed; 577 filtered out`.
  - Review-fix result: PASS, `6 passed; 0 failed; 577 filtered out`.
  - Added unit coverage requires the selected successful `mantle` unit to carry non-null `source_digest` with algorithm/value fields; missing or null source digest returns blocker `mantle bin unit lacks source_digest`.
- `cargo test -p mantle --bin mantle self_build_cli_accepts_cargo_free_out_dir -- --nocapture`
  - Result: PASS, `1 passed; 0 failed; 580 filtered out`.
  - Review-fix result: PASS, `1 passed; 0 failed; 581 filtered out`.
- `cargo test -p mantle --bin mantle cargo_free_self_build_rejects_legacy_options -- --nocapture`
  - Result: PASS, `1 passed; 0 failed; 580 filtered out`.
- `cargo test -p mantle --bin mantle self_build_cli_accepts_impure_mode -- --nocapture`
  - Result: PASS, `1 passed; 0 failed; 580 filtered out`.
  - Review-fix result: PASS, `1 passed; 0 failed; 581 filtered out`.
- Post-change regression rerun: `cargo test -p mantle --test rust_plan_cli rust_plan_cli_no_cargo_oracle_executes_path_workspace_without_invoking_cargo -- --nocapture`
  - Result: PASS, `1 passed; 0 failed; 55 filtered out`.
- Review-fix regression rerun: same command.
  - Result: PASS, `1 passed; 0 failed; 55 filtered out`.
- `cargo fmt -p mantle --check -- src/main.rs src/cargo_free_self_build.rs tests/cargo_free_self_build_cli.rs`
  - Result: PASS.
- Review-fix format rerun: `cargo fmt -p mantle --check -- src/cargo_free_self_build.rs tests/cargo_free_self_build_cli.rs src/main.rs`
  - Result: PASS.
- `git diff --check`
  - Result: PASS.
- Review-fix `git diff --check`
  - Result: PASS.
- Review-fix Cairn validation: `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .`
  - Result: PASS, `valid: true`, `changes: 0`, `specs_validated: 1`.

## Real Mantle cargo-free self-build command proof

- pueue task 85:
  - Command: `/home/brittonr/.cargo-target/debug/mantle --json self-build --cargo-free --out /tmp/mantle-cargo-free-self-build-command/run-20260530T163627Z`
  - Result: PASS.
  - Output bundle: `/tmp/mantle-cargo-free-self-build-command/run-20260530T163627Z`.
  - Produced binary: `/tmp/mantle-cargo-free-self-build-command/run-20260530T163627Z/mantle`.
  - Binary BLAKE3: `4d9e93b5583180d00905a08a5e228f652386400c35687c3b0bcef2e6b47bfbd6`.
  - Source digest: `blake3-tree-v1:a88b1233bc857624236a72334cf76f7b247b3b35638ed568be6247949ea2ed42`.
  - Source closure digest: `0c22128c4d52cbb8e316b87690460d4ac1772a37a433351d00b635357167d456`.
  - Unit count: 599.
  - Failed unit count: 0.
  - Execution status: `success`.
  - Cargo marker absent: true.
  - Non-claims in summary: not Crunch bootstrap, not release reproducibility, not source-built toolchain closure, not full Cargo compatibility.
- Review-fix final real command proof: pueue task 92.
  - Command: `/home/brittonr/.cargo-target/debug/mantle --json self-build --cargo-free --out /tmp/mantle-cargo-free-self-build-command/review-fix-final2-20260530T203253Z`.
  - Result: PASS.
  - Output bundle: `/tmp/mantle-cargo-free-self-build-command/review-fix-final2-20260530T203253Z`.
  - Produced binary: `/tmp/mantle-cargo-free-self-build-command/review-fix-final2-20260530T203253Z/mantle`.
  - Binary BLAKE3: `add57dbdf918838004e7fad9faee0234eb0e70d28ba1a6c47878dcf927ebc43e`.
  - Source digest: `blake3-tree-v1:d1196791a204ad688099c9433630c2ca02d0b8c63d772169efb69d67bd5fb61b`.
  - Source closure digest: `df5953af6f94915dd65e9aef6274804b00e8d8227ddd81ad1b736dca4f783867`.
  - Unit count: 599.
  - Failed unit count: 0.
  - Execution status: `success`.
  - Cargo marker absent: true.
  - Smoke status code: 0.

## Notes

- The real proof keeps `--out` under `/tmp`, outside the source root, to avoid changing native source digests.
- The first attempted pueue wrapper failed before invoking Mantle because the shell redirected stdout to a missing parent directory. The rerun pre-created the parent and passed.
- `cairn archive cargo-free-self-build-command --execute --root .` moved the change package but did not sync the delta requirement into `cairn/specs/rust-package-planning/spec.md`; the requirement was manually copied into the accepted spec.
- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .` after archive/manual sync reported `valid: true`, `changes: 0`, and `specs_validated: 1`.
