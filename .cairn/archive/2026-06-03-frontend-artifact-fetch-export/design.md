# Design: Frontend artifact fetch/export

## Boundary

Mantle owns build-tool artifact storage and retrieval. A frontend owns artifact semantics. The fetch/export boundary therefore accepts spec-admitted artifact identity and returns materialized contents plus evidence, but it does not interpret frontend artifact `kind` strings beyond the generic admission contract.

The primary consumer motivating this change is Onix, which needs to fetch `mantle://...` artifacts that validated against `onix.activation.v1`. Mantle must not learn Onix activation entrypoints, safe-mode policy, machine roles, tags, providers, or inventory rules. Onix will invoke its own activation entrypoints after Mantle exports the admitted artifact.

## Request model

An admitted artifact export request should include:

- artifact ref, such as `mantle://blake3/...`;
- expected artifact hash or digest when available;
- spec-admission attestation reference or inline attestation;
- expected spec id/version/hash when the caller wants a bounded proof;
- destination mode, such as directory export, archive export, or byte stream;
- optional frontend metadata preserved as opaque data.

Requests must be pure data. Mantle should validate the request before storage access or mutation.

## Result model

A successful export should report:

- materialized path, archive path, or stream metadata;
- artifact ref and digest actually exported;
- spec id/version/hash and validator identity from the accepted admission proof;
- build report or provenance hash;
- export receipt hash using BLAKE3 unless a caller-specified protocol requires another hash;
- no-hidden-fallback marker showing that the export did not use an ambient Nix runtime fallback.

## Fetch behavior

Mantle should resolve only artifact refs that were produced or admitted by Mantle's build-tool boundary. For `mantle://...` refs, the export path must use Mantle storage or a configured Mantle artifact backend. It must not require the ref to be a `/nix/store` path and must not shell out to `nix copy` as an implicit fallback.

Snix/castore is useful prior art: content-addressed blob/directory services, object-store URLs, and gRPC backends map well to Mantle's needs. This change does not make Snix a dependency. If Mantle later chooses a Snix-compatible backend, that should remain an implementation detail below this generic boundary.

## Failure behavior

Mantle must fail closed before exporting content when:

- the artifact ref is missing or uses an unsupported scheme;
- the artifact was not spec-admitted;
- attestation spec identity or artifact ref does not match the request;
- content is missing or digest verification fails;
- the export would require an undeclared ambient Nix runtime fallback;
- core code attempts to interpret an Onix-like artifact kind as a Mantle built-in.

## Verification strategy

- Positive test: a minimal admitted frontend artifact is exported and returns a receipt binding artifact ref, spec proof, content digest, and provenance.
- Negative tests: missing attestation, unsupported ref scheme, wrong artifact ref, digest mismatch, missing content, and Nix fallback marker all fail before export.
- Boundary test: an Onix-like `mantle-onix-activation-closure` kind remains opaque data under a validating spec.
- CLI/API test: a caller can request export by artifact ref without passing or receiving `/nix/store` paths.

## Requirement trace

- r[build_tool_boundary.admitted_artifact_fetch_export]
- r[verification_evidence.admitted_artifact_export_proof_before_claim]
