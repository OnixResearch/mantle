## Why

Mantle already owns build realization, release evidence, remote workers, cache substitution, and portable proof bundles. Oxide's Tufaceous/Tough update repository work and Buildomat ephemeral worker model are close architectural references for signed release metadata, artifact tags, compatibility verification, captured logs, replayable jobs, and worker cleanup. `cancel-safe-futures` adds another useful reference for making async worker orchestration fail predictably instead of losing cleanup or output evidence on cancellation.

This change writes the Cairn package for adapting those patterns into Mantle-native release and worker rails.

## What Changes

- Add a reference inventory for Tufaceous, Tough, Buildomat, and cancel-safe-futures.
- Define TUF-style release repository profiles for Mantle release evidence bundles while preserving Mantle's BLAKE3, PathInfo, Valence sidecar, and non-claim boundaries.
- Define ephemeral worker evidence profiles for build jobs, logs, artifacts, cleanup, and replayable delivery state.
- Define async cancellation-safety requirements for worker orchestration, remote build sessions, and output persistence.
- Require positive and negative fixtures before adopting dependency code or changing release bundle formats.

## Impact

- Release bundles get a clearer path toward signed metadata, artifact tags, and compatibility checks.
- Remote-build and cache-substitution jobs get stronger cleanup and evidence-preservation requirements.
- Mantle remains the semantic owner of build/release evidence; Oxide references inform architecture but do not become proof authorities.
