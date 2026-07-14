# Trellis proof release-sidecar implementation evidence

## Implemented scope

Mantle now has a pure `crunch-release-core` profile for `kamacite.trellis-proof-evidence-profile.v1` over the generic opaque-evidence binding. It checks canonical Preserves metadata, Valence artifact and logical identities, source/binary links, policy hashes, exact role vocabularies, claim scope, required non-claims, and optional JSON projection identity.

The profile accepts recorded evidence only with Valence role `recorded_only`, and accepts required evidence only for the exact Kamacite `formal-proof-candidate` plus Valence `property` pair. Optional recorded evidence reports `recorded-only`; required accepted evidence reports `accepted-formal-proof`; every other pair fails closed.

The core receives typed observations and never parses Verus source, proof IR, verifier logs, or Preserves internals.

## Authority inspected

- Kamacite revision: `de710a092d351e829abfb288d46124e2db8e5b7f`
  - `crates/kamacite-core/src/proof_evidence.rs`: exact canonical schema/projection identities and only `recorded-only` / `formal-proof-candidate` producer roles.
  - Candidate status does not itself imply downstream promotion.
- Valence commit: `27b8b212`
  - `cairn/archive/2026-07-12-trellis-proof-evidence-profile/`: all validator, role, graph, positive, negative, and final-validation tasks are archived complete.
  - `crates/valence-core/src/trellis_proof_evidence.rs`: `accepted_formal_proof` requires passed verifier status, policy acceptance, and verification role `property`; candidates cannot use `property`.
  - The exact required non-claim keeps acceptance bounded to scoped proof identity and linkage rather than downstream correctness or release eligibility.
- Trellis revision: `3bf9144b99d65ad0c00776d1d5b81b9c8878c222`
  - `README.md`: exported proof artifacts remain bounded local proof facts; Mantle does not infer broader correctness or certification.

## Resolved external dependency

The prior Valence blocker is resolved by archived commit `27b8b212`. Mantle now permits only the exact Kamacite `formal-proof-candidate` plus Valence `property` pair to satisfy required mode. Existing `recorded_only` evidence remains recorded-only, and `property` attached to a Kamacite `recorded-only` producer fails closed.

The accepted binding still proves only that measured release artifacts link to the declared Valence-accepted evidence. It does not make Mantle a proof verifier or establish release eligibility.

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

At that historical checkpoint the active change remained unready because the Valence accepted-proof authority had not yet shipped. The dependency was later resolved as recorded below.

## Integration hardening

An adversarial VibeThinker review challenged proof-kind dispatch as a possible generic-profile hijack. The integrated implementation now dispatches Trellis validation and claim scope only for the closed `kamacite.trellis-proof-evidence-profile.*` family, rejects an unsupported family version instead of treating it as generic, and preserves unrelated generic `proof` profiles on the generic opaque contract. The negative matrix now also rejects duplicate JSON projections.

Pueue task `497` ran the hardened Trellis-focused tests: 6 passed, 0 failed. Pueue task `506` ran the complete release core and strict core Clippy: 185 tests passed, 0 failed, and Clippy completed with `-D warnings`. The chained Nix-shell wasm leg could not find that shell's wasm target and is not success evidence. Pueue task `513` reran the no-std check with the installed nightly wasm target and cleared wrappers; `cargo check -p crunch-release-core --target wasm32-unknown-unknown` completed successfully.

Pueue task `519` ran repository validation and proposal, design, and tasks gates with `cairn-policy/generated/cairn-policy.json`. Validation reported 9 active changes, 36 specs, no issues, and `valid: true`; all three gates reported no issues, `valid: true`, and `verdict: PASS` under policy hash `d74df84554f5c11df44bab7edd16241150bc70f545bf5b058957516beab43d9c`.

## Accepted-proof completion evidence

On 2026-07-14, Valence focused stack authority passed from the clean Valence checkout:

```text
nix develop -c cargo test -p valence-core trellis_proof_evidence -- --nocapture
```

Result: 2 passed, 0 failed. This includes accepted proof plus graph output and the negative missing-assumptions/promotion/domain/boundary case.

Mantle established a before-change focused baseline of 3 passing Trellis-filtered release-core tests. After the exact role-pair extension and accepted fixture were added, the focused rail passed 7 tests:

```text
CARGO_INCREMENTAL=0 nix develop -c cargo test -p crunch-release-core --lib trellis_proof -- --nocapture
```

The complete release-core and strict lint rails then passed:

```text
CARGO_INCREMENTAL=0 nix develop -c cargo test -p crunch-release-core --lib
CARGO_INCREMENTAL=0 nix develop -c cargo clippy -p crunch-release-core --all-targets --no-deps -- -D warnings
nix develop -c cargo fmt -p crunch-release-core -- --check
```

Result: 207 tests passed, 0 failed; Clippy and formatting passed. The positive fixtures cover optional recorded-only and required accepted-formal-proof behavior. The negative matrix rejects unauthorized producer/validator pairs as well as stale, malformed, missing, projection-drifted, and overclaiming evidence.

Final Cairn validation reported 8 active changes, 31 specs, no issues, and `valid: true`. Proposal, design, and tasks gates passed with receipt hashes `11bf2be35e1e224307e135c22814d3bcd17e25a2cac53557bec9991ac78e82cc`, `7394e2fe974d54a45302ac1783f0374fa5cbaaa31c781c7cd2697ed98e837e76`, and `b7e6f5892657ea1c322a7624283a3de77b89aaaa80af4d4cd66bbdd7ba6347f5`. After sync, `mantle-default` Tracey coverage passed 140/140 with no missing or dangling references and receipt `335cd283d488ab7655a30d82d3829a48299226eb13d9469dc962d347054b8212`.
