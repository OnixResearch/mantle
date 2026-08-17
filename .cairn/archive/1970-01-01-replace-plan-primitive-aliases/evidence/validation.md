# Validation

## Baseline

Before the core change, `cargo test -p crunch-build` passed 648 library tests and one additional test.

`evidence/baseline/dynamic-plan-v1.json` records the accepted canonical JSON, BLAKE3 plan digest, placeholder result, graph diagnostic, and Rust call sites.

## Changed surface

The final focused `cargo test -p crunch-build` run passed:

- 652 library tests
- two additional test targets
- four compile-fail doctests

The frozen canonical fixture remained byte-identical. Its BLAKE3 plan digest remained `dc6814c1f500dc7e8575c3fd84a64ae78a70d38313ccbbfff4fbfcf7610f6750`.

The following checks passed:

- `./scripts/check-first-party-clippy.sh`
- `./scripts/check-first-party-tigerstyle.sh -p crunch-build -- --lib`
- Octet nominal-domain denial for `crunch-build`
- Nix `fmt` and `clippy` checks
- core purity search for filesystem, environment, process, async, and print effects
- Cairn validation, proposal gate, design gate, and tasks gate

Octet reported zero findings for `primitive_domain_alias`, `raw_domain_value`, and `newtype_invariant_bypass`. It retained 842 unrelated warning-level findings outside this migration.

## Broader workspace results

The unmodified base revision contains one deterministic workspace failure. `crunch-eval` finds `lib/artifact-auth-cutover-receipt.ncl`, but its embedded standard-library list does not contain that file.

The remaining 79 `crunch-eval` tests passed when that base test was skipped.

A broader workspace run passed 1,675 root tests and reported two unrelated parallel-test failures. Each failed test passed when run alone. These failures were the fake Slurm process test and a receipt-bound sysroot test.

No failing test referenced the dynamic-plan module, wire admission, typed graph, worker adapters, canonical bytes, or digest roles.

## Claim boundary

These checks support local scalar admission, category separation, wire compatibility, and source-shape policy conformance. They do not prove store presence, build success, sandbox enforcement, source trust, compiler correctness, or release eligibility.
