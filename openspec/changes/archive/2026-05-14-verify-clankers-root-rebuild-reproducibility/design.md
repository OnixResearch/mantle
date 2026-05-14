## Context

The current Clankers proof records:

- final proof BLAKE3: `264b66f3389075f84373967f5e79738ac4fbbf91d7ddd530d3bf0f412fb276e1`
- original output binary BLAKE3: `e1e8e1c36e0979a2534bcb8c394d4c360068985b0700b0e73ee32f1bd23917ff`
- original observed path: `.crunch-drain/clankers-root-fast-1778743330/store/gkya22mimz0ihj77kc4c17caxnr8qdrc-clankers-0.1.0/bin/clankers`

This change should not rebuild or re-bundle sources unless the fresh-store rebuild requires it. It should compare outputs from the committed derivation and existing pinned bundle.

## Goals / Non-Goals

**Goals:**
- Demonstrate repeatable Clankers root output identity from pinned inputs.
- Record a machine-readable rebuild receipt with original and rebuilt output paths, hashes, command, and verdict.
- Preserve SHA-256 SRI compatibility hashes while using BLAKE3 for the proof comparison.

**Non-Goals:**
- Prove full no-Nix bootstrap.
- Prove cross-machine reproducibility.
- Change Clankers source, features, or native dependency choices.

## Decisions

### 1. Fresh store, same derivation

**Choice:** Rebuild `packages/clankers/clankers.ncl` with a new `.crunch-drain/clankers-root-repro-*` store/state directory.

**Rationale:** This isolates the proof from the previous output directory while keeping the source bundle and derivation fixed.

**Alternative:** Rehash the existing output only. Rejected because it does not prove a repeat build.

### 2. Compare output binary BLAKE3 first

**Choice:** The primary reproducibility verdict is `rebuilt_bin_blake3 == original_bin_blake3`.

**Rationale:** The current final proof hash binds the original output binary digest. Matching the output binary digest is the smallest meaningful proof that the build result reproduced.

**Alternative:** Require whole-output-directory recursive hashing now. Deferred because the immediate proof target is the executable artifact already bound in the proof record.

## Risks / Trade-offs

**Long build time** → Run the build as a managed background task and keep the command in a helper script if needed.

**Host-tool ambiguity** → Report any Nix usage as outer host convenience only, as with the original build.

**Active rename work** → Keep this change scoped and do not modify or archive the unrelated `rename-crunch-to-mantle` change.
