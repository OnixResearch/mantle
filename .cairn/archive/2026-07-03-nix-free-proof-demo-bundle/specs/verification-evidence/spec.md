## ADDED Requirements

### Requirement: Nix-free proof demo bundle

r[verification_evidence.nix_free_proof_demo_bundle] Mantle MUST gate operator-facing Nix-free fixed-point demo claims on a validated proof bundle that contains current positive fixed-point evidence, current negative host-tool guard evidence, and explicit non-claims.

#### Scenario: successful demo bundle is claimable

GIVEN a source-root Cargo-free fixed-point proof bundle contains matching stage1 and stage2 Mantle binary BLAKE3 digests
AND the bundle records source-root identity, toolchain-closure policy digest, command-owned wrapper digests when present, Cargo guard status, Nix guard status, rustup guard status, and ambient-wrapper guard status
WHEN Mantle validates the bundle for the Nix-free demo profile
THEN validation MAY mark the bundle as demo-claimable.
AND the generated operator summary MUST include the exact fixed-point verdict, stage digests, guard results, source-root/toolchain policy digests, replay command hints, and non-claims.

#### Scenario: missing fixed-point evidence blocks the claim

GIVEN a proof bundle lacks successful stage1/stage2 digest comparison evidence
WHEN Mantle validates the bundle for the Nix-free demo profile
THEN validation MUST reject demo claimability with a deterministic missing-fixed-point-evidence diagnostic.
AND human summaries MUST NOT describe the bundle as Nix-free fixed-point evidence.

#### Scenario: missing guard evidence blocks the claim

GIVEN a proof bundle lacks a required Cargo, Nix, rustup, or ambient-wrapper denial record
WHEN Mantle validates the bundle for the Nix-free demo profile
THEN validation MUST reject demo claimability with a deterministic missing-guard-evidence diagnostic.
AND the bundle MAY still be described only as a narrower fixed-point or blocker artifact when that narrower evidence is present.

#### Scenario: generated README follows machine summary

GIVEN Mantle emits a demo-profile operator README
WHEN the README is compared with the machine summary
THEN pass/fail fields, digests, guard statuses, command hints, and non-claims MUST be derived from the machine summary.
AND hand-edited README content MUST NOT be the source of truth for demo claimability.

#### Scenario: demo remains bounded

GIVEN a Nix-free proof demo bundle validates successfully
WHEN docs, release-readiness output, status replies, or task evidence cite it
THEN the claim MUST be limited to the recorded source-root Cargo-free fixed-point proof profile.
AND it MUST NOT claim compiler correctness, full Cargo compatibility, full bootstrap source minimization, release reproducibility, deploy success, or general Nix replacement completeness without separate evidence.
