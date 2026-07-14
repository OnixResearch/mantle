# Mantle Nickel Export Core Cutover Specification

## Purpose

Adopt the immutable standalone evaluator-neutral Nickel export core while preserving Mantle's embedded evaluator, sandbox, destination, build, and release authority.

## Requirements

### Requirement: Standalone core source is immutable

r[mantle.nickel_export_cutover.source] Mantle MUST select `nickel-export-core` from `github.com/OnixResearch/nickel-export` revision `257fafc1c746f1faf156207043a4c826bfb16d49` in release dependency and lock material.

#### Scenario: Exact source is selected
- GIVEN Cargo, Nix, and lock material resolve the declared revision
- WHEN the cutover readiness check runs
- THEN Mantle MAY use the standalone core for compatibility evaluation.

#### Scenario: Source floats or is overridden
- GIVEN the dependency uses a branch, mutable tag, workspace-relative release path, or different revision
- WHEN readiness is evaluated
- THEN Mantle MUST fail before accepting cutover evidence.

### Requirement: Embedded evaluator authority remains in Mantle

r[mantle.nickel_export_cutover.boundary] Mantle MUST keep `crunch-eval`, import resolution, filesystem roots, evaluator diagnostics, and output writes in a Mantle-owned imperative shell and MUST pass only explicit in-memory observations into the standalone core.

#### Scenario: Embedded evaluation succeeds
- GIVEN Mantle has authorized roots and produced source, dependency, evaluator, diagnostic, and output observations
- WHEN the adapter invokes the core
- THEN the core MAY normalize and admit those explicit observations without performing I/O.

#### Scenario: Shared code requests ambient authority
- GIVEN a proposed adapter asks the core to read files, resolve imports, execute Nickel, inspect environment state, or write output
- WHEN boundary validation runs
- THEN Mantle MUST reject the adapter.

### Requirement: Mantle product authority is retained

r[mantle.nickel_export_cutover.authority] Mantle MUST retain sandbox/root admission, destination ownership, build evidence, receipt policy, and release eligibility decisions independently of standalone-core admission.

#### Scenario: Canonical receipt is admitted
- GIVEN the standalone core admits an exact evaluator observation
- WHEN Mantle evaluates build or release policy
- THEN Mantle MUST still apply every local policy and gate.

#### Scenario: Receipt is promoted into a build claim
- GIVEN only a canonical or projected export receipt
- WHEN a caller requests build success or release eligibility
- THEN Mantle MUST reject the unsupported promotion.

### Requirement: Cutover dual-runs shared facts

r[mantle.nickel_export_cutover.dual_run] Mantle MUST run legacy and canonical admission over identical source, dependency, evaluator, diagnostic, and output observations and MUST compare canonical identity, `mantle-nickel-export-receipt-v1`, path failures, and evaluator diagnostics.

#### Scenario: Positive observations agree
- GIVEN identical valid observations are supplied to both paths
- WHEN comparison runs
- THEN exact canonical identities and Mantle v1 projection fields MUST agree.

#### Scenario: Negative observations differ
- GIVEN path confinement or evaluator diagnostics produce different failure classes or a success receipt appears on one failing path
- WHEN comparison runs
- THEN cutover MUST fail and preserve bounded drift evidence.

### Requirement: Rollback remains authoritative until parity is explained

r[mantle.nickel_export_cutover.rollback] Mantle MUST keep the legacy implementation authoritative through one complete positive and negative validation cycle and MUST block cutover on unexplained drift.

#### Scenario: Validation cycle completes
- GIVEN all declared parity fixtures and gates pass at the immutable revision
- WHEN authority switches to the canonical path
- THEN Mantle MAY retain the legacy implementation as a bounded rollback adapter.

#### Scenario: Drift is detected
- GIVEN any byte, identity, receipt, diagnostic, or failure-class difference
- WHEN rollback is applied
- THEN the legacy path MUST remain authoritative and paired regression fixtures MUST be required before retry.

### Requirement: Cutover validation is positive, negative, and bounded

r[mantle.nickel_export_cutover.validation] Mantle MUST validate exact-source selection, canonical identity, Mantle projection, embedded-evaluator diagnostics, path confinement, stale outputs, tampering, mixed evaluators, and unsupported claim promotion without claiming evaluator or build correctness.

#### Scenario: Complete validation passes
- GIVEN positive and negative fixtures, focused tests, machine-contract freshness, Nix checks, Cairn gates, and release evidence pass
- WHEN duplicated local core logic is reviewed
- THEN Mantle MAY remove only logic delegated to the standalone core.

#### Scenario: Validation is incomplete
- GIVEN a required negative fixture, gate, lock check, or release non-claim is absent
- WHEN closeout is evaluated
- THEN the change MUST remain active and the duplicate logic MUST remain available for rollback.
