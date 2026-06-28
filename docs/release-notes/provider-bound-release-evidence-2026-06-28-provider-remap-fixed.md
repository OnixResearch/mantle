# Provider-bound release evidence, 2026-06-28

Release ID: `provider-bound-release-evidence-2026-06-28-provider-remap-fixed`

## Status

Mantle has a provider-bound release evidence bundle with a successful provider fixed-point proof and a successful local/provider-bound witness replay. The configured single-witness policy verified as `quorum-satisfied`.

This is a bounded provider-bound claim. Do not describe it as an external independent rebuild unless the `britton-desktop-provider-witness` identity is operated independently from the publisher.

## Durable artifact storage

A durable local copy of the ignored runtime artifacts is stored at:

```text
/home/brittonr/releases/mantle/provider-bound-release-evidence-2026-06-28-provider-remap-fixed
```

Contents:

- `release-evidence/` — release evidence bundle.
- `release-verification/` — signed release attestation, policy, imported witness, and `final-release-verify.json`.
- `witness-rebuild-audit/` — provider-bound witness replay audit.

## Verification summary

Final verification record:

```text
target/release-verification/provider-bound-release-evidence-2026-06-28-provider-remap-fixed/final-release-verify.json
```

Key fields:

```text
technical_class: external-witness-match
policy_status: satisfied
final_class: quorum-satisfied
independent_agreement_status: satisfied
independent_agreement_class: independent-rebuild-agreement
independent_agreement_counted_witness_count: 1
```

Matched output digests:

```text
binaries/01-mantle: aa55e64630390fbb1f1f2ab2102005b1e05ef07c8ed19325a00566631c626a20
binaries/02-stage2-mantle: 70f02150224073af2dfdabad5b697072a6399c361c8b448f0e17ee01431fc703
```

Release attestation digest:

```text
c16aad67e2981a178e739c52dc07617c90976e2c8d2a566659e008bdf9542880
```

## Operator note

For a stronger external claim, send the witness request to a separate operator or machine and import their returned sidecars before publishing external-independent wording.
