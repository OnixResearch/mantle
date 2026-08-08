# Tasks

## Promotion

- [x] [serial] I1 Fetch Mantle `origin` and record the canonical target, accepted parent, merge base, and divergence counts. r[mantle.durable_publication_promotion.candidate]
- [x] [serial] I2 Verify that one exact merge candidate contains both histories and changes only reviewed promotion lifecycle paths plus the bounded adoption receipt and validator refresh relative to its first parent. r[mantle.durable_publication_promotion.candidate]
- [x] [serial] I3 Verify that Onix Core remote `main` contains `bc4629c9e766d3db82e4dab9fe8c166c360b8435` and `b8387cd7d59fa3b0d4ea67646352dd27c4f7d7ed`. r[mantle.durable_publication_promotion.ordering]

## Verification and mutation

- [x] [parallel] V1 Run focused backend tests, the sequential Mantle binary suite, package build, version probe, formatting, product-owned Clippy, Tiger Style, Nickel, focused Nix, Cairn, and traceability checks. Require focused checks to pass and record exact pre-existing broad failures with first-parent path evidence. r[mantle.durable_publication_promotion.validation]
- [x] [serial] I4 Record explicit authorization for the exact merge candidate and canonical target. r[mantle.durable_publication_promotion.safe_push]
- [x] [serial] I5 Advance Mantle `origin/main` through a normal fast-forward push of that exact candidate. r[mantle.durable_publication_promotion.safe_push]
- [x] [serial] V2 Re-fetch and verify the exact remote candidate, both Mantle ancestors, durable RID, producer reconciliation, and durable revision. r[mantle.durable_publication_promotion.validation]
- [ ] [serial] V3 Synchronize the accepted specification and archive the completed change with receipts. r[mantle.durable_publication_promotion.validation]
