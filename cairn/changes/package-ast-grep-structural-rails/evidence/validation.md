# package-ast-grep-structural-rails validation evidence

Date: 2026-07-12
Task: `mantle.ast_grep_structural_rails.validation`

## Pinned package identity

Command:

```text
nix build .#checks.x86_64-linux.ast-grep-package-identity --no-link --print-out-paths -L
nix run .#ast-grep-toolchain -- --version
```

Result: PASS

```text
package: ast-grep
version: 0.42.1
upstream_store_path: /nix/store/zq24d8np6247dkf3zf6dk7s1y8pr6ivz-ast-grep-0.42.1
binary: bin/ast-grep
digest_algorithm: blake3
binary_digest_blake3: b0dc1f9346b20daf594729c1b81ee18f696536c19494551509a14f7153f6a3c6
identity_check_output: /nix/store/x3jh30yi4av16b6lqc4wvh7jrj2y5vjg-mantle-ast-grep-package-identity-smoke
version_output: ast-grep 0.42.1
```

The identity check also required `sg --version` to equal `ast-grep --version` and revalidated the recorded upstream store path.

## Focused Mantle checks

Commands and observed results:

```text
cargo test -p crunch-release-core --lib ast_grep::tests
PASS: 6 passed; 0 failed

cargo test -p mantle --bin mantle ast_grep
PASS: 10 passed; 0 failed

cargo test -p mantle --bin mantle remote_client_json_report_uses_build_report_schema_and_substitution_fields
PASS: 1 passed; 0 failed

cargo clippy -p crunch-release-core --lib --tests -- -D warnings
PASS

cargo check -p crunch-release-core --target wasm32-unknown-unknown
PASS

rustfmt --check crates/crunch-release-core/src/ast_grep.rs crates/crunch-release-core/src/lib.rs src/ast_grep_evidence.rs src/main.rs src/release_cmd.rs tools/tracey_refs.rs
PASS

nixfmt --check flake.nix
git diff --check
PASS
```

The focused tests cover positive scan/rule-test fixtures, a self-consistent unpinned tool version, stale and malformed identities, missing non-claims, unsupported output, unknown/explicit release overclaims, shell purity and bounded reads, accepted/diagnostic build-report paths, release revalidation, and unchanged empty-array remote-report shape.

## Cairn lifecycle gates

Commands:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal package-ast-grep-structural-rails --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design package-ast-grep-structural-rails --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks package-ast-grep-structural-rails --root .
```

Results:

```text
validate: valid=true, issues=[], changes=18, specs_validated=43
proposal: PASS, issues=[], receipt_hash=6668aa31853a0943e534ab44d24c040cbe450f6314b9c304681b435a9d5958d5
design: PASS, issues=[], receipt_hash=d624765fecffe6301bf97e0b696551a907cebd17a8941803b11b1eb52a0da83b
tasks: PASS, issues=[], receipt_hash=eaa32e5000c9cb053a45c9273e2b1803474148fb1161ca9c92d79e922455645a
```

## Exact pre-existing broad-rail blockers

These are not blockers to the focused change gates above:

- `./scripts/check-first-party-tigerstyle.sh -p crunch-release-core` still exits non-zero on 315 pre-existing findings in other modules. A rerun after correcting all new findings produced no `ast_grep.rs` diagnostic.
- `cairn tracey coverage --root .` uses the current `mantle-default` profile, which scans only accepted `cairn/specs/release-provenance/spec.md`; it reports 73 requirements, 18 referenced, and 55 pre-existing missing references. The active ast-grep verification-evidence delta is not a requirement source in that profile.
- Package-wide `cargo fmt --check -p mantle -p crunch-release-core` reaches unrelated pre-existing formatting differences in legacy `src/build_report.rs` test setup and `tests/trust_policy_offline_rail.rs`; all newly added Rust modules and the directly format-checked changed files pass.
- Root-package `cargo clippy -p mantle --bin mantle --no-deps -- -D warnings` reaches unrelated existing lint debt beginning at `src/build_correctness.rs:493` (`result_large_err`) and `src/build_correctness.rs:541` (`collapsible_if`); focused core Clippy passes.

No Cairn sync or archive command was run.
