# Design: Compile concrete foreign graphs before realization

## Context

`foreign-derivation-graph-v1` preserves concrete foreign graph facts. The current adapter plan does not contain executable native derivation units.

The Nix-compatible ATerm parser also requires `/nix/store`. It cannot parse a concrete Guix `.drv` that uses `/gnu/store`.

Mantle already computes target derivation and output paths in `crunch-glue`. It also registers completed native derivations for the scheduler. The missing seam is a pure compiler between the foreign IR and that registration model.

## Decisions

### Decision 1: Keep parsing and graph compilation in the pure core

**Choice:** Add prefix-aware ATerm parsing and graph compilation as pure functions over owned bytes, graph facts, and explicit policy.

The CLI shell will read files, decode configuration, and write artifacts. The core will not read files, inspect the environment, contact networks, or execute processes.

**Rationale:** Producers can use Guix or Nix before artifact emission. Consumption must not require either frontend.

### Decision 2: Own a prefix-neutral parser boundary

**Choice:** Add a Mantle-owned prefix-neutral parser module. Use the MIT-licensed `guix-transfer` parser and fixtures as a design reference.

The parser will receive one declared source prefix. It will reject paths outside that prefix and malformed store objects. Direct edits to vendored `nix-compat` parsing require its separate vendor synchronization process.

**Rationale:** The current parser hard-codes `/nix/store`. A prefix-neutral boundary also supports future concrete store formats.

### Decision 3: Use bounded dependency ordering with full coverage checks

**Choice:** Compute dependency order with a bounded deterministic algorithm. The compiler will require every reachable node to appear exactly once.

It will reject cycles, missing inputs, duplicate roots, duplicate edges, output-name mismatches, and graph limits before target registration.

**Rationale:** A visited set alone can hide cycles. Full emitted-node coverage makes incomplete ordering observable.

### Decision 4: Rewrite exact mapped store objects

**Choice:** Maintain separate maps for foreign derivations, outputs, and sources. Rewrite a recognized store object only after its exact target identity exists.

The scanner will preserve a path suffix after the mapped store object. It will reject every unknown or remaining declared foreign store path.

**Rationale:** Prefix replacement fabricates target paths. Exact maps preserve child-to-parent identity.

### Decision 5: Register resolved native derivations

**Choice:** Refactor `crunch-glue` with a pure resolved-registration helper. It will accept one complete `nix_compat::Derivation`, known input identities, target store prefix, addressing mode, and provenance facts.

The helper will compute the hash derivation modulo, output paths, derivation path, and pending registry entry. It will not use the current preliminary `name:builder:system` identity.

**Rationale:** Foreign nodes are already flat graph units. Rebuilding nested `CrunchDerivation` values adds weak identity and recursion hazards.

### Decision 6: Lower a bounded builtin set

**Choice:** The first compiler version will support `builtin:download` and declared Git download forms. It will map them to Mantle fetch requests with ordered source candidates and fixed-output facts.

All other builtins will fail with stable unsupported-builtin diagnostics.

**Rationale:** Explicit support is safer than approximating foreign builtin behavior.

### Decision 7: Separate foreign and Mantle hash domains

**Choice:** Preserve source-format SHA-256, NAR, fixed-output, and original derivation facts. Compute target paths and Mantle artifact digests with Mantle’s configured BLAKE3 domain.

The executable plan will label every digest domain and algorithm.

**Rationale:** Interoperability values and Mantle-owned identity serve different purposes.

### Decision 8: Emit a plan, not realization evidence

**Choice:** Emit `mantle-foreign-executable-plan-v1`. It will bind the admitted import receipt, selected roots, resolved native units, exact path maps, source requirements, execution-profile references, and non-claims.

The plan will not claim source availability, scheduler execution, store admission, or output trust.

**Rationale:** Compilation is a deterministic planning step. Realization has separate effects and evidence.

## Failure Semantics

- A malformed ATerm bundle fails before partial artifacts are written.
- A missing input, cycle, duplicate mapping, or unknown output fails graph compilation.
- An unknown foreign store path fails exact rewriting.
- An unsupported builtin fails before native unit registration.
- A hash-domain mismatch fails before plan identity is computed.
- The CLI writes no successful executable plan when any root fails.

## Risks / Trade-offs

- Prefix-neutral parsing adds a maintained compatibility surface.
- Exact source mapping depends on source records produced by the realization change.
- Sequential dependency-layer compilation is simpler but may be slower than parallel compilation.
- Target Mantle paths intentionally differ from original Guix and Nix paths.
