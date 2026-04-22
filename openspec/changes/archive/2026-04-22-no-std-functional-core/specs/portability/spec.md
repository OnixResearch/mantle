## ADDED Requirements

### Requirement: First-wave portable core compiles without std

The first-wave portable crunch core MUST treat `#![no_std]` plus `alloc` as the
required compilation floor.

ID: portability.nostd.core.compiles.without.std

For the first extraction wave, `crunch-attestation-core` and
`crunch-project-core` MUST compile without `std`. Their surrounding std crates
MUST own path discovery, file I/O, subprocesses, networking, clocks, and other
host facilities before translating those inputs into plain core values.

The required no-std portability proof for this wave MUST satisfy
`functional.core.nostd.boundary.continuously.verified`, including the target
prerequisite for `wasm32-unknown-unknown`.

#### Scenario: First-wave core compiles on no-std target
ID: portability.nostd.core.compiles.without.std.target

- GIVEN the first-wave core crates
- WHEN the required no-std boundary validation runs
- THEN `crunch-attestation-core` and `crunch-project-core` compile for
  `wasm32-unknown-unknown`
- AND their shell/adaptor crates remain free to use std on supported hosts

### Requirement: First-wave portable core stays inside approved no-std dependency closure

The first-wave portable core MUST limit its dependency closure to the approved
no-std allowlist enforced by
`functional.core.nostd.boundary.continuously.verified`.

ID: portability.nostd.core.dependency.allowlist

Dependencies outside that allowlist belong in std shell layers.

#### Scenario: Dependency audit catches std-only leak
ID: portability.nostd.core.dependency.allowlist.catches.std.leak

- GIVEN a no-std core crate manifest or dependency tree
- WHEN the required no-std boundary validation runs
- THEN std-only runtime crates and direct ambient-effect helpers are absent
  from the first-wave core boundary
- AND any later attempt to add them causes validation to fail
