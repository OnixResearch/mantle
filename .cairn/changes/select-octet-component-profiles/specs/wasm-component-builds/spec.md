# Component Artifact Profile Selection

## ADDED Requirements

### Requirement: Select the declared artifact profile from an exact toolchain cohort

r[mantle.wasm_component.profile_selection] Mantle MUST admit only the exact pinned portable and Molten Octet artifact profiles declared in its generated toolchain cohort, including independent identities produced by actual Octet collection and verification. A pipeline request MUST select its profile by its declared expected artifact-profile identity without fallback. The selected identity MUST bind the real Octet invocation, receipt validation, stage evidence, published validation evidence, portable admission and materialization bundle. Runtime-profile admission MUST remain independent, and the portable default MUST NOT be broadened to admit the Molten import or bulk memory.

#### Scenario: Supported actor world chooses its separate profile
- GIVEN the supported Molten actor-world source and the declared Molten artifact-profile identity
- WHEN the real producer invokes the pinned toolchain
- THEN the Molten profile is used and successful exact-byte validation is consistently bound through the complete consumer-verifiable bundle.

#### Scenario: Portable default remains restrictive
- GIVEN the same actor-world source and the portable artifact-profile identity
- WHEN the producer runs
- THEN independent Octet findings deny portable admission and no materialization bundle is admitted.

#### Scenario: Unknown or malformed profile cohort fails closed
- GIVEN an unknown request identity or a missing, duplicate, stale, misnamed or misbound toolchain profile
- WHEN preflight validates it
- THEN the request fails before component pipeline execution and no default profile is substituted.
