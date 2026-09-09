# Design: Stable SpaceWasm evidence

## Goal and scope

The producer must emit reproducible stable bundle bytes while retaining exact run diagnostics and required test facts. ChaosControl is the current consumer. This proposal does not implement or admit the repair.

Planning success means a native change package with requirements, ownership, positive and negative validation tasks, and a recorded baseline. Runtime success requires fresh producer execution and independent consumer evidence.

The planning budget covers local source inspection, the retained counterexample, and proposal/design/tasks gates. It excludes new builds, deployments, source publication, and changes to existing policy. Gate success proves package structure only. An existing policy blocker is an allowed planning result.

## Current behavior

`nix/spacewasm-reference.nix` runs Cargo for `upstreamUnitTests` and `upstreamSpectestAddress`. It stores raw stdout and stderr, hashes them into receipts, and copies the receipts and logs into the reference bundle.

`crunch-spacewasm-core` already owns profile admission, check evaluation, bundle planning, canonical identity, and verification. `crunch-spacewasm` and the Nix lane own effects. The repair extends these owners rather than introducing a shared evidence monolith.

The retained counterexample demonstrates variable compilation order, test completion order, and elapsed times under the same derivation. It does not establish that all historical bundle differences have this sole cause.

## Approach review

These are correlated design lenses, not independent audits.

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Refresh pin | Accept the latest raw bundle digest | Rejected: no repeatability guarantee | The retained same-derivation failure already defeats this approach |
| Quiet or serial output | Reduce Cargo progress and test concurrency | Insufficient alone: timing and format remain variable | Repeat actual report producers and compare bytes |
| Stable facts plus raw evidence | Canonicalize admitted facts and retain exact captures separately | Selected direction, implementation unproven | Mutation controls, capture-failure controls, and whole-bundle rebuild comparison |
| Normalize in consumer | Rewrite received bytes before admission | Rejected: changes the consumer trust boundary | Existing mismatch denials must remain unchanged |

## Contract and component ownership

### Stable facts

The core admits an explicit profile, expected test inventory, command identity, process outcome, and bounded parsed observations. Its output carries exact selected test identities and statuses in canonical order. The projection binds source, closure, toolchain, target, features, runner, suite, and encoding version.

The implementation must define which presentation fields do not affect identity. It must not broadly remove digits, sort arbitrary diagnostic lines, or trust only the final summary. Duplicate records are errors, not entries to deduplicate. Required failed or skipped tests remain visible and cannot qualify the cohort.

A structured producer format is preferable when the pinned toolchain supports it. A text adapter requires a closed versioned grammar and denial of unknown or incomplete forms. Selection between these formats requires an ADR and representative fixtures before implementation.

### Raw run evidence

The shell captures exact stdout, stderr, and process outcomes under explicit byte, deadline, and retention limits. A separate run record binds those bytes to the stable bundle and the exact execution contract.

The identity graph is one-way:

```text
run record -> stable bundle identity
run record -> exact raw capture identities
stable bundle -> canonical required facts and artifact members
```

The stable bundle cannot refer back to run identifiers or raw capture hashes. Otherwise nondeterminism reenters through a parent edge or receipt field.

The capture mechanism must preserve raw diagnostics outside deterministic Nix outputs. A secondary nondeterministic Nix output is not a repair. Nix build logs can be an acquisition surface, but they are not a retention guarantee by themselves. The shell must archive and verify required captures before reporting successful publication. Missing capture, truncation, or storage failure blocks publication.

The implementation must record retention authority, archive format, retrieval, completeness, and failure behavior. It must not borrow ambient sibling directories or introduce a product runtime dependency on operator evidence paths.

### Reuse review

First reuse the current core bundle, digest, profile, and result types. Review `bounded-exec` for bounded process capture and `durable-file-publication` for capability-relative publication only where those contracts fit. Their mechanisms do not own report meaning, whole-run retention, or qualification. Any new dependency requires an immutable published pin and compatibility evidence. No new generic port is required for internal canonicalization.

## Compatibility and migration

The accepted `spacewasm-reference-materialization` spec requires exact check identities and complete rehashable bundles. Separating raw evidence changes the artifact contract, not merely JSON formatting.

Before code changes, define a new report/bundle version and the exact required-member sets for both the stable bundle and run archive. Supply explicit modifications to accepted requirements where the new contract changes their meaning. Preserve all existing positive and negative scenarios. Old receipts retain their original bytes, identities, verifier behavior, and non-claims.

ChaosControl adoption is a separate owner action after producer verification and publication. Its profile must bind the new version and measured candidate identities. A changed expected digest is not independent evidence.

## Validation and adversarial review

- Establish focused core, shell, and Nix baselines before changes.
- Preserve test inventory and selection. Pair timing/order variation with changed-outcome controls.
- Reject duplicate, missing, malformed, unknown, over-limit, contradictory, truncated, and invalid-encoding inputs.
- Reject passing text paired with nonzero exit, cancellation, signal, or timeout.
- Prove that capture and retention failures prevent successful publication.
- Rerun all report-producing dependencies in separate scratch roots. Cache reuse of unrelated immutable toolchains is allowed and must be recorded.
- Compare complete stable manifests and member bytes. A bundle assembly rerun over cached reports is insufficient.
- Retain raw capture differences as run evidence, not as a reason to weaken stable comparisons.
- Run independent member verification and negative schema, pin, role, edge, tamper, and non-claim cases.
- Run a frozen ChaosControl differential check against the published candidate before consumer promotion.

## Risks and stop conditions

The main risks are hidden identity cycles, loss of failure evidence, parser overacceptance, incomplete raw retention, and false repeatability through caches. Any unresolved risk blocks implementation acceptance.

Current repository policy validation also has a baseline registry error, recorded in `evidence/baseline.md`. This proposal must not repair that policy incidentally or claim acceptance through a substituted policy.

The maintainer must create a dedicated worktree before implementation. Existing primary changes and long-running source-built proofs remain untouched.
