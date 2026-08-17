## ADDED Requirements

### Requirement: Rust project compatibility gallery [r[examples.rust_project_compatibility_gallery]]

Mantle SHOULD document the representative Rust compatibility rail in the examples gallery with explicit support tier, prerequisites, command, expected output shape, validation rail, and non-claims.

#### Scenario: Gallery identifies the Rust compatibility rail [r[examples.rust_project_compatibility_gallery.scenario.catalog]]

- GIVEN the representative Rust compatibility rail is added or updated
- WHEN the examples catalog and README are checked
- THEN they SHOULD name the fixture or generated example, support tier, required capabilities, validation command, and expected binary/output behavior
- AND they SHOULD distinguish fast local checks from heavyweight or Linux-only integration checks.

#### Scenario: Gallery does not overclaim Cargo compatibility [r[examples.rust_project_compatibility_gallery.scenario.non-claims]]

- GIVEN the Rust compatibility rail appears in user-facing docs
- WHEN the docs describe what the rail proves
- THEN they MUST state whether the evidence came from sandboxed offline Cargo, native rust-plan, or both
- AND they MUST NOT present the rail as proof of full Cargo compatibility, compiler correctness, release reproducibility, or bootstrap correctness.
