## Context

The authoritative command is `cargo-deny check --config deny.toml`. Its current failures come from independent policy mechanisms: advisory resolution, source admission, and SPDX license admission. A single blanket exception would make review easier but would weaken the evidence contract.

A dry-run targeted update confirms `crossbeam-epoch 0.9.20` satisfies the existing dependency constraints. The `nickel-export-core` Git source is already bound to revision `257fafc1c746f1faf156207043a4c826bfb16d49` by Cargo, Nix, generated source metadata, accepted Cairn requirements, and dedicated pin checks. `winx 0.36.4` is transitive through the capability filesystem stack and declares `Apache-2.0 WITH LLVM-exception`.

## Decisions

### 1. Fix the actionable vulnerability without a waiver

**Choice:** Generate a targeted `Cargo.lock` update from `crossbeam-epoch 0.9.18` to the compatible fixed release `0.9.20` and add no `RUSTSEC-2026-0204` ignore entry.

**Rationale:** The advisory provides a compatible fixed version, so a waiver would preserve avoidable vulnerable code and violate the existing minimal-safe-movement policy.

### 2. Source policy names the reviewed repository, not all Git sources

**Choice:** Add `https://github.com/OnixResearch/nickel-export` to `sources.allow-git` while leaving `unknown-git = "deny"` and all exact-revision checks intact.

**Rationale:** Source policy answers whether the repository is approved; Cargo/Nix/spec pin checks separately prove the selected immutable revision. Approving only the repository closes the configuration gap without admitting arbitrary Git dependencies or floating revisions.

### 3. License policy names the exact compound SPDX expression

**Choice:** Add `Apache-2.0 WITH LLVM-exception` to the license allow list without adding a crate-wide exception, wildcard, or lower confidence threshold.

**Rationale:** The expression is the dependency's declared SPDX license and is narrower than bypassing license checks for `winx` or allowing unknown license text.

### 4. Positive and negative evidence stay paired

**Choice:** Require the full checked-policy audit plus negative evidence that missing/default policy remains non-authoritative and that immutable Nickel export pin checks still reject drift.

**Rationale:** A green audit is insufficient if it was achieved through a missing policy, a blanket source rule, or weakened pin enforcement.

## Risks / Trade-offs

- A targeted lock update still changes compiled transitive code; locked compile and focused consumer tests are required.
- Approving a Git repository does not prove the selected commit is trustworthy; exact revision and source-identity checks remain separate mandatory evidence.
- License policy admission records review compatibility only; it does not prove legal conclusions for downstream distribution.
- The advisory database can change during validation, so final evidence must record any new findings rather than claiming a timeless clean state.
