## Context

The blocker inventory intentionally scans source and evidence text. Its simple
marker taxonomy now matches accepted V98 StageX names and normal bounded test
controls. The inventory reports 115 findings even though the independent parity
verifier accepts all required axes with no blockers.

A clean result is valid only if the checker can distinguish accepted immutable
bytes from new or changed blocker text. Fixed-output repair has a similar rule:
only an independently rebuilt immutable source can justify a hash update.

## Decisions

### Decision: keep clean-baseline enforcement unchanged

**Choice:** Preserve `--enforce --require-clean` in the Nix check. Do not remove
marker classes or exclude bootstrap directories.

**Rationale:** The check must still reject new blockers and promotion drift.

### Decision: separate structural facts from proof-bound classification

**Choice:** Recognize only explicit negative bridge-use facts, bounded timeout
controls, successful negative-test summaries, and positive smoke names without
proof lookup. Treat all other accepted V98 markers as proof-bound.

**Rationale:** A negative boolean and a timeout budget are not blocker
observations. Broader lexical exceptions would hide real failures.

### Decision: bind accepted markers by whole-file BLAKE3

**Choice:** Activate V98 classification only when the independent manifest,
verification receipt, parity receipt, and every declared file match reviewed
BLAKE3 identities. Limit every file to named marker classes.

**Rationale:** Whole-file identity makes any source drift fail closed. Unknown
paths and classes cannot inherit authority.

### Decision: repair fixed outputs from fresh local bytes

**Choice:** Build each previously mismatched output with local builders and no
secret keys. Change a hash only when Nix reports the actual immutable bytes and
a second build confirms them.

**Rationale:** A remote mismatch can be stale cache data or a stale pin. The
current local derivation is the smallest independent discriminator.

## Risks / Trade-offs

- Any proof-bound file edit reopens its findings until new proof evidence is
  reviewed.
- The checker depends on `b3sum`; absence fails closed.
- Full flake checks can expose later independent failures after the first gate is
  repaired.
- This work does not strengthen the V98 claim or prove compiler correctness.
