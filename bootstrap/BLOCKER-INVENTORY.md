# Bootstrap blocker inventory gate

`./scripts/check-bootstrap-blocker-inventory.sh` is the lightweight readiness-drift rail for the full-source bootstrap chain. It does **not** prove that the chain is promoted. It inventories remaining blockers and fails closed if repository-controlled status text claims full-source promotion while configured blockers remain.

## Reports

The wrapper writes deterministic reports by default:

- JSON: `target/bootstrap-blocker-inventory/current.json`
- Markdown: `target/bootstrap-blocker-inventory/current.md`

Both reports omit timestamps and host-specific absolute paths so they can be saved as evidence. The JSON summary names `actionable_finding_count`, `metadata_suppression_count`, and `source_suppression_count`; the Markdown report renders metadata suppressions, source suppressions, promotion claims, and actionable findings in separate sections.

## Marker classes

The current taxonomy is intentionally small:

- `bridge-output`: a stage uses/documents a bridge output instead of source-built proof.
- `compiler-runtime-crash-boundary`: a compiler/runtime segfault, timeout, signal-derived exit, or static-link boundary gates promotion.
- `legacy-provider-fallback`: legacy musl.cc/seed-legacy/host fallback remains in the path or documentation.
- `normalization-only-provider`: a provider satisfies shape/normalization but not full-source proof.
- `placeholder-deferred`: placeholder/TODO/deferred work remains in a bootstrap-critical surface.
- `prerequisite-gated-evidence`: evidence is explicitly blocked or prerequisite-gated.

## Modes

Default enforcement mode:

```sh
./scripts/check-bootstrap-blocker-inventory.sh
```

This exits successfully only when the checked baseline is clean: no unsuppressed actionable findings and no promotion claims. It exits nonzero while preserving the JSON/Markdown reports when source blockers or promotion claims remain.

Report-only mode:

```sh
./scripts/check-bootstrap-blocker-inventory.sh --report-only
```

Use this when refreshing inventory evidence or choosing the next repair target. It exits successfully after valid JSON/Markdown report generation even when blockers remain, and the report still states that the repository is not clean.

## Retiring a marker

Retire or narrow a marker class only when the corresponding blocker has positive source-built evidence. The same change should keep a negative promotion-drift fixture proving that any remaining blocker class still rejects overclaiming full-source readiness.

Selected full-source predecessor markers are classified only when both archive-stable reports pass exact checks: `full-source-provider-admission.json` must bind the admitted provider/output/source-closure identities, and `full-source-provider-fixed-point.json` must bind the full-source provider kind, complete override count, zero live fetches, and matching stage binaries. The classifier then accepts only the named path/class/excerpt tuples. Missing, stale, tampered, or new marker text remains actionable. Lexical uses such as the mkstemp filename placeholder, successful GCC static-link progress labels, and the explicit-only legacy compatibility comment are classified separately from proof-backed predecessor boundaries.
