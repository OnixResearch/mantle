# Completion status: P2P remote builders

Date: 2026-07-01

## Current state

The change is ready to validate and archive as the deterministic Mantle remote-builder foundation:

- versioned `mantle-remote-build/1` frame protocol with explicit state-machine validation;
- loopback and stdio/SSH-stdio-compatible bindings with stdout-as-wire discipline;
- ticket state, delayed redemption, secret-safe list/inspect/reveal/revoke behavior, and invalid-ticket negative coverage;
- concrete action/derivation request planning after local evaluation/lowering, with raw frontend/Nickel request rejection;
- bounded missing-input negotiation, inline source-input NAR upload artifacts, digest checks, and remote-store materialization;
- executor seam with fixture and Linux local-build executor paths;
- signed PathInfo, artifact-attestation digest, and bounded NAR output transfer artifacts;
- durable client-side output import through `StoreHandle`, artifact attestation sidecars, exported outputs, and substitution reports;
- `mantle build --builder <endpoint> --ticket <id:secret>` stdio dispatch for locally lowered concrete derivations;
- coordinator worker registration, capability/trust matching, normalized build-key dedupe, conflict rejection, resumable redelivery, bounded log replay, redacted status snapshots, stable reconnect decisions, and session-lease release decisions;
- `mantle remote status` for redacted status snapshots;
- remote client JSON output rendered as the existing `crunch-build-report-v1` shape with artifact attestation references plus substitution mode/byte/fallback fields.

## Non-claims

This change does not claim a production Iroh/libp2p transport or unbounded streaming of arbitrary CAS objects. The production P2P binding remains a later transport adapter that must reuse the same `RemoteFrame` protocol core and trust boundaries. The current transfer surface is intentionally bounded for deterministic local and stdio validation.

## Evidence

Primary evidence for the final coordinator/status/reporting slice is recorded in:

- `cairn/changes/p2p-remote-builders/evidence/coordinator-status-report-slice-2026-07-01.md`

Earlier slice evidence remains in the sibling files under this `evidence/` directory.

## Next best step

Run Cairn validate and proposal/design/tasks gates, then sync and archive the change if validation remains green.
