## Context

The parent repair attempted:

`timeout 360 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute --store .crunch-drain/make-repair-v2-store --state-dir .crunch-drain/make-repair-v2-state bootstrap/make-tcc.ncl`

The run generated a signing key and then timed out with exit status 124 before producing a build output or enough transcript to run smoke/leakage checks. Earlier preflight also showed the upstream GNU and Savannah primary URLs were unreliable, so the parent switched to stable mirrors while keeping fixed-output hashes unchanged.

## Goals / Non-Goals

**Goals:**

- Obtain a completed or diagnostically useful Crunch build transcript for `bootstrap/make-tcc.ncl`.
- If successful, identify the output path and run make 3.82 runtime smokes.
- Scan for undeclared host-tool, host-path, or environment leakage.
- Preserve evidence sufficient to close the parent runtime proof.

**Non-Goals:**

- Reopening source-pin hardening already completed in the parent.
- Treating a timeout with no output as PASS evidence.
- Broad downstream bootstrap validation beyond the first make pass.

## Decisions

### 1. Long-running validation boundary

**Choice:** keep long build/smoke/leakage proof in this successor rather than blocking the parent source-level repair.

**Rationale:** the parent has completed the actionable source and mirror fixes; repeated short timeouts only add noise. A separate runtime-validation change can use a longer-lived background process and preserve progress without mixing source edits with runtime proof bookkeeping.

### 2. Evidence ordering

**Choice:** V1 must produce the build output before V2 and V3 can run.

**Rationale:** version/smoke and output leakage checks require the actual built make path. If V1 fails, V2/V3 should record blocked/failing evidence rather than fabricate checks.

## Validation Plan

1. Run `crunch build bootstrap/make-tcc.ncl` with fresh local store/state and a long-running budget.
2. Preserve the full transcript and output path or failure diagnostics.
3. Run `bin/make --version` on the output and require `GNU Make 3.82`.
4. Run a simple Makefile positive smoke and missing-target negative smoke.
5. Scan `bootstrap/make-tcc.ncl`, the build transcript, and output metadata for undeclared host leakage.
6. Run OpenSpec verification for this successor.
