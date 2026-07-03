## Context

`mantle release global-reproducibility` already evaluates a universe/policy/evidence set and fails closed. The missing operator seam is evidence preparation: release bundles already contain manifest, proof, provider proof, release attestation, witness attestation, and final verification facts, but today a global surface evidence file must be assembled manually.

## Decisions

### 1. Helper derives evidence, evaluator remains authoritative

**Choice:** Add `mantle release global-reproducibility-evidence` as a thin shell that loads release artifacts and writes surface evidence. It does not decide eligibility. Operators still run `mantle release global-reproducibility` to produce the final report.

**Rationale:** Keeping admission in the existing evaluator avoids duplicating policy logic. The helper only normalizes release facts into the existing evidence schema.

### 2. Pure derivation core over loaded release facts

**Choice:** Split the helper into a pure derivation core over in-memory manifest/attestation/verify structs and a shell that reads paths, hashes files, and writes JSON.

**Rationale:** Release fact mapping is deterministic and testable without filesystem mocks, while path reads and report writing stay in the CLI shell.

### 3. Provider fixed-point handoff stays blocked for strict global release universes

**Choice:** When a release artifact is backed only by provider fixed-point handoff evidence, the helper emits surface evidence with an `unsupported_reason` rather than marking strict/fresh global evidence true.

**Rationale:** The provider proof is valuable release-bounded evidence, but current docs explicitly say it does not by itself prove full bootstrap/release/global reproducibility. A full release universe should preserve that blocker honestly.

## Risks / Trade-offs

- The helper trusts the supplied final release-verify JSON as already-produced verification evidence; it does not re-run signature verification. Operators needing current cryptographic verification should run `mantle attest release-verify --json` immediately before deriving evidence.
- The first helper recognizes current release bundle shapes and explicit artifact paths. Future artifact classes may need additional derivation modes.
