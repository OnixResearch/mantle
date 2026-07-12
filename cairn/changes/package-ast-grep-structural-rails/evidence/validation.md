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

No Cairn sync or archive command was run by the implementation agent.

## Integrated-main hardening and validation

After integration, adversarial review identified two claim-boundary issues and one filesystem hazard. The accepted design and documentation now state that sidecar observed/expected binary equality is producer-declared consistency, not authentication that the packaged executable ran; stronger attribution requires the checked package identity record plus an authenticated execution receipt. Sidecar reads now open every parent directory through a no-follow capability root, open the final file without following symlinks, apply post-open regular-file and size checks, and read at most the named byte bound plus one detection byte.

Current integrated-main results:

```text
pueue 1858: cargo test -p mantle --bin mantle ast_grep
PASS: 12 passed; 0 failed
Includes positive parsing/report/attachment paths and negative stale, malformed,
oversized, final-component symlink, and symlinked-parent paths.

pueue 1856: cargo test -p mantle --bin mantle release_capability::tests
PASS: 6 passed; 0 failed

pueue 1877: cargo test -p crunch-release-core --lib ast_grep::tests
PASS: 6 passed; 0 failed

pueue 1875: cargo check -p crunch-release-core --target wasm32-unknown-unknown
PASS with the preinstalled rustup target selected explicitly inside the dev shell.

pueue 1876: cargo clippy -p crunch-release-core --lib --tests -- -D warnings
PASS

pueue 1870: cargo check -p mantle --bin mantle
PASS at the repository's existing warning baseline.

pueue 1870: cargo test -p mantle --bin mantle remote_client_json_report_uses_build_report_schema_and_substitution_fields
PASS: 1 passed; 0 failed

pueue 1861: nix build .#checks.x86_64-linux.ast-grep-package-identity --no-link --print-out-paths -L
PASS: /nix/store/x3jh30yi4av16b6lqc4wvh7jrj2y5vjg-mantle-ast-grep-package-identity-smoke

pueue 1862: targeted rustfmt, nixfmt --check flake.nix, and git diff --check
PASS
```

The authoritative integrated-main lifecycle rerun used the generated Mantle policy:

```text
validate: valid=true, issues=[], changes=16, specs_validated=40
proposal: PASS, issues=[], receipt_hash=bb934844efb565965bb28bcbdb419481ebe5493def4e0fc375c096a428816f31
design: PASS, issues=[], receipt_hash=b12afac2abcb5b4638e20550b6ca46523609aef70beab6dbf90a3f3d88e43f79
tasks: PASS, issues=[], receipt_hash=fc9954753f319f5f003e583c83359dc2957c2160c64da83a25662be90bbfa8a3
```

No Cairn sync or archive command had run at this point.

## Delta-marker repair before sync

The implementation packet used main-spec headings, which would make native sync a no-op. Before lifecycle completion, the change-local spec was repaired to `# Verification Evidence Specification Delta` with `## ADDED Requirements`. The authoritative validation and all three gates passed again:

```text
validate: valid=true, issues=[], changes=16, specs_validated=40
proposal: PASS, issues=[], receipt_hash=f81f83ab5b3be029eed6465a1fd0fc8acc52a93671f0de4655f334ba81080439
design: PASS, issues=[], receipt_hash=93fff3d21ab58233ad6f9bb43046484929cc32295bde340243995dd25ac08235
tasks: PASS, issues=[], receipt_hash=ca8c6aaee454d112bbddac98c43ca9b45d501582f53057eb0868e7c3f8d337dd
```

The native sync dry run now planned one `sync_delta_spec` action for `cairn/specs/verification-evidence/spec.md` with plan hash `b10159816c678db263dab0a1fc440b39ee62c132872adfbc21387fc7ab451ff7`. No mutation or archive had run at this point.

## Executed sync evidence

```text
change: package-ast-grep-structural-rails
verification-evidence before: c48bc581eda865d43d3b310055a511e59a8a4b03abd39974327f6cb56f237219
verification-evidence after: f35ad9ca7a90b6d39ef02c0f5398aa075e76c6c5e7a46552c5c2f741234e14f8
plan_hash: 4b707880e69a75835e100e6c2fdcd189e9f55c1a5abdf9860970e6efce6d92e1
receipt_hash: c58cff8dd6c3d38c53ae1457c05c1dc4e2f1417eca38e40b7a4b8df174b2dadc
mutated: true
blocked: false
```

Pueue task 1894 ran authoritative validation after sync. Exact output:

```text
{
  "change_issues": [],
  "changes": 16,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 40,
  "valid": true
}
```
