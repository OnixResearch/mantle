# Nominal dynamic-plan integration validation

## Immutable inputs

- Source commit: `a2c872cfe6826df2d3ca7c85597365e6076fdfba`
- Source tree: `17c155109a663ff3574c4865c3c905ee0e5ee486`
- Source parent: `01dcfc6e05daf7dd56b0f483714bb695291f66ad`
- Source subject: `prevent dynamic plan domain substitution`
- Selected target commit: `bb241ec422bc590693fc5eaab199730b3e4bf2fe`
- Selected target tree: `d070fe6e461f2d0f9c66660fd7d2726eb95496ec`
- Resolved code commit: `a6bb77773f1458d5919a869ac2efc7edd8efaf70`
- Resolved code tree: `74312ae617578ad0b6f9a9c999d0f0d0dfb502ce`

The dedicated branch was `integrate-nominal-dynamic-plan`. The dedicated worktree was `mantle-nominal-dynamic-plan-integration`.

`git cherry-pick a2c872cfe6826df2d3ca7c85597365e6076fdfba` completed without a textual conflict.

Git reported `16 files changed, 1335 insertions(+), 217 deletions(-)` for the resolved code commit.

## Exact changed-file set

The source commit and resolved code commit have the same changed-file set:

- `Cargo.toml`
- `README.md`
- `cairn/archive/1970-01-01-replace-plan-primitive-aliases/design.md`
- `cairn/archive/1970-01-01-replace-plan-primitive-aliases/evidence/baseline/dynamic-plan-v1.json`
- `cairn/archive/1970-01-01-replace-plan-primitive-aliases/evidence/octet/nominal-domain-check.json`
- `cairn/archive/1970-01-01-replace-plan-primitive-aliases/evidence/validation.md`
- `cairn/archive/1970-01-01-replace-plan-primitive-aliases/proposal.md`
- `cairn/archive/1970-01-01-replace-plan-primitive-aliases/specs/build-correctness/spec.md`
- `cairn/archive/1970-01-01-replace-plan-primitive-aliases/tasks.md`
- `cairn/specs/build-correctness/spec.md`
- `crates/crunch-build/src/dynamic_plan.rs`
- `crates/crunch-build/src/dynamic_plan/wire.rs`
- `crates/crunch-build/src/worker.rs`
- `crates/crunch-build/testdata/dynamic-plan-v1-canonical.json`
- `docs/nominal-dynamic-plan-types.md`
- `dylint.toml`

## Conflict audit

The integration kept the source patch as one commit. It did not recreate the patch by hand.

The `Cargo.toml` Octet metadata applied without a conflict. It did not replace the target Tiger Style metadata.

The README kept the target wording and added the source paragraph about nominal dynamic-plan types.

The canonical build-correctness specification kept target requirements. It also added the source nominal-type requirements.

The StageX files stayed byte-identical to the selected target. The audit covered these paths:

- `bootstrap/stagex-transition-lineage.ncl`
- `src/main.rs`
- `src/stagex_musl.rs`
- `src/stagex_musl_native.rs`
- `src/stagex_transition.rs`
- `cairn/changes/materialize-stagex-lineage-provider/`

The source patch did not delete or replace StageX work. It did not change StageX receipt claims.

## Focused baseline and resolved tests

The selected target passed the focused `crunch-build` baseline:

```text
cargo test -p crunch-build --lib
648 passed; 0 failed
cargo test -p crunch-build --test export_api
1 passed; 0 failed
```

The resolved tree passed the focused suite:

```text
cargo test -p crunch-build --lib
652 passed; 0 failed
cargo test -p crunch-build --test export_api
1 passed; 0 failed
cargo test -p crunch-build --doc
1 positive doctest passed; 4 compile-fail doctests passed
```

These results cover valid wire admission and malformed-value rejection. They also cover wrong-role and unknown-root rejection.

The worker tests cover typed unit lookup, output lookup, placeholder resolution, and store-path conversion.

## Canonical compatibility

The frozen baseline is:

`cairn/archive/1970-01-01-replace-plan-primitive-aliases/evidence/baseline/dynamic-plan-v1.json`

The frozen canonical bytes are:

`crates/crunch-build/testdata/dynamic-plan-v1-canonical.json`

The focused test `wire_projection_preserves_frozen_canonical_bytes_and_plan_digest` passed.

The canonical plan BLAKE3 stayed:

`dc6814c1f500dc7e8575c3fd84a64ae78a70d38313ccbbfff4fbfcf7610f6750`

This result proves compatibility for the frozen fixture. It does not prove compatibility for an unknown schema version.

## Source-shape and purity checks

The following structural search found no public primitive string alias:

```text
ast-grep pattern: pub type $A = String;
paths: dynamic_plan.rs, dynamic_plan/wire.rs, worker.rs
result: no matches
```

The following raw-container search found no match in those files:

```text
BTreeMap<String,|HashMap<String,|Vec<String>
```

The pure-core search found no filesystem, environment, process, async, or print effect in `dynamic_plan.rs` or `dynamic_plan/wire.rs`.

Octet found zero instances of these denied lints:

- `primitive_domain_alias`
- `raw_domain_value`
- `newtype_invariant_bypass`

The Octet result is in `evidence/octet-nominal-domain-2026-07-28.json`.

## Quality and Nix checks

Pueue task `3439` passed these commands in the repository development shell:

```text
./scripts/check-first-party-clippy.sh
./scripts/check-first-party-tigerstyle.sh -p crunch-build -- --lib
```

Pueue task `3438` passed:

```text
nix build .#checks.x86_64-linux.fmt .#checks.x86_64-linux.clippy -L --option secret-key-files ''
```

`git diff --check` also passed.

## Broader test triage

`cargo test --workspace --lib --tests` stopped at the known `crunch-eval` standard-library inventory failure.

The selected target reproduces the exact failure. It expects `artifact-auth-cutover-receipt.ncl` in the embedded list.

The resolved `crunch-eval` run passed 79 tests before that failure. No failure named a dynamic-plan path.

The selected target and resolved tree each reported this parallel root-suite summary:

```text
1758 passed; 2 failed; 39 ignored
```

Both runs failed the fake Slurm test. Their second parallel failure differed, which confirms unstable shared-process or lock behavior.

Each dynamic resolved failure passed as an exact serial test. The resolved binary suite then passed serially:

```text
cargo test -p mantle --bin crunch -- --test-threads=1
1760 passed; 0 failed; 39 ignored
```

The serial broader run then reached an example-project assertion. The selected target reproduces that assertion exactly:

```text
production_workflow_projects_evaluate_and_keep_negative_paths
assertion failed: cache.contains("unknown signer is skipped")
```

The integration does not change `crunch-eval`, the fake Slurm test, remote-build locks, or the example-project fixture.

## Claim boundary

This evidence supports source identity, conflict audit, nominal separation, frozen-wire compatibility, and focused test results.

It does not prove store presence, build success, sandbox enforcement, source trust, compiler correctness, or release eligibility.

It does not close the StageX provider-admission change. That change remains active and keeps its existing claim boundary.
