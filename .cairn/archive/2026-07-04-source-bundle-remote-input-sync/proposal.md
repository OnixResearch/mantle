## Why

Remote builders should not fetch undeclared sources, read ambient package-manager caches, or rely on the client's physical store path having every source input on disk. Mantle already has source-bundle state; remote input synchronization should consume that verified source/input material directly.

## What Changes

- Include source-bundle and imported-source-state refs in remote input manifests.
- Let builders request only missing CAS, PathInfo, and source-input refs.
- Let clients satisfy requested source refs from local store state or verified imported source-bundle state.
- Enforce upload privacy policy before transfer, with bounded class, byte, and object-count reports.

## Impact

- **Files**: remote input manifest core, source-bundle state lookup, upload artifact materialization, route planning, reports, docs, and Cairn remote-builds/source-transport spec deltas.
- **Testing**: positive imported-source upload; negative stale source digest, unsupported source kind, missing source state, upload quota, disallowed class, and undeclared network fetch fixtures.

## Out of Scope

- Source acquisition during remote import/verify.
- Builder-side Nickel evaluation.
- Treating a source bundle as output trust or build success evidence.
