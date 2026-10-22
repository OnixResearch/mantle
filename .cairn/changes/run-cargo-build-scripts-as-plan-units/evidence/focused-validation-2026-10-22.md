# Focused validation for the Cargo build-script plan units (2026-10-22).

The focused unit-plan package suites, the store fetch cases the lane depends on, strict
Clippy for the touched first-party package, the canonical format check, the Cairn
validation, and the proposal, design, and task gates.

$ cargo test -p mantle-cargo-unit-plan --lib --tests --bins --locked --offline -- --test-threads=1
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.35s
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s

$ cargo test -p crunch-store --lib pull_ --locked --offline -- --test-threads=1
test pull::tests::http_pull_nix_cache_info_client_error_warning_proceeds ... ok
test pull::tests::http_pull_store_dir_mismatch_is_hard_error_before_narinfo_fetch ... ok
test pull::tests::pull_nonexistent_source_returns_error ... ok
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 405 filtered out; finished in 8.96s

$ cargo clippy --locked --offline --no-deps -p mantle-cargo-unit-plan --lib --bins --tests -- -D warnings
    Checking mantle-rust-plan-core v0.1.0 (/home/brittonr/git/OnixResearch/mantle/crates/mantle-rust-plan-core)
    Checking mantle-cargo-unit-plan v0.1.0 (/home/brittonr/git/OnixResearch/mantle/crates/mantle-cargo-unit-plan)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 25s
clippy exit=0

$ cargo fmt --check -p mantle-cargo-unit-plan
fmt exit=0

$ cargo test -p mantle --test rust_compatibility_rail -- --test-threads 1
test native_path_only_surface_reports_matrix_binding_without_cargo_fallback ... ok
test representative_rust_plan_receipt_is_bounded_success_or_blocker_without_cargo_fallback ... ok
test supported_unit_plan_matrix_rows_retain_bounded_non_claims ... ok
test unit_plan_matrix_rejects_supported_rows_without_named_fixture ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.88s
