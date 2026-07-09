# Oxide release and worker pattern intake

Mantle uses the following repositories as reference-only prior art for release
metadata, ephemeral worker receipts, and cancellation-safe async orchestration.
They inform Mantle-owned evidence contracts; they are not proof authorities for
Mantle release correctness, reproducibility, build soundness, worker cleanup, or
artifact truth.

## Reference inventory

| Source | Intended adaptation | License/trust posture | Non-claim boundary |
|---|---|---|---|
| `oxidecomputer/tufaceous` | TUF-style release repository roles, signed target metadata, compatibility metadata, and artifact tag review shape | Reference-only architecture input; Mantle keeps its BLAKE3 identities, release manifest contract, and verifier policy | Tufaceous behavior does not prove Mantle release correctness, target metadata correctness, or release eligibility |
| `awslabs/tough` | TUF client/server metadata validation shape, expiration checks, target binding, and trust-root handling | Reference-only implementation input; Mantle does not delegate release verification semantics to Tough | Tough behavior does not prove Mantle artifact truth, reproducibility, or stack semantics |
| `oxidecomputer/buildomat` | Ephemeral worker job receipts, trusted-policy separation, captured logs, artifact delivery state, cleanup outcome, and replayable event ids | Reference-only worker architecture input; Mantle policy/config remains authoritative and candidate inputs cannot override trusted worker policy | Buildomat behavior does not prove Mantle build soundness, worker cleanup, or output acceptance |
| `oxidecomputer/cancel-safe-futures` | Cancellation-aware async adapter patterns for preserving cleanup, upload integrity checks, and final receipt emission | Reference-only cancellation design input; Mantle classifies cancellation outcomes in its own receipts | The reference crate does not prove Mantle sessions are cancel-safe or that artifacts are accepted after interruption |

## Mantle-owned contract shape

The pure core in `crunch-release-core::validate_oxide_release_worker_fixture`
validates already-loaded fixture data for four profile families:

- reference intake rows with source, adaptation, license/trust posture, and a
  visible `not proof authority` non-claim;
- TUF-style release repository profiles with signed metadata identity, expiration
  policy, trust root, release targets, and typed tag matching;
- ephemeral worker receipts with input identity, target profile, worker identity,
  log identity, artifact identities, cleanup outcome, and replayable event id;
- cancellation outcomes that require cleanup attempts, final receipt persistence,
  and artifact-integrity proof before mid-upload artifacts can be accepted.

Negative fixtures cover expired metadata, artifact tag mismatch, untrusted policy
override, worker cleanup failure, mid-upload cancellation without integrity, and
overclaim text. Passing the fixture validator proves only that the declared
Mantle evidence rows are internally bounded; it does not prove release
correctness, reproducibility, build soundness, worker cleanup, artifact truth, or
external project semantics.
