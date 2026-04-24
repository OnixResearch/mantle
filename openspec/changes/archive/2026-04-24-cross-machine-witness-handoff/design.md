## Context

The repo can already verify a local witnessed-self-hosting workflow, but the
file handoff between publisher and witness is still implicit. A real second
environment needs three public things from the publisher:

- the verified release-evidence bundle,
- the signed release-attestation seed, and
- enough deterministic metadata to know what request layout was exported.

After the witness rebuilds, the publisher needs the returned `witnesses/*.json`
plus `.sig` sidecars copied back into the destination verification directory.
Doing those steps with ad hoc file copies is both error-prone and hard to
document as the checked-in path toward a publishable external witness result.

## Goals / Non-Goals

**Goals**
- Export one portable witness-request directory from verified public artifacts.
- Import returned witness sidecars with fail-closed validation.
- Reuse existing release and witness attestation formats.
- Document one narrow publisher -> witness -> publisher flow.
- Add positive and negative integration coverage around the handoff.

**Non-Goals**
- Add networking, hosted witness discovery, or transparency logs.
- Move private key material across environments.
- Replace `release attest`, `witness-create`, or `release-verify`.
- Claim that a same-host simulation equals real-world social independence.

## Decisions

### 1. Put request export under `crunch release`

**Choice:** add `crunch release witness-export`.

**Rationale:** the exported request directory is derived from a verified
release-evidence bundle and a signed release attestation. It starts from the
publisher's release material and should therefore live with the other release
bundle lifecycle commands.

**Alternative:** add `crunch attest witness-export`.

**Why not:** the exported request always includes the release-evidence bundle,
not only attestation files. Grouping it under `attest` would hide the release
bundle dependency.

### 2. Put witness import under `crunch attest`

**Choice:** add `crunch attest witness-import <verification-dir> <source>`.

**Rationale:** import mutates the publisher's verification directory and works
on witness-attestation sidecars. That is the same surface already managed by
`witness-create`, `witness-show`, `policy-init`, and `release-verify`.

### 3. Use a deterministic directory layout instead of a tarball-only format

**Choice:** export a directory tree with three fixed pieces:
- `request.json`
- `release-evidence/<release-id>/...`
- `release-verification/<release-id>/release-attestation.json(.sig)`

`request.json` will use a fixed schema token `crunch-witness-request-v1`.
In this change the token is operator-facing metadata, not a parsed protocol
input: export writes it, docs show it, and tests pin it. No current command
accepts arbitrary request versions, so "unknown version" handling is simply
out of scope until a later request-consuming CLI exists.

**Rationale:** the current CLI already works on directories, and a visible tree
is easier to inspect, copy, and test. Operators can still archive the exported
directory with ordinary tools later.

**Alternative:** emit a tarball directly.

**Why not:** that would add format handling now without improving the trust
boundary. The main missing piece is deterministic layout, not compression.

### 4. Validate import against the destination release digest before copying

**Choice:** `witness-import` reads the destination release attestation digest,
parses each returned witness attestation, and rejects any witness whose
`release_attestation_digest` differs.

**Rationale:** importing first and discovering the mismatch only at later
`release-verify` time is too lax. The import boundary should fail fast on
obviously wrong witness material.

### 5. Treat exact duplicate witness material as idempotent, not conflicting

**Choice:** if the destination already contains `witnesses/<identity>.json` and
`.sig` whose bytes exactly match the import, the command returns success without
rewriting them. If either file differs, the command fails.

**Rationale:** this keeps repeated imports safe and boring while still
protecting the destination from silent overwrites.

### 6. Keep the request bundle public-only

**Choice:** the export command copies only verified release evidence,
release-attestation sidecars, and request metadata.

**Rationale:** witness operators need public verification material, not the
publisher's private signing key or verifier-local trust config. Keeping the
exported directory public-only makes the boundary auditable and easier to share.

### 7. Emit summary JSON objects for the new mutating commands

**Choice:** `release witness-export` and `attest witness-import` will follow the
existing mutating-command convention: human-readable summaries by default and
small JSON summary objects under global `--json` with a stable `kind` field.
They will not emit attestation-envelope JSON because they create and move
workflow artifacts rather than render a single canonical attestation body.
Errors keep the repo's existing CLI convention: the command exits non-zero and
prints the error text through the standard CLI error path rather than inventing
a new per-command error schema.

**Rationale:** this keeps output consistent with `release attest`,
`attest policy-init`, and `attest witness-create` while staying honest about
what these commands return.

## Request metadata

`request.json` will carry deterministic compact JSON with these exact fields:
- `schema: "crunch-witness-request-v1"`
- `request_layout_version: 1`
- `release_id: <string>`
- `release_bundle_relative_path: "release-evidence/<release-id>"`
- `verification_seed_relative_path: "release-verification/<release-id>"`

This file is not a new attestation. It is only an exchange manifest for the
portable handoff layout.

## Data flow

```text
publisher
  verified release bundle
    -> `crunch release attest`
    -> publisher verification dir
    -> `crunch release witness-export`
    -> portable request dir

witness
  request dir
    -> `crunch release verify release-evidence/<release-id>`
    -> independent rebuild
    -> `crunch attest witness-create release-verification/<release-id> ...`
    -> returned witness sidecars

publisher
  returned witness sidecars
    -> `crunch attest witness-import <publisher-verification-dir> <source>`
    -> `crunch attest release-verify ...`
    -> technical class + policy status + final class
```

## Verification strategy

- export integration tests for happy-path layout and release-id mismatch
- import integration tests for happy path, missing signature, wrong release
  digest, and conflicting duplicate identity
- end-to-end cross-machine simulation test that uses two temp working dirs,
  separate `CRUNCH_CONFIG_DIR`s, export -> witness-create-from-request ->
  import -> final `release-verify`
- docs update proving the bounded cross-machine workflow is the checked-in path

## Risks / Trade-offs

**Directory export is larger than a tarball.**
That is acceptable for the first workflow slice. Inspectability matters more
than compact transport.

**A same-host integration test is still only a simulation.**
Correct. The docs and evidence must keep the social-independence claim bounded.

**Import validation adds more filesystem cases.**
That is worth it because witness sidecars become a shared trust boundary rather
than an ad hoc copy step.
