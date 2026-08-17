## Context

Mantle currently has two Rust project lanes: sandboxed offline Cargo for practical project builds, and native `rust-plan` for explicit Cargo-free topology evidence. The representative rail documents both, but its fixture is too narrow to guide future work or explain why unsupported cases remain unsupported.

The rail should become a matrix of bounded surfaces. The matrix is not a claim that all of Cargo works; it is a maintained inventory of what the lanes cover, what they block, and which evidence class each result belongs to.

## Decisions

### 1. The matrix is the source of truth

**Choice:** Add a checked-in matrix that names surface ids, expected lane behavior, fixture ownership, positive checks, negative checks, and non-claims. Tests consume the matrix to prevent docs and fixtures from drifting.

**Rationale:** A matrix makes frontier changes reviewable and avoids implicit broad claims from one growing fixture.

### 2. Offline Cargo and native rust-plan stay separate

**Choice:** The rail records separate outcomes for `cargo-inside-mantle-sandbox` and `cargo-free-bounded-topology` / `blocked-unsupported-surface`. A surface can be supported by offline Cargo while blocked by native rust-plan.

**Rationale:** Operators need a practical build lane now, but native planner evidence must remain honest about unsupported surfaces.

### 3. Unsupported means fail-closed with a class

**Choice:** Each unsupported surface has a deterministic blocker class and a negative test. The native rail must not invoke Cargo when `--no-cargo-oracle` is selected, and the offline Cargo lane must not read ambient caches or network sources.

**Rationale:** The most important guarantee is absence of hidden fallback, not coverage of every Cargo feature.

### 4. Native-link surfaces are staged

**Choice:** `links`, `pkg-config`, and native C compilation are added as explicit surfaces with narrow fixtures. If a surface cannot be supported safely in the first implementation, it remains blocked with a stable class and next-action notes.

**Rationale:** Native linking is where many offline builds fail. It should be visible even before full support lands.

### 5. Docs cite evidence class, not compatibility slogans

**Choice:** Examples and README text name the rail, supported surfaces, current blockers, commands, and non-claims. They avoid wording like “Cargo-compatible” except as a bounded matrix label.

**Rationale:** This keeps proof-before-claim intact while making progress easier to communicate.

## Risks / Trade-offs

- A single huge fixture can become hard to debug; the matrix should allow multiple focused fixtures.
- Some surfaces are host-sensitive (`pkg-config`, native C); tests must use controlled fake tools or local fixtures rather than ambient host packages.
- Expanding the rail may make ordinary test suites slower; heavyweight surfaces should be explicit and skippable with durable ignored-test rails.
