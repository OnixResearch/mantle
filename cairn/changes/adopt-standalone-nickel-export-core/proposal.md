## Why

Mantle currently duplicates evaluator-neutral Nickel export identity, receipt, and admission logic around its embedded `crunch-eval` evaluator. `OnixResearch/nickel-export` now publishes that shared functional core at immutable revision `257fafc1c746f1faf156207043a4c826bfb16d49`. Keeping the duplicate makes receipt drift possible, while replacing Mantle's evaluator or build authority would erase an important product boundary.

## What Changes

- Pin `nickel-export-core` to the exact published revision and record the immutable source identity in Mantle's release inputs. r[mantle.nickel_export_cutover.source]
- Add a thin adapter that supplies Mantle's embedded-evaluator observations to the standalone pure core. r[mantle.nickel_export_cutover.boundary]
- Preserve `crunch-eval` semantics, import and sandbox roots, destination writes, build evidence, and release authority in Mantle. r[mantle.nickel_export_cutover.authority]
- Dual-run the legacy and canonical admission paths over identical observations and compare canonical identities plus `mantle-nickel-export-receipt-v1` projections. r[mantle.nickel_export_cutover.dual_run]
- Retain the legacy implementation as the authoritative rollback path until positive and negative compatibility fixtures complete one full validation cycle without unexplained drift. r[mantle.nickel_export_cutover.rollback]
- Remove duplicated evaluator-neutral logic only after the cutover evidence and release gates pass. r[mantle.nickel_export_cutover.validation]

## Impact

- **Shared dependency**: exact Git revision `257fafc1c746f1faf156207043a4c826bfb16d49` of `github.com/OnixResearch/nickel-export`.
- **Mantle surfaces**: `src/nickel_export.rs`, embedded evaluator adapters, receipt fixtures, dependency locks, Nix source pins, and release evidence.
- **Authority**: Mantle remains the sole owner of evaluator behavior, filesystem and sandbox roots, output writes, build claims, and release decisions.
- **Claims**: compatibility proves checked request, exact identity, receipt projection, and freshness agreement only. It does not prove evaluator equivalence, product-policy correctness, build correctness, deployability, or release eligibility.
