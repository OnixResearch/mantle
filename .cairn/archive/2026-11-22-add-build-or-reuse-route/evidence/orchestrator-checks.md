# Orchestrator verification (2026-11-22)

Independent reruns on branch `build-or-reuse-20261122` after the implementation slices.
Environment: plain cargo with `RUSTC_WRAPPER=`, `SNIX_BUILD_SANDBOX_SHELL=/bin/sh`, a scratch
`CARGO_TARGET_DIR`, and bubblewrap 0.12.0 plus the clang wrapper on `PATH`. Without `bwrap` on
`PATH` the sandboxed CLI tests fail with `bubblewrap executable 'bwrap' not found`; that is an
environment condition, not a route regression.

| Command | Result |
|---|---|
| `cargo test -p mantle --test cache_inputs_build_route` | 14 passed (exit 0) |
| `cargo test -p mantle --test foreign_import_cli` | 18 passed, 1 ignored (exit 0) |
| `cargo test -p mantle --test operator_diagnostics` | 30 passed (exit 0) |
| `cargo test -p mantle --bin mantle -- foreign_` | 118 passed, 1 ignored (exit 0) |
| `cargo fmt --check -p mantle` | exit 0 |
| `cairn validate --root .` | valid, 0 issues, 0 findings |
| `cairn gate proposal add-build-or-reuse-route --root .` | verdict PASS |
| `cairn gate design add-build-or-reuse-route --root .` | verdict PASS |
| `cairn gate tasks add-build-or-reuse-route --root .` | verdict PASS |
| `CARGO_INCREMENTAL=0 cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 |
| `git diff --check` | exit 0 |

Clippy first failed on lints this branch introduced: nine single-element `&[x.clone()]` arguments in
the `classify_present_root` and trust-policy tests (now `std::slice::from_ref`), the
544-byte `IndexedBuiltReceipt::Present` variant (now boxed), and the widened
`foreign_effect_outputs` return type (now the `ForeignEffectOutputs` alias). With incremental
compilation on, `crunch-delta` hit a compiler panic in this target directory; `CARGO_INCREMENTAL=0`
avoids it. The tests above were rerun after these fixes with the same results.
