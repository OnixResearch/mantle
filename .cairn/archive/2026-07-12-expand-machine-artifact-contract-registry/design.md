## Context

Mantle emits machine JSON from many Rust modules. The current doctor-report pilot demonstrates the desired layering, but its inventory and checker encode one surface through report-specific constants and validation functions. Scaling that pattern by copying the checker would create more duplicated authority rather than a contract system.

The design makes inventory completeness and generation data-driven while preserving Rust as owner of runtime-emitted facts.

## Decisions

### 1. Inventory every public machine surface before contracting it

**Choice:** The Nickel inventory records every CLI- or package-exposed machine JSON family and classifies it as:

- `contracted`: stable consumer/review boundary with schema, generated Nickel contract, fixtures, and version policy;
- `internal`: Rust-owned implementation DTO not promised to consumers;
- `debug`: bounded diagnostic output not suitable for readiness or automation;
- `compatibility`: retained legacy projection with an explicit successor and support policy; or
- `external`: shape owned by an interoperability standard or upstream producer.

Every entry includes Rust owner, producer command/API, consumers, artifacts, validation commands, freshness strategy, fixture coverage, and non-claims.

**Rationale:** Not every serializable struct deserves a public contract, but every public machine output needs an explicit decision.

### 2. Rust DTOs own emitted facts; schemas own the review projection

**Choice:** Contracted Rust DTOs deterministically render or derive an exact JSON Schema snapshot. A repo-owned generator converts the supported schema subset to a Nickel contract. Serialization parity tests prove representative Rust values satisfy the schema/contract, and unknown or unsupported schema constructs fail generation.

If a schema cannot be derived directly, its deterministic renderer and Rust parity test are treated as one owner; hand-maintained untested duplicate schemas are not accepted.

**Rationale:** Runtime report production must not depend on Nickel, while consumers still need a language-neutral schema and a reviewable Nickel contract.

### 3. The registry drives generation and checks

**Choice:** Refactor `check-machine-schema-contracts.rs` into a thin filesystem shell around pure registry, schema-subset, contract-rendering, fixture-classification, and freshness cores. The checker iterates registry entries rather than naming report families in source constants.

Each contracted entry binds schema, generated contract, positive/negative fixture set, producer owner, and consumer policy with BLAKE3 identities.

**Rationale:** Adding a surface should be data plus focused pure tests, not another branch in a monolithic script.

### 4. Migrate a bounded high-value cohort first

**Choice:** The first cohort includes stable public projections for:

- `BuildJsonReport` and `BuildPlanReport`;
- `RoutePlanReport` and its stable policy basis;
- portable receipt bundle, verify, and import reports;
- source-bundle plan, verify, and offline-preflight reports;
- Nickel export report/receipt projections; and
- the stable release/attestation handoff envelope selected by the inventory.

Other surfaces are classified now and may be contracted in later packages. Active remote-attempt and Wasm-component changes register future surfaces but do not block this cohort.

**Rationale:** Build, cache/reuse, source, receipt, and release boundaries have the highest automation and cross-repository impact.

### 5. Common artifact invariants are reusable

**Choice:** Generated contracts use a shared Nickel prelude for exact schema literals, closed enums, bounded integers, lowercase BLAKE3 digests, safe logical references, bounded arrays/maps, redaction-safe strings, and deterministic diagnostic classes. Interoperability-required hashes remain in their mandated algorithm; Mantle-owned identities default to BLAKE3.

**Rationale:** A generated record shape without semantic scalar contracts is not enough for review-boundary safety.

### 6. Version changes fail closed and migrate explicitly

**Choice:** Consumers reject unknown schema versions by default. A compatibility entry may admit a prior version only through an explicit converter and fixtures. Contract, schema, fixture, or consumer drift fails freshness before packaging.

**Rationale:** Silent best-effort parsing turns stable machine output into an accidental API.

### 7. Nickel is not a runtime dependency

**Choice:** Build, substitute, import, attest, and release commands serialize Rust DTOs directly. Nickel contracts are evaluated only in explicit generation/check workflows and may be consumed by review tooling or downstream configuration.

**Rationale:** Machine-contract review must not add evaluator authority or latency to product operations.

## Risks / Trade-offs

- The public-output inventory may reveal undocumented consumers and require compatibility decisions before schemas can close.
- JSON Schema to Nickel conversion supports a bounded subset; unsupported constructs must fail with a clear issue rather than degrade to `Dyn`.
- Contracting too many internal DTOs would freeze implementation detail. Classification is mandatory before migration.
- Passing fixtures establish shape and declared linkage only; behavioral and release claims remain with their existing Rust/evidence rails.
