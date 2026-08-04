# Design: Add optional build-witness policy

## Context

The canonical release and witness attestations are technical evidence. `mantle-release-policy-v1` is the social-policy layer. The policy already supports a zero minimum, but the CLI cannot scaffold a zero-minimum policy that also names trusted witness identities.

The current `self-proof-only` profile therefore means both “no witness required” and “do not trust witness identities.” The current `single-witness` profile enables witness trust and immediately makes one witness mandatory. Mantle needs a stable state between those profiles.

## Decisions

### Decision: Add an explicit optional-witness profile

**Choice:** `optional-witness` scaffolds `mantle-release-policy-v1` with a zero matching-witness minimum. It may contain trusted witness identities and uses a supported independence selector for classification only.

**Rationale:** A valid witness remains useful evidence even when release admission does not depend on witness count. A zero threshold must not forbid evidence collection.

### Decision: Keep quorum opt-in

**Choice:** `witness-quorum` requires an explicit positive `--min-matching-witnesses` value and one supported `--independence-field`. Policy construction rejects missing, zero, unsupported, or unbounded values before writing files.

`single-witness` remains a compatibility profile that expands to a named minimum of one and the existing witness-identity selector.

**Rationale:** Operators can choose a stronger gate without making it the project default. Explicit parameters prevent an accidental policy change when witness files appear.

### Decision: Preserve schema compatibility

**Choice:** This change keeps `mantle-release-policy-v1`. It adds profile scaffolding and report semantics around existing fields instead of changing canonical attestation or policy bytes.

**Rationale:** The current policy already represents a zero or positive minimum. A schema revision is not needed for this bounded change.

### Decision: Report evidence and policy separately

**Choice:** Human and JSON output preserves discovered, signature-valid, matching, skipped, failed, and revoked witness facts. It also reports quorum status as `not-required`, `satisfied`, or `insufficient`.

In optional mode, an absent or excluded witness does not fail release policy solely because no witness is required. Invalid evidence never becomes accepted evidence.

**Rationale:** Operators need both technical facts and policy status. One failed optional witness must not erase a valid release or other valid witnesses.

### Decision: Keep bootstrap and witness decisions independent

**Choice:** `--require-stagex-no-quorum`, bootstrap parity, and full-source fixed-point verification do not select or imply a witness quorum profile. A caller may evaluate the same release later under an explicit witness-quorum policy.

**Rationale:** Bootstrap evidence and external rebuild agreement prove different facts. Neither can substitute for the other.

### Decision: Preserve the functional core and imperative shell

**Choice:** A pure profile-construction core validates profile mode, minimum count, selector, trusted identities, and bounds. The CLI shell parses arguments, reads and writes policy files, and renders output.

**Rationale:** Policy behavior must be testable without filesystem, process, clock, network, or environment state.

## Compatibility

- `self-proof-only` keeps its current zero-witness and no-trusted-witness behavior.
- `single-witness` keeps its current one-witness behavior.
- Existing hand-authored `mantle-release-policy-v1` files remain valid.
- Existing final verification semantics remain authoritative when an explicit positive minimum is present.

## Risks / Trade-offs

- Operators can misread optional witness presence as release admission. Output must show `not-required` clearly.
- One independence selector cannot express cross-axis diversity. Reports retain other metadata, but this change does not count it.
- Tolerating invalid optional witness files can hide operational problems unless summaries keep failed and skipped counts visible.
- A future multi-axis policy requires a separately versioned contract and migration plan.

## Claim Boundary

A valid optional witness proves only that the signed witness attestation matches the exact release facts under the supplied trust inputs. It does not prove witness independence, source review, compiler correctness, broad reproducibility, or release eligibility under a policy that was not selected.
