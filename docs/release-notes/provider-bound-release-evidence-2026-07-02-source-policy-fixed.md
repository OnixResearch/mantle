# Provider-bound release evidence with Aspen external witness, 2026-07-02

Release ID: `provider-bound-release-evidence-2026-07-02-source-policy-fixed`

## Status

Mantle has a fresh provider-bound release evidence bundle whose public source archive now matches the self-build staging policy. Aspen replayed the exported request as `aspen1-external-witness`, returned signed sidecars, and final verification reached `independent-rebuild-agreement` with one policy-counted external witness.

This supersedes the blocked `provider-bound-release-evidence-2026-06-28-provider-remap-fixed` external handoff attempt. That older release bundle remains historical provider-bound evidence, but it must not be described as externally reproduced because its original proof staged `vendor/.pi/prompt-history.jsonl`, which was not present in the exported public source archive.

## Durable artifact storage

A durable local copy of the ignored runtime artifacts is stored at:

```text
/home/brittonr/releases/mantle/provider-bound-release-evidence-2026-07-02-source-policy-fixed
```

Contents:

- `release-evidence/` — fresh release evidence bundle.
- `release-verification/` — signed release attestation, policy, imported Aspen witness, and final verification JSON.
- `external-witness-request-2026-07-02/` — public-only witness request sent to Aspen.
- `external-witness-aspen1-2026-07-02/` — copied Aspen sidecars and rebuild logs.
- `negative-missing-signature/` — fail-closed negative import check.

## Verification summary

Final verification record:

```text
/home/brittonr/releases/mantle/provider-bound-release-evidence-2026-07-02-source-policy-fixed/final-release-verify.json
```

Key fields:

```text
technical_class: external-witness-match
policy_status: satisfied
final_class: quorum-satisfied
independent_agreement_status: satisfied
independent_agreement_class: independent-rebuild-agreement
independent_agreement_counted_witness_count: 1
witness_identity: aspen1-external-witness
signer_key_name: crunch-aspen1-1
signature_valid: true
digest_match: true
policy_counted: true
```

Matched output digests:

```text
binaries/01-mantle:        8127c7952dc61a12ec38c39d2aef1a51be9c431962d9991d882089e85889e4cf
binaries/02-stage2-mantle: d555cb68327adb9ef6341646062250245037fee52b8a3fffe394f4e6a9dfcd71
```

Release attestation digest:

```text
7557adc8ea544fb15193ed8cec18804a149a4df3e22597e2ad46801604a45d84
```

Public witness request digest:

```text
1f5f4f527bcd0499b98682f8f62189a7c341477f58d946aab865876d4a30eeb5
```

## Bounded claims and non-claims

Claimed:

- The packaged release evidence verifies locally.
- The bundled provider fixed-point proof is valid and matches `binaries/01-mantle`.
- The bundled self-hosting proof is fixed-point, strict for stage2, and matches `binaries/02-stage2-mantle`.
- Aspen returned signed witness sidecars for `aspen1-external-witness`; final policy counted that witness and classified the release as `independent-rebuild-agreement`.

Not claimed:

- This does not prove full compiler correctness.
- This does not prove global reproducibility for all Mantle builds; that claim requires an eligible `mantle-global-reproducibility-report-v1` for an explicit universe and policy.
- This does not make the superseded 2026-06-28 request externally reproducible.
- This does not claim the provider fixed-point proof by itself is a full bootstrap proof; it remains bounded source-built handoff evidence.

## Negative check

A missing-signature import check failed closed:

```text
exit_status=3
error: missing witness signature sidecar .../aspen1-external-witness.json.sig: No such file or directory (os error 2)
```
