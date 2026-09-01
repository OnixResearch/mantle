# Pipeline Tiger Style repair evidence

## Result

The focused baseline contained eight `crunch-pipeline` findings. The first
repair round exposed two additional local findings: one quantity name and one
function-boundary assertion-density finding. The second round reports no
pipeline finding.

The repository Tiger derivation also reports no pipeline finding. It now exits
1 at 36 later Mantle root-library findings across five files:

- `src/operator_contract.rs`: 17;
- `src/protected_exec_seccomp.rs`: 8;
- `src/protected_exec.rs`: 7;
- `src/errors.rs`: 3;
- `src/bootstrap.rs`: 1.

## Structural repair

- Eager root collection reserves the admitted root count and rejects an extra
  message before growth.
- Managed registration derives a checked entry count, deduplicates before I/O,
  and asserts only internal plan facts.
- The existing builder bundle now crosses the private registered-build helper
  as one capability group.
- Cache-only service construction is split at the builder boundary and retains
  strict source policy and hermeticity.
- Failure-key normalization uses named private roles.
- Public pipeline APIs, result schemas, root classes, and effect order remain
  unchanged.

## Validation

- Pre-change tests: 39 library and 19 integration tests passed; four integration
  tests were ignored.
- Post-change tests: 40 library and 19 integration tests passed; four
  integration tests were ignored.
- The new negative test rejects a root message beyond the admitted count.
- The existing managed-generation test now also proves duplicate output paths
  produce one root registration.
- Strict package Clippy passes with `-D warnings`.
- Mantle all-target caller compilation passes.
- Package formatting, diff checks, and Nix flake evaluation pass.
- The source diff adds no lint allowance or expectation attribute.

## Full-check boundaries

The local-builder full check exits 1 at the same 36 root-library Tiger Style
findings. The ordinary full check exits 1 earlier when a remote builder imports
`rust-src-1.96.0-nightly-2026-03-21-x86_64-unknown-linux-gnu` with a SHA-256
that differs from the specified fixed-output value. Neither full check reports
a pipeline finding.

## Non-claims

The repository Tiger derivation and full flake checks are not yet green. This
change does not repair or suppress the 36 later root-library findings or the
independent remote fixed-output mismatch. It does not claim builder correctness,
store durability, source trust, or release eligibility.
