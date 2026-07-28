# Tasks

## Source and inventory repair

- [ ] [serial] I1 Reproduce the Nix source-filter failure and inventory findings from the accepted adoption snapshot. r[mantle.durable_publication_validation.source_closure]
- [ ] [serial] I2 Include the exact `fixtures/semantic-operation` closure without widening unrelated source roots. r[mantle.durable_publication_validation.source_closure]
- [ ] [parallel] V1 Add positive fixture-presence coverage and a negative omission case for the Nix-built package. r[mantle.durable_publication_validation.source_closure]
- [ ] [serial] I3 Classify new bootstrap inventory matches with exact evidence, while preserving every live blocker and promotion claim. r[mantle.durable_publication_validation.inventory]
- [ ] [parallel] V2 Run report-only and enforcement inventory tests with positive and negative fixtures. r[mantle.durable_publication_validation.inventory]

## Tool boundaries

- [ ] [serial] I4 Keep product-owned Clippy strict with `--no-deps` and record vendored dependency failures in a separate audit. r[mantle.durable_publication_validation.tools]
- [ ] [serial] I5 Remove the Octet parent-traversal diagnostic path through an accepted owner-side path contract. r[mantle.durable_publication_validation.tools]
- [ ] [parallel] V3 Run workspace tests, focused Clippy, Tiger Style, Octet, Nickel, and source-closure checks. r[mantle.durable_publication_validation.broad_rail]

## Broad validation

- [ ] [serial] V4 Run `nix flake check -L` and record success or the next exact independent blocker. r[mantle.durable_publication_validation.broad_rail]
- [ ] [parallel] V5 Run Cairn validation, gates, and focused traceability. r[mantle.durable_publication_validation.evidence]
- [ ] [serial] V6 Synchronize the accepted specification and archive only after the evidence records all remaining blockers and non-claims. r[mantle.durable_publication_validation.evidence]
