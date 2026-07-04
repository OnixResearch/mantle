## Implementation

- [x] [serial] I1 Extend remote input manifests with source-bundle/imported-source-state refs, readiness classes, BLAKE3 identities, store-prefix binding, and named limits. r[remote_builds.source_bundle_input_sync]
- [x] [serial] I2 Let builders compute missing input sets across CAS, PathInfo, and source refs and request only missing refs. r[remote_builds.source_bundle_input_sync]
- [x] [serial] I3 Let clients materialize requested source upload artifacts from verified local store or imported source-bundle state without live network access. r[remote_builds.source_bundle_input_sync]
- [x] [serial] I4 Add upload privacy policy summaries and fail-closed class/quota enforcement before bytes move. r[remote_builds.source_bundle_input_sync]

## Verification

- [x] [serial] V1 Positive: a remote build upload uses verified imported source state when the logical source path is absent from the physical store. r[remote_builds.source_bundle_input_sync]
- [x] [serial] V2 Negative: stale source digest, unsupported source kind, missing source state, upload quota overflow, disallowed upload class, and attempted network fetch fail closed. r[remote_builds.source_bundle_input_sync]
- [x] [serial] V3 Run focused source-bundle/remote-input tests plus Cairn validate and proposal/design/tasks gates for this change. r[remote_builds.source_bundle_input_sync]
