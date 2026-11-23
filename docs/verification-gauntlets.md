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
- `mantle-strict-hermeticity-regression-suite-report-v1`
- `mantle-continuous-reproducibility-gauntlet-report-v1`

Each report keeps a functional-core classification boundary: inputs are plain
owned data, outputs are canonical reports, and side effects stay in the CLI shell.
Strict-mode failures, stale evidence, unsupported axes, mismatched digests,
undeclared protected execution, reused fresh-store cells, malicious cache
acceptance, non-equivalent Nix/Mantle cases, and strict hermeticity regression
fixture drift become explicit blockers or non-claims.

## CLI entry points

Canonicalize a track report without changing its claim class:

```bash
mantle release gauntlet canonicalize \
  --kind release-repeatability \
  path/to/report.json \
  --output target/gauntlets/repeatability.json
```

Evaluate the maintained strict hermeticity regression suite from its declared
plan and fixture evidence:

```bash
mantle release gauntlet strict-hermeticity-regression \
  --run-id strict-hermeticity-2026-07-04 \
  --plan target/gauntlets/strict-hermeticity-plan.json \
  --evidence target/gauntlets/strict-hermeticity-evidence.json \
  --report-path target/gauntlets/strict-hermeticity-report.json
```

The plan schema is `mantle-strict-hermeticity-regression-suite-plan-v1`. It lists
each fixture, perturbation axis, strict mode, expected verdict, required blocker
or audit class, unsupported-host non-claim, and optional expected output digest.
The required axes are `clean-strict`, `environment-poisoning`, `path-poisoning`,
`undeclared-host-execution`, `network-access`, `missing-closure-facts`,
`umask-drift`, `temp-root-dependence`, and `nondeterministic-output`. The suite
report binds the BLAKE3 digest of the canonical plan. The clean fixture must
record matching first/repeated output digests; poisoned fixtures must be blocked
or rejected with the required blocker/audit class. Unsupported host axes are
non-claims and never count as passing coverage.

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

### Pinned deterministic execution machines in witness evidence

Each witness in `GlobalWitnessEvidence` carries an optional `execution_machine`
record (`engine_name`, `engine_version_digest_blake3`, `cpu_vendor_class`, and
`pinned_inputs` containing optional `start_epoch_seconds`, `entropy_seed`, and
`schedule_seed`). When `execution_machine` is omitted, the witness represents
native host execution and existing canonical report bytes are unchanged.

When a witness carries `execution_machine`, the evaluator enforces three rules:

1. **Axis neutralization**: a pinned input neutralizes the corresponding
   perturbation axes so that witness cannot satisfy them in
   `required_perturbation_axes` (counted in `witness_counts.skipped_pinned_axes`,
   and surfaced as a `witness-perturbation-axis-pinned` blocker whenever no
   unpinned witness covers that required axis):

   | Pinned input | Neutralized perturbation axes (case-insensitive) |
   |---|---|
   | `start_epoch_seconds` | `time`, `clock`, `timestamp`, `mtime`, `build-time`, `wall-clock`, `epoch`, and `time:*` / `clock:*` / `timestamp:*` |
   | `entropy_seed` | `randomness`, `entropy`, `rng`, `uuid`, `aslr`, and `randomness:*` / `entropy:*` / `rng:*` |
   | `schedule_seed` | `scheduling`, `schedule`, `concurrency`, `parallelism`, `thread-order`, `cpu-order`, and `schedule:*` / `scheduling:*` |

2. **Host-class collapsing**: pinned-machine witnesses count toward
   `required_host_classes` by their machine identity
   (`pinned-machine:<engine_name>:<engine_version_digest_blake3>:<cpu_vendor_class>`),
   so multiple witnesses running on the same pinned machine collapse to a single
   host class regardless of their free-form `host_class` label.
3. **Pinned-only non-claim**: when every accepted witness for an included
   surface ran inside a pinned machine, both that surface report and the
   top-level `GlobalReproducibilityReport` emit the
   `matched-under-pinned-machine` non-claim so the report cannot be misread as
   native-host reproducibility.
