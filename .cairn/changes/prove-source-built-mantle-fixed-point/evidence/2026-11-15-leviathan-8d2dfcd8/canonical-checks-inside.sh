#!/usr/bin/env bash
set -euo pipefail
set -x
date -Is
pwd
rustc -Vv
cargo -V
nickel --version
cargo test --locked --offline -p mantle --test release_cli -- --test-threads=1
cargo test --locked --offline -p mantle --test bootstrap_parity_cli -- --test-threads=1
cargo test --locked --offline -p mantle --test machine_schema_contracts -- --test-threads=1
cargo test --locked --offline -p mantle --lib source_built_fixed_point_trust_report::receipt_tests:: -- --test-threads=1
cargo test --locked --offline -p mantle --bin mantle cargo_free_self_build::tests -- --test-threads=1
cargo test --locked --offline -p mantle --bin mantle source_toolchain_closure::tests -- --test-threads=1
cargo test --locked --offline -p mantle --bin mantle self_build::tests -- --test-threads=1
cargo test --locked --offline -p crunch-release-core --lib determinism::tests -- --test-threads=1
cargo test --locked --offline -p crunch-release-core --lib genuine_rebuild::tests -- --test-threads=1
cargo -Zscript scripts/check-machine-schema-contracts.rs
cairn validate --root .
cairn gate proposal prove-source-built-mantle-fixed-point --root .
cairn gate design prove-source-built-mantle-fixed-point --root .
cairn gate tasks prove-source-built-mantle-fixed-point --root .
date -Is
