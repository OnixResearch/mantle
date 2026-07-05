## Implementation

- [x] [serial] I1 Define a bootstrap source-bundle profile model that enumerates required provider archive, provider manifest, bootstrap source archive, Mantle source, vendored Cargo, toolchain source-root, and proof input records for each supported bootstrap mode. r[bootstrap_inventory.offline_bootstrap_source_bundles]
- [x] [serial] I2 Extend source-bundle export/preflight commands or bootstrap-specific helpers to produce and validate the profile without running builds or fetching missing sources. r[bootstrap_inventory.offline_bootstrap_source_bundles]
- [x] [serial] I3 Teach bootstrap/self-build shells to consume imported and pinned profile records before live fetchers when offline bootstrap mode is selected. r[bootstrap_inventory.offline_bootstrap_source_bundles]
- [x] [serial] I4 Validate provider metadata, fixed-output hashes, reduced-provider provenance, normalized seed contract facts, source tree identity, and vendored Cargo input identity at the offline bootstrap boundary. r[bootstrap_inventory.offline_bootstrap_source_bundles]
- [x] [serial] I5 Update bootstrap docs and proof guide to show connected export, disconnected import/pin, offline preflight, and offline bootstrap/self-build command sequences with bounded non-claims. r[bootstrap_inventory.offline_bootstrap_source_bundles]

## Verification

- [x] [serial] V1 Positive: a bootstrap source-bundle profile can be exported on a fixture root, imported and pinned into a fresh state dir, and preflighted as ready without network access. r[bootstrap_inventory.offline_bootstrap_source_bundles]
- [x] [serial] V2 Positive: bootstrap or self-build offline mode consumes matching imported profile records and does not invoke live source fetchers for profile-covered inputs. r[bootstrap_inventory.offline_bootstrap_source_bundles]
- [x] [serial] V3 Negative: missing provider archive, stale provider manifest, wrong provider kind, wrong logical store prefix, missing vendored Cargo input, unsupported profile version, and unpinned source record fail before bootstrap execution. r[bootstrap_inventory.offline_bootstrap_source_bundles]
- [x] [serial] V4 Negative: evidence wording and reports do not claim seed trust removal, compiler correctness, release reproducibility, or self-build success from source-bundle readiness alone. r[bootstrap_inventory.offline_bootstrap_source_bundles]
- [x] [serial] V5 Run focused bootstrap/source-bundle tests plus `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .`, proposal gate, design gate, and tasks gate for this change. r[bootstrap_inventory.offline_bootstrap_source_bundles]
