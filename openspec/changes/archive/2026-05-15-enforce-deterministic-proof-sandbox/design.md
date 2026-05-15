## Context

The existing deterministic proof flow runs a rebuild command multiple times with separate output and store directories. That prevents simple output reuse, but it does not establish a complete proof boundary. The rebuild command may still see the host filesystem, host PATH, network, current working directory, and unrecorded tools.

## Goals / Non-Goals

**Goals:**
- Make deterministic proof runs fail closed unless executed through an explicit sandbox envelope.
- Make the sandbox envelope canonical and digest-addressed in the proof receipt.
- Deny network by default and require explicit future OpenSpec work for networked fixed-output proof cases.
- Keep claims bounded: sandboxed release-artifact proof, not global Mantle/Nix-like determinism.

**Non-Goals:**
- Prove every normal `mantle build` is globally deterministic.
- Replace snix-build/bwrap derivation sandboxing.
- Support arbitrary host rebuild recipes without declaring their tool/input needs.
- Prove CPU/kernel/filesystem equivalence across machines.

## Decisions

### 1. Dedicated proof sandbox envelope

**Choice:** Deterministic proof orchestration will build a canonical sandbox profile for each proof run and execute the rebuild recipe through that profile.

**Rationale:** Release proof rebuilds are operator-supplied workflows, not necessarily Mantle derivations. They need their own proof boundary instead of inheriting assumptions from derivation builds.

**Implementation:** Start with a Linux bwrap-backed envelope when available. The envelope binds the release bundle read-only, the selected rebuild command/tool read-only, a fresh output directory writable, and a fresh proof store directory writable. It supplies only the Mantle reproducibility env vars and minimal execution environment.

### 2. Fail closed rather than silently downgrade

**Choice:** Requested deterministic proof runs MUST fail when sandbox execution is unavailable or unsupported.

**Rationale:** Producing a deterministic proof receipt from an unsandboxed process would overstate evidence. Operators can still run ordinary `release reproduce` for `self-rebuild-match` evidence.

### 3. Canonical sandbox profile digest

**Choice:** Receipts record a canonical sandbox profile digest covering executor kind/version, network policy, bind mounts, writable directories, env allowlist, working directory policy, and recipe identity.

**Rationale:** Reviewers and verifiers need to know what boundary was actually used and be able to reject unsupported/bypassed profiles.

## Risks / Trade-offs

**Tool availability:** bwrap may be absent in some developer shells. Mitigation: deterministic proof mode fails with a targeted diagnostic; non-deterministic release reproduce remains available.

**Existing arbitrary scripts:** host-dependent scripts may fail once sandboxed. Mitigation: docs explain that deterministic proof recipes must declare or bundle their tools.

**Overclaiming:** Even a sandboxed proof still depends on kernel/CPU/filesystem behavior. Mitigation: receipts and docs keep those assumptions explicit and bounded.
