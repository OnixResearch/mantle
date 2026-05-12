## Context

The previous probe built `binutils-2.30-tcc`, but direct bwrap smokes failed because binding only the output path omitted the logical `/crunch/store` closure expected by scripts/tools. The existing parity gate already checks for a JSON transcript at `bootstrap/evidence/binutils-tcc-tool-smoke.json`.

## Goals / Non-Goals

**Goals:**
- run a closure-aware smoke probe against real `bootstrap/binutils-tcc.ncl` output;
- write a deterministic transcript only on successful required tool smokes;
- keep parity fail-closed if smokes fail.

**Non-Goals:**
- make `gcc.4.0` native-correct;
- satisfy StageX lineage;
- rewrite binutils build logic beyond tiny probe-enabling fixes.

## Decisions

### 1. Bind the whole scratch store at `/crunch/store`

**Choice:** build into a repo-local scratch store and bind that store root into bwrap as `/crunch/store` for smokes.
**Rationale:** Crunch derivations use logical `/crunch/store` paths; binding only the output path recreates the previous incomplete-closure failure.
**Alternative:** recursively compute and bind individual paths. Rejected for this narrow drain because the scratch store is already the bounded closure arena.

### 2. Transcript is evidence, not a build step

**Choice:** generate JSON from the smoke harness after all required commands pass.
**Rationale:** The transcript should describe observed output behavior; failed smokes should leave no promoting transcript.

## Risks / Trade-offs

**Large scratch store** → Use `.crunch-drain/store-binutils-tcc-transcript` and bounded commands.
**False promotion** → Existing parity tests plus schema checks must reject missing tools, host fallback, or wrong derivation.
