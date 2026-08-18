# Design: Frontend artifact spec admission

## Boundary

Mantle remains a frontend-neutral build tool. A frontend owns its domain model and may provide an artifact spec that describes a deployable or otherwise meaningful output kind. Mantle owns generic admission mechanics: content-addressed spec references, validator dispatch, manifest validation, failure diagnostics, and attestation in build reports.

Mantle must not special-case Onix activation, NixOS systems, roles, tags, providers, upstream exports, or other frontend concepts. Artifact `kind` values are data supplied by the frontend. Mantle may validate that a kind appears in the frontend spec and that the artifact manifest conforms to that spec, but it must not infer semantic support from the kind string alone.

## Spec reference model

A frontend artifact spec reference should include at minimum:

- `id`: stable frontend-owned spec id, such as `onix.activation`;
- `version`: frontend-owned spec version;
- `validator_kind`: supported validation mechanism, such as `nickel`, `rust`, or a later explicitly added kind;
- `validator_ref`: content-addressed ref or build input pointing to the validator/spec material;
- `hash_algorithm`: BLAKE3 by default for new Mantle/Onix spec hashes;
- `hash`: digest of the canonical validator/spec material;
- optional `metadata`: opaque frontend metadata that Mantle preserves but does not interpret.

## Artifact manifest model

A frontend artifact manifest should include:

- frontend `kind` string;
- machine/target or other frontend identity when supplied;
- artifact refs and content hashes;
- declared entrypoints or output roles when the frontend spec requires them;
- provenance links to build inputs and build reports;
- spec reference binding.

Mantle should treat the manifest as typed by the frontend spec. Generic validation checks ensure fields required by the admission contract are present. Spec-specific checks are delegated to the declared validator.

## Validation and attestation

Validation runs before Mantle reports a frontend artifact as spec-admitted. Validation failure is a build/report diagnostic, not partial success. A successful attestation should bind:

- spec id/version/hash;
- validator kind and validator ref/hash;
- artifact ref and artifact digest when available;
- build root or output identity;
- validation result;
- build report hash or enough material to recompute one;
- no hidden fallback marker.

Attestation fields should be included in JSON build reports and in artifact sidecar/receipt material when available.

## Failure behavior

Mantle must fail closed or mark the artifact not admitted when:

- the spec reference is missing or malformed;
- the validator kind is unsupported;
- the spec hash does not match the validator/spec input;
- validation fails;
- the artifact manifest claims a frontend kind without a spec binding;
- core code attempts to interpret a frontend-specific kind as a built-in Mantle kind.

## Verification strategy

- Positive test: a minimal frontend spec validates a minimal artifact manifest and emits report attestation.
- Negative tests: missing spec ref, unsupported validator kind, hash mismatch, malformed manifest, and frontend kind without spec binding fail with deterministic diagnostics.
- Boundary test: Onix-like artifact kind strings are accepted only as data under a validating spec and are never interpreted by Mantle core.
- Report test: JSON build/report output carries spec id, version, hash, validator kind, artifact ref, and validation result.

## Requirement trace

- r[build_tool_boundary.frontend_artifact_spec_admission]
- r[build_tool_boundary.spec_validation_attestation]
- r[build_tool_boundary.frontend_agnostic_artifact_kinds]
- r[verification_evidence.spec_admission_proof_before_claim]
