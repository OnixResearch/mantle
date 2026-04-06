## Context

7 spec contradictions found during audit. Code is correct in each case;
specs weren't updated when implementation decisions diverged.

## Goals / Non-Goals

**Goals:** Make every spec match the code and each other. Zero
contradictions.

**Non-Goals:** Change any code. Add new features. Write new specs.

## Decisions

### 1. Update nickel-eval spec: JSON intermediate is intentional

**Change:** Remove "MUST NOT use JSON as intermediate format." Add a
requirement documenting the JSON round-trip and why: `Expr::to_serde()`
fails on Nickel enum tags in nested derivation inputs. JSON export
handles this correctly.

### 2. Update nickel-stdlib spec: contract closure deferred

**Change:** Document that the contract is currently open (`..`) to
support mkDerivation fields, and will be re-closed when
extract-stdlib-builders lands. Add a cross-reference.

### 3. Update ca-derivations spec: provisional path strategy

**Change:** Replace "MUST assign provisional output paths using
hash_placeholder(output_name)" with the actual approach: input-addressed
provisional paths from the derivation environment, with blake3-derived
markers for self-reference rewriting. Document why hash_placeholder is
wrong (different-length string).

### 4. Update persistent-pathinfo spec: castore cache check

**Change:** Remove the filesystem existence requirement from cache
check. Reference the castore-store spec as authoritative. Cache hit =
PathInfo + castore content probe.

### 5. Update nickel-stdlib spec: sandbox default

**Change:** Change `default = 'wasm` to `default = 'native` in the
Sandbox section. Reference the portability spec's rationale (WASI lacks
process spawning).

### 6. Update fetchers spec: --fix behavior

**Change:** Replace "continue the build" with "exit with an error
instructing the user to re-run." Document why: the derivation path
changes after hash rewrite, so the current session's KnownPaths and
goal registry are stale.

### 7. Update architecture spec: add crunch-build

**Change:** Add `crunch-build` to the crate layout table with role
"Goal scheduler, Worker, build dispatch, output processing."

## Risks / Trade-offs

None. Documentation-only change.
