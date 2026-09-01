# Pipeline Tiger Style baseline

## Findings

The focused command exits 1 with eight findings in
`crates/crunch-pipeline/src/lib.rs`:

- two `assertion-density` findings;
- two `unbounded-collection-growth` findings;
- two `too-many-parameters` findings;
- one `bool-naming` finding;
- one `ambiguous-params` finding.

## Tests

`nix develop -c cargo test -p crunch-pipeline --lib --tests` exits 0.

- Library tests: 39 passed.
- Integration tests: 19 passed and four ignored.
- No test failed.

This baseline proves existing positive and negative behavior before the repair.
It does not accept the Tiger Style findings.
