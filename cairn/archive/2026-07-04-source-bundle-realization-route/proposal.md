## Why

Mantle already has a source-bundle format, import/verify commands, and an `--offline-source-preflight` gate. That proves source/input material can be present and identity-matched before a build. The remaining offline-build gap is execution: after preflight, ordinary build planning still treats source bundles as a rejected route and fixed-output fetchers may still need live network fetches when their outputs are not already in local store state.

Offline source state should be able to realize declared fixed-output source inputs without network access. If a selected build root needs a fixed URL, tarball, or VCS snapshot that has been imported and pinned as source state, Mantle should verify the source record and materialize the corresponding store/castore input rather than reaching out to the network or failing late in a sandbox.

## What Changes

- Make source-bundle readiness a first-class realization route for fixed-output fetcher/source inputs.
- Materialize accepted fixed URL, unpacked tarball, and VCS snapshot records from imported source state into the store/castore boundary with digest verification.
- Teach build planning to mark the `source-bundle` route eligible when imported/pinned source state satisfies all required source records under offline policy.
- Reject stale, unpinned, unsupported, untrusted, or network-required source records before sandbox execution.
- Keep source-bundle evidence bounded: it proves source availability and identity, not build success or output correctness.

## Impact

- **Files**: `src/source_bundle.rs`, `src/build_plan.rs`, `src/realization_routing.rs`, `crates/crunch-build/src/fetch_build_service.rs`, `crates/crunch-store` import/export helpers, source-bundle docs, and this Cairn spec delta.
- **Testing**: offline source preflight positives/negatives, fixed-url/tarball/git source-state realization, route-plan selection, stale/unpinned rejection, and no-network execution smokes.

## Out of Scope

- Fetching or repairing missing source records during build execution.
- Treating source-bundle readiness as proof of build success, output trust, compiler correctness, or reproducibility.
- Arbitrary package-manager cache hydration beyond modeled source records.
- Remote-builder input sync beyond consuming the same source-state facts already modeled here.
