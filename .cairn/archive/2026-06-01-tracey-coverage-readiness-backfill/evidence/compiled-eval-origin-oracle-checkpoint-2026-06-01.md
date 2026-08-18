# Compiled-eval coverage origin oracle checkpoint

Task-ID: compiled-eval-origin-checkpoint
Covers: verification_evidence.tracey_coverage_readiness

## Question

Do the remaining `compiled-eval.*` Tracey missing IDs in the first-batch coverage report come from the legacy OpenSpec spec `openspec/specs/compiled-eval-backends/spec.md`, rather than from accepted Cairn specs under `cairn/specs/`?

## Inspected evidence

- `cairn/changes/tracey-coverage-readiness-backfill/evidence/tracey-coverage-after-first-batch-grouped-2026-06-01.json` lists exactly 21 missing IDs in group `compiled-eval`.
- `openspec/specs/compiled-eval-backends/spec.md` contains exactly 21 `r[compiled-eval.*]` requirement markers.
- `cairn/specs/` contains no `compiled-eval` requirement markers.
- Set comparison between those coverage missing IDs and those legacy OpenSpec spec markers is empty.

Command transcript:

```sh
jq -r '.groups[] | select(.group=="compiled-eval") | .missing_ids[]' \
  cairn/changes/tracey-coverage-readiness-backfill/evidence/tracey-coverage-after-first-batch-grouped-2026-06-01.json \
  | sort > /tmp/mantle-compiled-eval-missing.ids
rg -o 'r\[compiled-eval[^]]+\]' openspec/specs/compiled-eval-backends/spec.md \
  | sed 's/^.*r\[//; s/\]$//' \
  | sort > /tmp/mantle-compiled-eval-spec.ids
printf 'missing_count='; wc -l < /tmp/mantle-compiled-eval-missing.ids
printf 'spec_count='; wc -l < /tmp/mantle-compiled-eval-spec.ids
printf 'diff:\n'
comm -3 /tmp/mantle-compiled-eval-missing.ids /tmp/mantle-compiled-eval-spec.ids
```

Output:

```text
missing_count=21
spec_count=21
diff:
```

Additional inspected marker locations:

```text
openspec/specs/compiled-eval-backends/spec.md:14:r[compiled-eval.backend-boundary]
openspec/specs/compiled-eval-backends/spec.md:23:r[compiled-eval.backend-boundary.default]
openspec/specs/compiled-eval-backends/spec.md:32:r[compiled-eval.backend-boundary.swap]
openspec/specs/compiled-eval-backends/spec.md:44:r[compiled-eval.profiling-gate]
openspec/specs/compiled-eval-backends/spec.md:52:r[compiled-eval.profiling-gate.no-evidence]
openspec/specs/compiled-eval-backends/spec.md:60:r[compiled-eval.profiling-gate.evidence]
openspec/specs/compiled-eval-backends/spec.md:71:r[compiled-eval.benchmark-guardrail]
openspec/specs/compiled-eval-backends/spec.md:83:r[compiled-eval.benchmark-guardrail.recorded]
openspec/specs/compiled-eval-backends/spec.md:93:r[compiled-eval.benchmark-guardrail.regression]
openspec/specs/compiled-eval-backends/spec.md:104:r[compiled-eval.cranelift-first]
openspec/specs/compiled-eval-backends/spec.md:111:r[compiled-eval.cranelift-first.initial]
openspec/specs/compiled-eval-backends/spec.md:120:r[compiled-eval.cranelift-first.llvm-later]
openspec/specs/compiled-eval-backends/spec.md:131:r[compiled-eval.private-backend-seam]
openspec/specs/compiled-eval-backends/spec.md:137:r[compiled-eval.private-backend-seam.callers]
openspec/specs/compiled-eval-backends/spec.md:145:r[compiled-eval.private-backend-seam.no-leak]
openspec/specs/compiled-eval-backends/spec.md:155:r[compiled-eval.cranelift-prototype-subset]
openspec/specs/compiled-eval-backends/spec.md:167:r[compiled-eval.cranelift-prototype-subset.default]
openspec/specs/compiled-eval-backends/spec.md:175:r[compiled-eval.cranelift-prototype-subset.unsupported]
openspec/specs/compiled-eval-backends/spec.md:185:r[compiled-eval.future-semantics]
openspec/specs/compiled-eval-backends/spec.md:194:r[compiled-eval.future-semantics.contracts]
openspec/specs/compiled-eval-backends/spec.md:203:r[compiled-eval.future-semantics.shape]
```

## Decision

Yes. The `compiled-eval` missing group in the current Tracey coverage report is sourced from legacy OpenSpec requirements in `openspec/specs/compiled-eval-backends/spec.md`. This checkpoint only proves source provenance for that missing group. It does not claim those requirements are implemented, verified, migrated into Cairn, or safe to bridge without inspecting the archived compiled-eval evidence.

## Owner

Mantle maintainer / current agent for `tracey-coverage-readiness-backfill`.

## Next action

For the next backfill batch, either:

1. inspect `openspec/changes/archive/2026-04-27-explore-compiled-eval-backends/` and add bounded bridge refs only for archived prototype claims with durable evidence, or
2. leave `compiled-eval.*` as explicit legacy scanner/migration debt and move to a smaller `rust_package_planning` archived-change batch.
