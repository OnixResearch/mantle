# Verification gauntlets

Mantle verification gauntlets are deterministic report layers for pressure-testing
release and global reproducibility evidence. They do not run expensive proofs by
themselves and they do not promote broader claims. Track-specific runners produce
JSON evidence; Mantle canonicalizes the reports, records BLAKE3 digests, and lets
existing release/global admission gates consume those digests and blockers.

## Track reports

`crunch-release-core` defines canonical compact JSON schemas for these tracks:

- `mantle-adversarial-hermeticity-gauntlet-report-v1`
- `mantle-bootstrap-pressure-gauntlet-report-v1`
- `mantle-substitution-cache-attack-gauntlet-report-v1`
- `mantle-nix-mantle-comparison-corpus-report-v1`
- `mantle-release-repeatability-matrix-report-v1`
- `mantle-continuous-reproducibility-gauntlet-report-v1`

Each report keeps a functional-core classification boundary: inputs are plain
owned data, outputs are canonical reports, and side effects stay in the CLI shell.
Strict-mode failures, stale evidence, unsupported axes, mismatched digests,
undeclared protected execution, reused fresh-store cells, malicious cache
acceptance, and non-equivalent Nix/Mantle cases become explicit blockers or
non-claims.

## CLI entry points

Canonicalize a track report without changing its claim class:

```bash
mantle release gauntlet canonicalize \
  --kind release-repeatability \
  path/to/report.json \
  --output target/gauntlets/repeatability.json
```

Aggregate current track evidence into a continuous gauntlet report:

```bash
mantle release gauntlet continuous \
  --context target/gauntlets/context.json \
  --track target/gauntlets/repeatability-track.json \
  --track target/gauntlets/cache-attack-track.json \
  --report-path target/gauntlets/continuous.json
```

The continuous context binds the current source, policy, universe, toolchain,
host class, witness-set, and required track schema versions. Any mismatch marks
track evidence stale instead of promoting it.

## Global admission seam

`mantle-global-reproducibility-surface-evidence-v1` accepts optional
`gauntlet_report_digests_blake3` and `gauntlet_blockers`. The global evaluator
adds those digests to the evidence digest set and treats the blockers exactly
like other admission blockers. This preserves the existing global gate: gauntlet
reports can block or support scoped evidence, but they cannot bypass action,
source, toolchain, hermeticity, output, or witness requirements.
