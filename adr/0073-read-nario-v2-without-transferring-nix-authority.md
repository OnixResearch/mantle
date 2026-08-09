# ADR 0073: Read Nario v2 without transferring Nix authority

## Status

Accepted

## Context

Determinate Nix can export store metadata and NAR payloads in Nario v2 archives. Mantle needs a bounded path to inspect and import those bytes.

A transport archive does not contain package recipes or complete frontend meaning. Original Nix signatures also authorize original Nix paths only.

## Decision Drivers

- Bind compatibility to one reviewed producer revision.
- Keep the Mantle-native archive format as the default.
- Validate complete archives before PathInfo publication.
- Keep archive resource use bounded.
- Preserve original-path trust without transferring it to target identities.
- Reject built outputs as source requirements.
- Keep export outside the first compatibility boundary.

## Decision

Mantle supports Nario v2 list and import for Determinate Nix 3.12.0 revision `9512828397f684d0f732ea76b7631f69a0db34f7`.

The reader accepts the exact v2 magic and WorkerProto v16 metadata shape. It checks record state, limits, NAR identity, references, signatures, content-address metadata, logical prefix, and complete termination.

Import stages castore payloads before publication. Redb and LRU PathInfo backends provide one atomic batch operation. Other backends reject batch publication.

Foreign source preparation accepts a Nario record only when its original path matches one exact source requirement. It then uses the existing source-bundle canonicalizer for the target identity.

Source evidence binds the producer, archive, original PathInfo, trust policy, plan, requirement, target path, and target content identity.

Nario export remains unsupported.

## Alternatives Considered

### Treat Nario as a package format

Rejected. Nario carries store data, not complete recipes or frontend meaning.

### Transfer original signatures to target paths

Rejected. A signature over a Nix PathInfo does not authorize a recomputed Mantle target.

### Publish each record immediately

Rejected. A later malformed record would leave a partial archive admission.

### Add Nario export now

Rejected. Export needs a separate producer contract and compatibility review.

## Consequences

- Compatibility is narrow, versioned, read-only, and explicit.
- Failed archive validation does not publish PathInfo records.
- Direct import preserves `/nix/store` identities.
- Source projection creates new target identities with separate evidence.
- The boundary does not prove recipes, evaluation, correctness, reproducibility, or release eligibility.
