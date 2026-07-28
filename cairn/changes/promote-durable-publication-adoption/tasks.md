# Tasks

## Promotion

- [ ] [serial] I1 Fetch Mantle `origin` and record the canonical target, candidate, merge base, and candidate-only commits. r[mantle.durable_publication_promotion.candidate]
- [ ] [serial] I2 Verify that one exact candidate descends from `d1f3d6d96e2b0d9cd8497cd89b8e5a93d4a7dfaf`, remains fast-forwardable, and contains only reviewed post-acceptance Cairn paths. r[mantle.durable_publication_promotion.candidate]
- [ ] [serial] I3 Verify that Onix Core remote `main` contains `b8387cd7d59fa3b0d4ea67646352dd27c4f7d7ed`. r[mantle.durable_publication_promotion.ordering]

## Verification and mutation

- [ ] [parallel] V1 Run focused backend tests, the sequential Mantle binary suite, package build, version probe, formatting, product-owned Clippy, Tiger Style, Nickel, focused Nix, Cairn, and traceability checks. r[mantle.durable_publication_promotion.validation]
- [ ] [serial] I4 Record explicit authorization for the exact candidate and canonical target. r[mantle.durable_publication_promotion.safe_push]
- [ ] [serial] I5 Advance Mantle `origin/main` through a normal fast-forward push of that exact candidate. r[mantle.durable_publication_promotion.safe_push]
- [ ] [serial] V2 Re-fetch and verify the exact remote candidate, reviewed Mantle ancestor, durable RID, and durable revision. r[mantle.durable_publication_promotion.validation]
- [ ] [serial] V3 Synchronize the accepted specification and archive the completed change with receipts. r[mantle.durable_publication_promotion.validation]
