## ADDED Requirements

### Requirement: Correctness primitives remain frontend-neutral [r[build_tool_boundary.correctness_primitives_frontend_neutral]]

Mantle MUST expose action specs, CAS object refs, reference policies, sandbox reports, receipts, and substitution admission as frontend-neutral build-tool primitives. Frontend artifact kinds, expected refs, and spec refs MAY be supplied as data, but Mantle core MUST NOT interpret Onix roles, tags, settings, providers, upstream exports, Nickel contracts, NixOS modules, or other frontend module-layer semantics to validate those primitives.

#### Scenario: Frontend data is admitted as data [r[build_tool_boundary.correctness_primitives_frontend_neutral.scenario.data]]

- GIVEN a frontend supplies action specs with artifact kind strings, expected refs, and spec refs
- WHEN Mantle validates action correctness primitives
- THEN Mantle MUST treat those fields as data governed by generic schemas and frontend-provided specs
- AND it MUST NOT require frontend module semantics to compute action, object, reference, or receipt identity

#### Scenario: Raw module semantics are rejected [r[build_tool_boundary.correctness_primitives_frontend_neutral.scenario.reject-module-semantics]]

- GIVEN a caller passes raw Onix/NixOS module-layer concepts directly to Mantle's correctness primitive boundary
- WHEN Mantle validates the request
- THEN Mantle MUST fail closed or route to an explicitly external frontend adapter
- AND it MUST NOT silently interpret those concepts as build action inputs
