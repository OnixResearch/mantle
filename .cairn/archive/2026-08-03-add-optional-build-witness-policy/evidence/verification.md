# Verification: optional build-witness policy

Date: 2026-08-03

## Result

Mantle now separates optional witness collection from explicit witness quorum.
The change keeps `mantle-release-policy-v1` and all canonical attestation bytes.

The pure core defines four named profiles:

- `self-proof-only` keeps a zero threshold and accepts no trusted witness identities.
- `optional-witness` keeps a zero threshold and can name trusted witness identities.
- `single-witness` keeps the compatible one-witness policy.
- `witness-quorum` requires an explicit positive threshold and independence field.

The supported independence fields are `witness_identity`, `signer_key_name`, and
`rebuild_environment_summary.host_class`.

## Core and shell boundary

`crates/crunch-attestation-core/src/policy.rs` owns profile construction and
evaluation. It has no file, network, process, clock, or environment access.

The core normalizes names in canonical order. It rejects empty names, control
characters, new-profile duplicates, unbounded collections, invalid thresholds,
unsupported selectors, and insufficient trusted identity sets.

The CLI shell parses options and publishes `policy.json` and
`revocations.json`. Each file uses a synchronized temporary file and an atomic
rename. The two files are not claimed as one transactional filesystem update.
The shell preflights both destination paths before the first rename.

`--force` preserves replacement behavior. Policy initialization never creates,
loads, or changes a signing key.

## Status and claim behavior

The stable witness-quorum statuses are:

- `not-required` for a zero threshold;
- `satisfied` for a met positive threshold;
- `insufficient` for an unmet positive threshold.

A zero threshold cannot produce the final class `quorum-satisfied`. Valid
optional witnesses can still produce `external-witness-match` technical status.

Release verification keeps JSON on standard output. Without global `--json`,
it also writes the two stable status lines to standard error. Tests cover all
three status values in human and JSON output.

Unknown keys, bad signatures, revocations, stale release references, rebuilt
digest mismatches, duplicate independence domains, and malformed environment
evidence stay visible. Invalid evidence never enters the accepted witness set.

## Positive and negative tests

All final Rust checks used this private target directory:

```text
target/optional-witness-private
```

This private target prevented another worktree from replacing same-version
Cargo artifacts during validation.

Results:

```text
crunch-attestation-core:                         82 passed; 0 failed
mantle release_attestation unit tests:           11 passed; 0 failed
release_cli complete suite:                     149 passed; 0 failed
focused release_cli attest_policy:                7 passed; 0 failed
focused release_cli attest_release_verify:       15 passed; 0 failed
StageX no-quorum label test:                      1 passed; 0 failed
```

The core tests cover optional mode with zero, one, and multiple witnesses. They
also cover successful quorum under each supported independence selector.

Compatibility tests cover `self-proof-only`, `single-witness`, and direct
`mantle-release-policy-v1` construction. The serialized policy fields remain the
existing schema, threshold, selector, and signer lists.

Negative tests cover missing, zero, and unbounded quorum thresholds. They also
cover missing and unsupported selectors, too few identities, duplicate names,
unknown keys, invalid signatures, revocations, stale release references, wrong
rebuilt digests, and destination clobbering.

Invalid profile input creates no policy or revocation file. An optional invalid
witness leaves policy status satisfied and quorum status `not-required`.

## Quality checks

These checks passed:

```text
cargo fmt --check -p mantle -v
cargo clippy -p crunch-attestation-core --all-targets --no-deps -- -D warnings
cargo clippy -p mantle --bin mantle --test release_cli --no-deps -- -D warnings
cargo check -p crunch-attestation-core --target wasm32-wasip2
cargo tigerstyle check -p crunch-attestation-core -- --lib
nickel export schemas/machine-contracts/inventory.ncl
git diff --check
```

The complete first-party Tiger Style check still reports 12 existing findings
in `src/protected_exec.rs` and `src/protected_exec_seccomp.rs`. It reports no
finding in a file changed by this work.

The machine-contract inventory classifies
`attestation.verification-reports` as a compatibility surface. Nickel export of
the updated inventory passed. The broad checker still reports 40 existing root
JSON producers without an inventory decision. It reports no finding for
`src/release_attestation.rs` or this classified family.

The unrelated offline-runbook documentation test passed two of three tests. Its
existing failure requires two missing sentences in the offline build runbook.
This change does not edit that runbook.

## Bootstrap and StageX independence

The StageX no-quorum test passed. It confirms that the technical profile never
emits `quorum-satisfied`.

The full `release_cli` suite passed. This includes provider fixed-point and
release-verification tests. Witness policy does not select those checks.

The broad bootstrap-parity suite passed 15 tests and failed four tests. The
same four tests failed on clean `origin/main` at `ce6fa749`. This comparison
proves that these failures predate this change.

The four baseline failures are:

```text
bootstrap_parity_report_accepts_independently_receipted_early_native_rows
bootstrap_parity_report_accepts_independently_receipted_final_native_rows
bootstrap_parity_report_rejects_stale_generated_artifact_status
bootstrap_parity_report_rejects_untrusted_acceptance_fingerprint
```

## Nix blocker

`nix flake check path:$PWD -L` stopped during evaluation. It could not fetch the
existing private Git input:

```text
error: Failed to fetch git repository 'https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git'
```

The cached detached dev shell supplied the repository toolchain for Cargo,
Nickel, Clippy, Tiger Style, and no-std checks. This does not clear the private
input blocker.

## Cairn lifecycle

Commands:

```text
/home/brittonr/git/OnixResearch/cairn/target/debug/cairn validate --root .
/home/brittonr/git/OnixResearch/cairn/target/debug/cairn gate proposal add-optional-build-witness-policy --root .
/home/brittonr/git/OnixResearch/cairn/target/debug/cairn gate design add-optional-build-witness-policy --root .
/home/brittonr/git/OnixResearch/cairn/target/debug/cairn gate tasks add-optional-build-witness-policy --root .
```

Results:

```text
validate: PASS, no issues
proposal: PASS, receipt 0e9127c0d253fe5a2f9f4c19d68ab6be1e5830603b8005ad2ff739d257f9464a
design:   PASS, receipt 172c344e4cab28483f94cce9df5850c1a62bd69e724aa389390d25efc5f2f187
tasks:    PASS, 15 of 15 complete, receipt c327cbe854ed9309797076928ee5b8a479812e86a2dee6b9ebfdb847d32577f0
```

The local Mantle Tracey profile reported 145 of 148 accepted
release-provenance requirements referenced. The receipt is
`068670c6966065d4ba6a0ba966a545bf655d02cacec4f90a630e809d3ea24c56`.

The three missing references are existing release-provenance gaps:

```text
mantle.release_provenance.content_bound_evidence_manifest
mantle.release_provenance.content_bound_requirement_coverage
mantle.release_provenance.legacy_coverage_boundary
```

## Adversarial review

The final review checked four false-claim risks:

1. A zero threshold cannot resolve to `quorum-satisfied`.
2. Only valid, active, digest-matching witnesses can satisfy a positive quorum.
3. Distinct counts use the selected field and cannot use duplicate domains.
4. Invalid policy parameters fail before policy publication.

The review also checked publication scope. Each file is atomic, but the pair is
not presented as a transactional multi-file commit.

## Claim boundary

A valid witness proves signature and digest agreement for the supplied release
facts and trust inputs. It does not prove source review, compiler correctness,
witness independence beyond the selected field, broad reproducibility, or
release eligibility.

Optional witness presence does not grant quorum admission. Witness policy does
not change bootstrap parity, StageX status, full-source fixed-point status, or
an unrelated release policy decision.
