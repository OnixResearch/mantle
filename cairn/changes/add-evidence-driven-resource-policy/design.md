# Design: Evidence-driven remote resource policy

## Context

Mantle already receives resource declarations and owns resource leases. The new policy adds observations and selection. It does not make observations authoritative, bypass admission, or change the meaning of a declared requirement.

Every decision is a pure result over explicit inputs. The evidence records those inputs, the policy version, the result, and a reason code so that another process can replay the decision.

## Resource observations

A terminal attempt can produce a bounded observation containing:

- public attempt, action-family, platform, and machine-class identities;
- declared and selected resource quantities;
- queue, execution, and terminal timing facts;
- bounded CPU, memory, I/O, and transfer measurements;
- cgroup or equivalent OOM evidence category;
- terminal outcome and retry linkage;
- collector and schema versions.

Action-family identity is a BLAKE3 identity over canonical public request features chosen by policy. It excludes project secrets, token claims, environment values, and log content.

Observations are not resource promises. Missing, stale, incompatible, outlier, or untrusted observations are excluded by explicit policy and reason code.

## Selection core

The pure selector accepts:

- declared minimum requirements and required platform features;
- eligible OnixOS machine classes and current availability facts;
- project and account quota facts;
- compatible historical observations;
- a versioned policy with named margins, sample thresholds, age limits, and fallback rules.

It returns one selected class or a typed rejection plus ordered reason codes. Ties use canonical stable ordering. The same canonical inputs and policy version produce the same result.

Declared minima are hard lower bounds. Historical data can increase a request and can choose among eligible classes. It cannot remove required architecture, platform, KVM, trust, or isolation features.

## OOM recovery

A failed attempt can escalate only when the worker supplies positive, trusted OOM evidence. Exit status alone is insufficient unless the platform contract defines and verifies an unambiguous OOM status.

The pure retry planner receives the prior decision, OOM evidence, retry history, eligible classes, quota facts, and versioned escalation policy. It either chooses a strictly larger eligible class or returns a terminal reason.

The policy has named limits for retry count, largest class, cumulative charge, and wall time. A retry is a new fenced attempt linked to its predecessor. Non-OOM failures, ambiguous termination, policy denial, exhausted limits, or unavailable larger classes do not retry.

## Usage accounting and quotas

Admission reserves usage against explicit project and account ledgers. Completion reconciles the reservation with bounded observed usage under a versioned unit schedule.

Ledger operations are idempotent and reference attempt identity. Duplicate completion does not double-charge. Cancellation, worker loss, partial observation, and accounting-store failure have explicit reconciliation states. A failed accounting mutation cannot silently grant more quota.

API responses expose public units, windows, reservations, charges, and reason codes. Pricing, if added later, is a separate policy surface.

## Authorized result sharing

Default result discovery is private to its project. Policy can declare a sharing scope with explicit producers and consumers. Reuse still requires:

- matching canonical request and action identities;
- a trusted producer signature;
- compatible policy and platform identities;
- authorized producer and consumer scopes;
- verified output identities and current CAS availability;
- a recorded producer-to-consumer evidence link.

Shared CAS objects do not imply result authority. A consumer denial does not delete or corrupt the producer record.

## Benchmarks

Mantle adds a bounded benchmark corpus based on reviewed workload patterns from `nixbench` and representative Onix builds. Each workload records source and license identity, fixed inputs, platform requirements, expected completion class, and measurement bounds.

Benchmarks compare policy versions against a declared static baseline. Reports separate correctness, compatibility, throughput, latency, memory, transfer, and cost-unit observations. No unpublished hosted-service limit or proprietary image is copied.

## Fault validation

ChaosControl exercises positive OOM evidence, ambiguous failures, worker loss, delayed completion, duplicate events, accounting interruption, CAS unavailability, and retry replay. Fault evidence is scoped to the declared harness and schedule.

## Evidence and non-claims

Valence links request, selection, observation, retry, accounting, result, producer, and consumer identities. Cairn gates require policy replay, fixture coverage, and benchmark comparison.

The evidence does not prove future resource sufficiency, whole-build correctness, host isolation, fair billing, or production performance outside tested inputs.

## Rollout

The selector first runs in observe-only mode beside the existing static policy. Differences are recorded but do not affect scheduling. After replay and benchmark gates pass, selected projects can opt in. OOM retry, quota enforcement, and cross-project sharing each require separate feature enablement and rollback switches.
