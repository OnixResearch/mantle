## Implementation

- [x] [serial] I1 Extract pure matching logic that decides whether an imported source record satisfies an evaluated fixed-output fetcher record, including kind, identity, hash mode, content digest, store prefix, VCS revision, and pin state. r[source_transports.source_bundle_realizes_fetcher_inputs]
- [x] [serial] I2 Add a thin shell path that materializes eligible source-state records into the existing store/castore/fixed-output input boundary without live network access. r[source_transports.source_bundle_realizes_fetcher_inputs]
- [x] [serial] I3 Thread summarized source-bundle readiness facts into build planning and route planning so `source-bundle` can be selected before local build when imported source state is the missing input. r[realization_routing.source_bundle_route_execution]
- [x] [serial] I4 Preserve fail-closed handling for missing, stale, unsupported, untrusted, unpinned, network-required, wrong-revision, wrong-hash, and wrong-prefix source records before sandbox execution. r[source_transports.source_bundle_realizes_fetcher_inputs] r[realization_routing.source_bundle_route_execution]
- [x] [serial] I5 Update operator docs to distinguish source-bundle input realization from cached/substituted output reuse and to keep source-bundle evidence bounded. r[source_transports.source_bundle_realizes_fetcher_inputs]

## Verification

- [x] [serial] V1 Positive: a build root with a fixed URL source succeeds offline after a matching source bundle is imported and pinned, with no live network request and with route evidence selecting or naming `source-bundle`. r[source_transports.source_bundle_realizes_fetcher_inputs] r[realization_routing.source_bundle_route_execution]
- [x] [serial] V2 Positive: local tarball and VCS snapshot fixtures materialize from source state only when expected hash mode, content digest, and revision identity match. r[source_transports.source_bundle_realizes_fetcher_inputs]
- [x] [serial] V3 Negative: stale payload bytes, wrong fixed-output hash, wrong VCS revision, mismatched store prefix, unpinned imported source, unsupported adapter, and remote-only URL without imported payload fail before sandbox execution. r[source_transports.source_bundle_realizes_fetcher_inputs]
- [x] [serial] V4 Negative: route planning in offline mode rejects trusted-substitute, remote-builder, and live-fetch routes when source-bundle readiness is incomplete, and emits stable reason codes. r[realization_routing.source_bundle_route_execution]
- [x] [serial] V5 Run focused source-bundle/route/fetcher tests plus `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .`, proposal gate, design gate, and tasks gate for this change. r[source_transports.source_bundle_realizes_fetcher_inputs] r[realization_routing.source_bundle_route_execution]
