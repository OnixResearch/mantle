## Phase 1: Dev-cache foundation

- [x] [serial] I1 Add the opt-in dev-only provider-output cache keyed by source-authority and policy digests, with receipt validation and cold fallback. r[source_built_fixed_point_improved_iteration.dev_provider_cache]
- [x] [serial] I2 Add the content-addressed native store and state keyed by the plan digest. r[source_built_fixed_point_improved_iteration.dev_store_snapshot]
- [x] [serial] I3 Add the unchanged-source fast-fail baseline with an exact prior identity and dev-only disposition. r[source_built_fixed_point_improved_iteration.dev_fast_fail_baseline]
- [x] [serial] I4 Keep cache, store reuse, and fast-fail paths dev-only. Prevent promoted receipts and release-alias updates. r[source_built_fixed_point_improved_iteration.dev_provider_cache]

## Phase 2: Foundation validation

- [x] [serial] V1 Add positive and negative tests for provider adoption, receipt mismatch, persistent store reuse, fast-fail, disabled cold paths, and non-authorizing transcripts. r[source_built_fixed_point_improved_iteration.dev_provider_cache] r[source_built_fixed_point_improved_iteration.dev_store_snapshot] r[source_built_fixed_point_improved_iteration.dev_fast_fail_baseline]
- [ ] [serial] V2 Run focused source-built fixed-point and CLI tests, leaf formatting, focused Clippy, and `git diff --check`. Record exact output in the change evidence. r[source_built_fixed_point_improved_iteration.dev_provider_cache]
- [ ] [serial] V3 Record the scope split, make sure that the promoted path stays cache-disabled in focused tests, and run Cairn validation plus all three gates. r[source_built_fixed_point_improved_iteration.dev_provider_cache]
