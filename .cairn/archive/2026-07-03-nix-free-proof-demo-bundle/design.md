## Context

A strong Mantle demo is not the same as a broad compatibility promise. The proof should say exactly what happened: which source root was used, which source-root/toolchain closure policy was enforced, whether Cargo/Nix/rustup were guarded, whether stage1 and stage2 digests matched, and which claims remain out of scope.

## Decisions

### 1. Demo claims are bundle-gated

**Choice:** Human summaries, docs snippets, and release/readiness output may use a Nix-free demo phrase only when a validator accepts the demo bundle profile.

**Rationale:** This prevents an isolated command success or stale transcript from being promoted into a stronger public claim.

### 2. Positive and negative evidence are both required

**Choice:** The demo profile requires fixed-point success evidence and explicit guard-denial evidence for Cargo, Nix, rustup, and undeclared wrappers. Missing negative evidence keeps the bundle useful but not demo-claimable.

**Rationale:** The differentiator is absence of hidden host tools, so the proof must show both what succeeded and what was denied.

### 3. README is generated from machine summary

**Choice:** The operator README is rendered from the same machine summary that validators inspect, with no hand-edited pass/fail fields.

**Rationale:** The copyable story should stay synchronized with the receipt facts and avoid prose drift.

## Risks / Trade-offs

- Requiring negative guard evidence may make the first demo fail until guard fixtures are wired into the proof runner.
- A demo bundle can become large; only concise evidence transcripts and digests should be committed.
- The phrase "Nix-free" must remain scoped to the fixed-point proof profile, not all Mantle behavior.
