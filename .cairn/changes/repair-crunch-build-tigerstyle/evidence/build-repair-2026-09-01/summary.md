# Build Tiger Style repair

## Accepted scope

The initial strict run reported 20 findings across six build-layer files. After
those library failures cleared, the next focused round exposed two package-local
wrapper CLI findings. Both were repaired in the same package boundary.

The final focused and repository transcripts contain zero finding in
`crunch-build` or `crunch-rustc-wrapper`. Both stop later at eight
`crunch-pipeline` findings in `crates/crunch-pipeline/src/lib.rs`:

- two assertion-density findings;
- two unbounded-collection-growth findings;
- two too-many-parameters findings;
- one bool-naming finding;
- one ambiguous-params finding.

## Structural repair

- Build-request bounds use checked arithmetic and named inputs.
- Structured attributes use bounded, non-recursive sorted-map serialization.
  Nested canonical byte assertions guard ordering.
- Structured shell rendering returns typed errors instead of production
  `expect` calls.
- Content-addressed name and output planning return typed errors instead of
  panicking.
- Execution-profile path and environment checks use explicit predicates and
  established invariants.
- Registry and worker interfaces use named request records.
- Rustc-wrapper publication keeps atomic no-replace behavior and explicit path
  invariants.
- Wrapper CLI argument behavior remains unchanged.

## Validation

- Pre-change package tests: 702 passed, zero failed.
- Post-change package tests: 704 passed, zero failed.
- Strict first-party Clippy with `--no-deps`: pass.
- Mantle caller and all targets: pass.
- Package and root formatting: pass.
- `nix flake check --no-build -L`: pass.
- New Tiger allowances, budgets, baselines, or scope reductions: none.
- Focused Tiger rounds used: 2 of 10.

## Full-check boundaries

The local-builder full check exits 1 when `mantle-nextest` cannot read five
tracked V47/V48 evidence files from its filtered Nix source. Six `include_*`
diagnostics cite those five paths. Its parallel Tiger derivation did not finish
before Nix stopped the run.

The ordinary full check exits 1 at the eight `crunch-pipeline` Tiger findings.
The dedicated repository Tiger transcript records the same eight findings.
Neither full check reports a `crunch-build` or `crunch-rustc-wrapper` finding.

## Non-claims

The repository Tiger derivation and full flake checks are not green. This change
does not suppress or repair the pipeline findings or the independent filtered
Nix-source failure. It does not claim full flake success.
