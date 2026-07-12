# Trellis proof release-sidecar implementation evidence

## Implemented scope

Mantle now has a pure `crunch-release-core` profile for `kamacite.trellis-proof-evidence-profile.v1` over the generic opaque-evidence binding. It checks canonical Preserves metadata, Valence artifact and logical identities, source/binary links, policy hashes, exact role vocabularies, claim scope, required non-claims, and optional JSON projection identity.

The profile accepts Kamacite producer role `recorded-only` or `formal-proof-candidate` only with Valence validation role `recorded_only`. Optional mode reports valid present evidence as `recorded-only`; required mode fails closed with an explicit missing-acceptance-authority diagnostic.

The core receives typed observations and never parses Verus source, proof IR, verifier logs, or Preserves internals.

## Authority inspected

- Kamacite revision: `de710a092d351e829abfb288d46124e2db8e5b7f`
  - `crates/kamacite-core/src/proof_evidence.rs`: exact canonical schema/projection identities and only `recorded-only` / `formal-proof-candidate` producer roles.
  - `cairn/specs/stack-integration/spec.md`: Kamacite preserves candidate metadata but must not claim downstream proof acceptance.
- Valence revision: `7a027529dd4b7057cf52e86dc5b258f2a9541545`
  - `crates/valence-core/src/stack_role_registry.rs`: shipped verification roles are `property`, `recorded_only`, `boundary`, and `manual_review`.
  - `crates/valence-core/src/formal_proof_chain.rs` and archived formal-proof-chain design: property evidence is Octet-owned; Trellis imports remain recorded-only.
  - `cairn/changes/trellis-proof-evidence-profile/tasks.md`: every task is unchecked, including profile registration, accepted-formal-proof behavior, accepted fixtures, graph output, and final stack validation.
- Trellis revision: `3bf9144b99d65ad0c00776d1d5b81b9c8878c222`
  - `README.md`: exported proof artifacts are local facts/manual-review or reference inputs, not Mantle release eligibility or downstream certification.

## Exact external blocker

A passing required accepted-proof fixture would need an authoritative Valence accepted Trellis validation receipt. No such shipped validator or receipt/profile vocabulary exists at the inspected Valence revision. Valence's proposed `trellis.proof-evidence` change is active and wholly unchecked. Creating a passing Mantle receipt now would invent upstream authority or promote Kamacite's candidate role contrary to both Kamacite and current Valence semantics.

Therefore these remain intentionally incomplete:

1. required accepted-proof positive fixture;
2. accepted-proof completion claim;
3. Trellis -> Kamacite -> Valence -> Mantle accepted-proof stack smoke.

## Baseline evidence

Command (pueue task `279`):

```text
CARGO_TARGET_DIR=/tmp/mantle-trellis-sidecars-baseline-target nix develop --option builders '' -c cargo test -p crunch-release-core --lib
```

Result before core edits:

```text
test result: ok. 179 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

The earlier pueue task `252` is not evidence because it failed before running tests with `cargo: command not found`.

## Focused development evidence

Command (pueue task `375`):

```text
CARGO_TARGET_DIR=/tmp/mantle-trellis-sidecars-baseline-target nix develop --option builders '' -c cargo test -p crunch-release-core --lib trellis_proof -- --nocapture
```

Result:

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 182 filtered out; finished in 0.00s
```

This focused run predates the final negative-matrix additions.

## Post-change core evidence

Command (pueue task `425`):

```text
nix develop --option builders '' -c cargo fmt -p crunch-release-core && git diff --check && CARGO_TARGET_DIR=/tmp/mantle-trellis-sidecars-post-target nix develop --option builders '' -c cargo test -p crunch-release-core --lib
```

Result:

```text
test result: ok. 184 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

No-std target command (pueue task `432`):

```text
PATH=<documented clang+mold+nightly paths> CARGO_TARGET_DIR=/tmp/mantle-trellis-sidecars-host-wasm-target RUSTC=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo check -p crunch-release-core --target wasm32-unknown-unknown
```

Result:

```text
Checking crunch-release-core v0.1.0 (/tmp/mantle-agent-trellis/crates/crunch-release-core)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.73s
```

The first Nix-shell wasm attempt lacked the target, and the first direct-host attempts encountered the configured stale rustc wrapper and then a missing linker. They are not success evidence. Task `432` cleared the wrappers and supplied Mantle's documented clang/mold PATH before compiling successfully.

## Lifecycle evidence

Pueue task `435` ran the following commands under `set -eu` after the implementation and blocker records were present:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal trellis-proof-release-sidecars --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design trellis-proof-release-sidecars --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks trellis-proof-release-sidecars --root .
```

The command sequence completed successfully. The final tasks receipt reported:

```json
{
  "issues": [],
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

The active change is intentionally not ready to archive: the accepted-proof positive and upstream stack-smoke tasks remain unchecked with the exact Valence blocker recorded above.

## Integration hardening

An adversarial VibeThinker review challenged proof-kind dispatch as a possible generic-profile hijack. The integrated implementation now dispatches Trellis validation and claim scope only for the closed `kamacite.trellis-proof-evidence-profile.*` family, rejects an unsupported family version instead of treating it as generic, and preserves unrelated generic `proof` profiles on the generic opaque contract. The negative matrix now also rejects duplicate JSON projections.

Pueue task `497` ran the hardened Trellis-focused tests: 6 passed, 0 failed. Pueue task `506` ran the complete release core and strict core Clippy: 185 tests passed, 0 failed, and Clippy completed with `-D warnings`. The chained Nix-shell wasm leg could not find that shell's wasm target and is not success evidence. Pueue task `513` reran the no-std check with the installed nightly wasm target and cleared wrappers; `cargo check -p crunch-release-core --target wasm32-unknown-unknown` completed successfully.

Pueue task `519` ran repository validation and proposal, design, and tasks gates with `cairn-policy/generated/cairn-policy.json`. Validation reported 9 active changes, 36 specs, no issues, and `valid: true`; all three gates reported no issues, `valid: true`, and `verdict: PASS` under policy hash `d74df84554f5c11df44bab7edd16241150bc70f545bf5b058957516beab43d9c`.
