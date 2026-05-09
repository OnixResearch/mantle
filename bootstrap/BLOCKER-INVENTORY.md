# Bootstrap blocker inventory gate

`./scripts/check-bootstrap-blocker-inventory.sh` is the lightweight readiness-drift rail for the full-source bootstrap chain. It does **not** prove that the chain is promoted. It inventories remaining blockers and fails closed if repository-controlled status text claims full-source promotion while configured blockers remain.

## Reports

The wrapper writes deterministic reports by default:

- JSON: `target/bootstrap-blocker-inventory/current.json`
- Markdown: `target/bootstrap-blocker-inventory/current.md`

Both reports omit timestamps and host-specific absolute paths so they can be saved as OpenSpec evidence.

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

This exits successfully when blockers are present but status remains explicitly gated. It exits nonzero only if a promotion claim conflicts with remaining blockers.

Report-only mode:

```sh
./scripts/check-bootstrap-blocker-inventory.sh --report-only
```

Use this when refreshing inventory evidence without enforcing promotion-drift rejection.

## Retiring a marker

Retire or narrow a marker class only when the corresponding blocker has positive source-built evidence. The same change should keep a negative promotion-drift fixture proving that any remaining blocker class still rejects overclaiming full-source readiness.
