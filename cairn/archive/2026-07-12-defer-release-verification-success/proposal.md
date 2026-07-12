## Why

`mantle release verify` currently renders its human success banner or JSON verification payload before enforcing required reproducibility, deterministic proof, provider fixed-point, stack-provenance, external-role, and StageX policy checks. A command can therefore print success-looking output and then exit with an error.

Operator and machine-readable success must represent the final selected policy verdict, not an intermediate manifest-integrity result.

## What Changes

- Aggregate every selected release verification contribution into one pure final decision before rendering terminal output.
- Keep manifest integrity distinct from complete policy satisfaction in the decision model.
- Emit a human success banner only for a passing final decision; rejected decisions receive explicit failure wording and ordered diagnostics.
- Add a stable JSON `valid`/disposition contract for both accepted and policy-rejected results while preserving nonzero exit status on rejection and JSON-only stdout.
- Add positive success fixtures and negative fixtures for every late policy gate, asserting that no failure path emits a success marker.

## Impact

- **Files**: release verification decision core, CLI rendering shell, JSON schema/output, tests, and operator documentation.
- **Compatibility**: consumers of release verification JSON gain an explicit final validity field and must not infer success from payload presence alone.
- **Related work**: function-address and Cairn handoff checks supplied by active changes become additional contributors to the same final decision; this change does not reimplement those validators.
- **Claims**: success means all policy checks selected by that invocation passed; it does not strengthen the bounded meaning of any individual evidence class.
