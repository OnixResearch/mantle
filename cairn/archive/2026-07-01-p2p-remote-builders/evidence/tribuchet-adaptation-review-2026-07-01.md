# Tribuchet adaptation review — 2026-07-01

## Question

What should Mantle adapt from `https://github.com/Mic92/tribuchet` for the active `p2p-remote-builders` change?

## Inspected evidence

- Tribuchet README from `https://github.com/Mic92/tribuchet`.
- Tribuchet design document from `https://github.com/Mic92/tribuchet/blob/main/DESIGN.md`.
- Existing Mantle active change: `cairn/changes/p2p-remote-builders/`.
- Existing Mantle reference list in `README.md`.

## Decision

Adapt Tribuchet's architecture patterns, not its Nix-specific protocol:

- coordinator/hub scheduling as an optional Mantle remote-builder mode;
- worker-initiated registration for NAT-friendly builders;
- capability-matched queues over Mantle systems, feature labels, sandbox modes, network modes, and resource limits;
- normalized concrete-build request dedupe with conflicting live-output claims rejected;
- missing-input negotiation before upload;
- worker-signed outputs verified by the client before import;
- bounded live-log replay and slow-subscriber handling;
- restart/reload adoption through stable job keys and resumable worker summaries.

Reject direct adaptation of Tribuchet's Nix external-builders shim, nix-daemon input import, hardcoded `/nix/store`, TOML-first configuration, and identical-scratch-path output trick. Mantle output admission stays based on signed PathInfo, artifact attestations, CAS digests, requested output identity, and configured logical store prefix.

## Changed Cairn artifacts

- `cairn/changes/p2p-remote-builders/proposal.md` now lists Tribuchet as remote-builder service-architecture prior art.
- `cairn/changes/p2p-remote-builders/design.md` now describes optional coordinator mode, worker registration, request-key dedupe, conflict rejection, log replay, restart adoption, and the explicit rejection of scratch-path identity as an output contract.
- `cairn/changes/p2p-remote-builders/specs/remote-builds/spec.md` now adds `r[remote_builds.coordinator_scheduling]` plus log replay and restart-adoption scenarios.
- `cairn/changes/p2p-remote-builders/tasks.md` now includes contract, implementation, and verification tasks for coordinator scheduling and Tribuchet-inspired resilience semantics.
- `README.md` now records `Mic92/tribuchet` under `## References`.

## Owner

Mantle remote-builder implementation owner.

## Validation

Transcript: `cairn/changes/p2p-remote-builders/evidence/tribuchet-validation-2026-07-01.txt`.

Passed:

- `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .`
- `nix run path:/home/brittonr/git/cairn#cairn -- gate proposal p2p-remote-builders --root .`
- `nix run path:/home/brittonr/git/cairn#cairn -- gate design p2p-remote-builders --root .`
- `nix run path:/home/brittonr/git/cairn#cairn -- gate tasks p2p-remote-builders --root .`

## Next action

Implement the coordinator-scheduling core after the current `src/remote_build.rs` worktree changes are reviewed, preserving the new `r[remote_builds.coordinator_scheduling]` requirement coverage.
