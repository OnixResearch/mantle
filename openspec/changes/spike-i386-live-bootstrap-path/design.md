# Design: i386 live-bootstrap path spike

## Context

Crunch currently models live-bootstrap as per-part NCL derivations on the amd64 path. StageX wraps upstream live-bootstrap stage1 scripts and declares `platforms = ["linux/386"]`. The upstream `steps/manifest` order at the StageX-pinned live-bootstrap commit runs `tcc-0.9.26`, `tcc-0.9.27`, then `make-3.82` before later bootstrap parts.

## Goals / Non-Goals

**Goals:**
- Determine whether an i386-first path is a better unblocker for Make 3.82 runtime validation than continued amd64 TinyCC/Mes repair.
- Keep the decision evidence separate from existing amd64 runtime-validation tasks.
- Preserve Crunch's store/derivation model; StageX container behavior is reference input, not an implementation contract.

**Non-Goals:**
- No wholesale switch during the spike.
- No claim that StageX's stage1 output validates Crunch's amd64 Make part.
- No broad architecture migration without a positive Crunch-local proof.

## Decisions

### 1. Spike before pivot

**Choice:** Add a dedicated OpenSpec spike and evidence trail instead of editing `bootstrap/make-tcc.ncl` again or switching the whole bootstrap chain immediately.

**Rationale:** The prior Make-only repair attempts produced either compile failures or immediate runtime segfaults. A bounded i386 proof can answer whether the canonical upstream path avoids the blocker without destabilizing the amd64 work.

### 2. Proof target stops at Make 3.82 pass1

**Choice:** The initial proof target is `tcc-0.9.27 -> make-3.82 pass1` with `make --version` plus simple Makefile execution.

**Rationale:** This is the exact current blocker boundary. Passing later packages does not matter until Make itself starts and runs recipes.

### 3. Decision criteria

Pivot toward i386-first bootstrap validation only if Crunch-local evidence shows the i386 path can build GNU Make 3.82 and pass the Makefile smoke with less risk than continued amd64 repair. If the i386 path requires a larger architecture rewrite before any proof, continue amd64 repair and keep this spike as documented rejected alternative evidence.

## Validation Plan

1. Audit StageX and pinned upstream live-bootstrap scripts.
2. Identify Crunch NCL/runtime assumptions that would block i386 outputs.
3. Build or prototype the smallest Crunch-local i386 proof target.
4. Record PASS/BLOCKED evidence and update the Make runtime-validation decision.
