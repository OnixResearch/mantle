## ADDED Requirements

### Requirement: Full-check blocker closure is evidence-bound

r[bootstrap_inventory.full_check_blocker_closure] Mantle MUST clear a bootstrap inventory or fixed-output full-check blocker only from exact current evidence while preserving clean-baseline enforcement, immutable source meaning, and fail-closed behavior for unknown or changed inputs.

#### Scenario: promoted source markers require exact proof identities

GIVEN an independently verified promoted proof names accepted source and receipt bytes
WHEN the blocker inventory classifies marker text in those files
THEN it MUST require exact BLAKE3 identities for the promotion manifest, verification receipt, parity receipt, file path, whole file, and permitted marker class
AND a missing tool, changed byte, unknown path, unknown class, or stale receipt MUST restore an actionable finding.

#### Scenario: structural non-blockers remain narrow

GIVEN source text contains a negative bridge-use fact, bounded timeout control, successful rejection summary, or positive smoke name
WHEN the lexical inventory sees a blocker word
THEN it MAY classify only that explicit structural form as non-actionable
AND positive bridge use, observed timeout diagnostics, crashes, and near-miss text MUST remain actionable.

#### Scenario: fixed-output repair uses current immutable bytes

GIVEN a prior builder reported a fixed-output mismatch
WHEN Mantle repairs the Nix input
THEN a fresh local build MUST independently identify the current immutable bytes and a repeated build MUST confirm the selected hash
AND the repair MUST NOT change the source revision, lockfile, toolchain, features, or package meaning to fit a cached hash.

#### Scenario: full checks expose the next real blocker

GIVEN the classified inventory and repaired fixed outputs pass focused checks
WHEN local-builder and ordinary `nix flake check -L` run
THEN Mantle MUST preserve any later independent failure with its exact derivation and diagnostic
AND it MUST NOT disable, baseline, or downgrade a failing check to report-only success.
