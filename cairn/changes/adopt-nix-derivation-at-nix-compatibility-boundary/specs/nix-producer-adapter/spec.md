# Nix Producer Adapter Delta

## ADDED Requirements

### Requirement: Reviewed derivation admission for producer output

r[nix_producer_adapter.reviewed_derivation_admission] Every backend-produced concrete Nix `.drv` closure MUST enter foreign graph production through the reviewed Mantle `nix-derivation` adapter after shell-owned byte, closure, path, and process bounds pass.

#### Scenario: Backend closure is admitted

- GIVEN a supported backend produced a bounded complete Nix `.drv` closure in an owned directory
- WHEN the producer adapter admits the closure
- THEN it MUST use the reviewed compatibility adapter and Nix-required identity algorithms
- AND later graph production MUST remain independent of the backend, Nix daemon, and evaluator

#### Scenario: Closure contains an unsupported or malformed derivation

- GIVEN a backend closure contains malformed bytes, an invalid logical name, an unsupported parsed form, a missing reachable input, or a wrong-domain digest
- WHEN producer admission runs
- THEN it MUST fail with the shared stable producer error class before artifact publication
- AND it MUST NOT emit a partial graph, package index, or success receipt

#### Scenario: Mantle-native hashing reaches producer compatibility code

- GIVEN a producer path attempts to use Mantle BLAKE3 derivation hashing or a configurable native prefix as original Nix `.drv` identity
- WHEN hash-domain validation runs
- THEN it MUST reject the identity before graph publication
- AND the diagnostic MUST name the required Nix compatibility domain

### Requirement: Producer compatibility evidence stays bounded

r[nix_producer_adapter.nix_derivation_evidence_boundary] Producer evidence MUST bind the exact `nix-derivation` package, adapter, Nix version, fixture corpus, parity result, and non-claims without promoting parser agreement to evaluator or build equivalence.

#### Scenario: Producer parity passes

- GIVEN backend output and explicit `.drv` fixtures pass the reviewed adapter corpus
- WHEN Mantle records compatibility evidence
- THEN the evidence MUST identify the exact covered inputs and agreement classes
- AND it MUST NOT claim arbitrary Nix compatibility, evaluator parity, build success, or output correctness

#### Scenario: Dependency or corpus changes

- GIVEN the dependency, adapter, Nix version, or required fixture corpus changes
- WHEN old producer evidence is evaluated
- THEN the evidence MUST be stale for the changed surface
- AND it MUST NOT authorize a new cutover without rerun parity
