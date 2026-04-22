## MODIFIED Requirements

### Requirement: Adopted portable core compiles without std

The adopted portable crunch core MUST treat `#![no_std]` plus `alloc` as the
required compilation floor.

ID: portability.nostd.core.compiles.without.std

For the adopted no-std waves, `crunch-attestation-core`,
`crunch-project-core`, `crunch-shell-core`, `crunch-release-core`, and
`crunch-delta-core` MUST compile without `std`. Their surrounding std crates or
std root modules MUST own path discovery, file I/O, subprocesses, networking,
clocks, hashing of host files, store/runtime probing, and other host
facilities before translating those inputs into plain core values.

The required no-std portability proof for the adopted waves MUST satisfy
`functional.core.nostd.boundary.continuously.verified`, including the target
prerequisite for `wasm32-unknown-unknown`.

#### Scenario: Adopted cores compile on no-std target
ID: portability.nostd.core.compiles.without.std.target

- GIVEN the adopted no-std core crates
- WHEN the required no-std boundary validation runs
- THEN `crunch-attestation-core`, `crunch-project-core`, `crunch-shell-core`,
  `crunch-release-core`, and `crunch-delta-core` compile for
  `wasm32-unknown-unknown`
- AND their shell/adaptor crates remain free to use std on supported hosts

### Requirement: Adopted portable core stays inside approved no-std dependency closure

The adopted portable core MUST limit its dependency closure to the approved
no-std allowlist enforced by
`functional.core.nostd.boundary.continuously.verified`.

ID: portability.nostd.core.dependency.allowlist

Dependencies outside that allowlist belong in std shell layers.

#### Scenario: Dependency audit catches std-only leak
ID: portability.nostd.core.dependency.allowlist.catches.std.leak

- GIVEN an adopted no-std core crate manifest or dependency tree
- WHEN the required no-std boundary validation runs
- THEN std-only runtime crates and direct ambient-effect helpers are absent
  from the adopted no-std core boundary
- AND any later attempt to add them causes validation to fail
