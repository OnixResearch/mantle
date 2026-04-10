## Context

crunch already has a checked-in self-hosting proof and a self-build report that
records which binary drove each stage plus which sandbox tools were selected.
What was still missing was tighter binding for the proof inputs:

- stage2 could still rediscover source from the checkout if the proof runner
  happened to execute from the repo
- the generated self-build derivation could rediscover `bwrap` and `busybox`
  by scanning a shared store for the first matching sibling output

That makes the proof less audit-friendly than it should be. The proof should
bind to one staged source tree and one pair of exported bootstrap-tool outputs.

## Goals / Non-Goals

**Goals:**
- Reuse the exact stage0 staged source tree during stage2
- Make staged-source reuse stable even when the proof changes cwd
- Reject reused staged trees whose contents no longer match their staged store
  name
- Thread the exact exported `bwrap` and `busybox` outputs into the final
  self-build derivation
- Make the proof fail loudly if it would otherwise fall back to host source
  staging from the repo checkout or host PATH

**Non-Goals:**
- New user-facing self-build flags
- A byte-for-byte stage1 == stage2 fixed-point proof
- General attestation or supply-chain metadata work

## Decisions

### 1. Reuse the staged source tree by explicit path

**Choice:** `crunch self-build` accepts a hidden `--source-store-path` used only
by proof and internal workflows.

**Rationale:** the proof already produces a staged `*-crunch-src` tree in the
proof store. Stage2 should consume that exact tree instead of restaging from
whatever checkout happens to be on disk.

### 2. Validate reused staged source trees against their staged hash name

**Choice:** reused staged-source directories are validated in two ways:
- they must live directly under the proof store
- their current contents must still hash to their `*-crunch-src` store name

**Rationale:** path reuse alone is too weak. The proof needs to reject a staged
source tree that was modified in place after stage0.

### 3. Normalize proof paths before reuse

**Choice:** self-build absolutizes the output store and reused staged-source
path before emitting or validating proof markers.

**Rationale:** the proof intentionally launches stage2 from outside the repo.
Relative store paths should not change meaning when cwd changes.

### 4. Bind the final derivation to exact bootstrap-tool outputs

**Choice:** after step 2 builds `bwrap.ncl` and `busybox.ncl` as root
outputs, step 3 threads those exact store-entry names into the generated
self-build derivation.

**Rationale:** rescanning a shared proof store for `*-bwrap` or `*-busybox`
can pick stale siblings. The proof should report and use the same exact tool
outputs.

### 5. Keep host isolation in the proof runner

**Choice:** the stage2 proof run happens from outside the repo with `PATH=""`.

**Rationale:** if stage2 regresses to host `git`, `cargo`, `tar`, `cp`, or
checkout-relative staging, the proof should fail immediately instead of
passing by accident.

## Risks / Trade-offs

**[Validation cost]** Re-hashing a reused staged source tree adds work before
stage2 starts. That cost is acceptable because the proof path is already a
slow, explicit workflow.

**[Hidden flag misuse]** `--source-store-path` is internal and could be used
incorrectly by a human. Mitigation: keep it hidden, validate aggressively, and
only document it as proof plumbing.

**[Shared-store ambiguity]** Exact root-output binding removes one source of
nondeterminism, but the proof still depends on a writable shared store.
Mitigation: assert against the exact reported tool paths during stage2.
