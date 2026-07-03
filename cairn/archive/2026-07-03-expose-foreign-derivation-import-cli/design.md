## Context

`src/foreign_derivation_import.rs` defines the in-memory core and fixtures. The next useful slice is an operator surface that reads explicit JSON artifacts, invokes the core, and prints deterministic JSON output. This should look like the existing demo-bundle CLI pattern: thin shell, pure validation core.

## Decisions

### 1. CLI consumes artifacts, not frontends

**Choice:** `foreign-import` commands take graph/index/policy files that are already lowered into Mantle's neutral IR.

**Rationale:** The accepted boundary says Mantle consumption must not depend on Guix, Nix, flakes, overlays, or evaluator semantics.

### 2. Validate and plan are separate commands

**Choice:** `validate` checks schema, references, policy admission, and receipt consistency. `plan` emits the accepted translated graph receipt plus a Mantle adapter plan.

**Rationale:** Operators often need a cheap preflight before producing a build-plan artifact.

### 3. Fixtures are files, not hidden test constructors

**Choice:** Check in small Guix-like and Nix-like hello fixture JSON files and use them in CLI integration tests.

**Rationale:** File fixtures make the CLI contract reviewable and reusable by docs.

### 4. Error output stays deterministic

**Choice:** CLI errors should surface the core diagnostic class, path, and message in JSON mode and bounded human text otherwise.

**Rationale:** Operator automation needs stable failure classes.

## Risks / Trade-offs

- JSON artifact schemas may evolve; schema versions must be explicit.
- Pretty output can tempt overclaiming. CLI docs must keep receipt non-claims visible.
- Large real-world graphs are out of this first slice and may need streaming later.
