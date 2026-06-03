## ADDED Requirements

### Requirement: Mantle exports admitted frontend artifacts generically [r[build_tool_boundary.admitted_artifact_fetch_export]]

Mantle MUST provide a frontend-neutral fetch or export boundary for artifacts that have been admitted through a frontend artifact spec. The boundary MUST resolve artifact refs, verify admission evidence, materialize artifact contents, and preserve provenance without hard-coding frontend artifact semantics.

#### Scenario: Spec-admitted artifact exports with proof [r[build_tool_boundary.admitted_artifact_fetch_export.scenario.valid]]

- GIVEN a build report or receipt contains an artifact ref with successful frontend spec-admission attestation
- WHEN a caller requests export for that artifact ref
- THEN Mantle MUST verify that the artifact ref and requested spec identity match the admission proof
- AND Mantle MUST return materialized artifact contents with a receipt or sidecar binding artifact ref, content digest, spec proof, and provenance.

#### Scenario: Missing admission proof fails before export [r[build_tool_boundary.admitted_artifact_fetch_export.scenario.missing-proof]]

- GIVEN a caller requests export for a frontend artifact ref without successful spec-admission attestation
- WHEN Mantle validates the export request
- THEN Mantle MUST fail closed before reading or materializing artifact content
- AND diagnostics MUST state that admitted-artifact proof is missing.

#### Scenario: Mantle refs do not require Nix store paths [r[build_tool_boundary.admitted_artifact_fetch_export.scenario.no-nix-store-requirement]]

- GIVEN a caller requests export of a `mantle://...` artifact ref
- WHEN Mantle resolves the artifact
- THEN Mantle MUST NOT require the ref to be a `/nix/store` path
- AND Mantle MUST NOT silently fall back to `nix copy` or another undeclared ambient Nix runtime command.

#### Scenario: Frontend artifact kind remains opaque [r[build_tool_boundary.admitted_artifact_fetch_export.scenario.frontend-kind-opaque]]

- GIVEN an admitted artifact manifest uses a frontend-owned kind such as `mantle-onix-activation-closure`
- WHEN Mantle exports the artifact
- THEN Mantle MUST treat the kind as opaque data governed by the frontend spec
- AND Mantle MUST NOT interpret Onix activation, roles, tags, providers, inventory, or deploy policy in core.
