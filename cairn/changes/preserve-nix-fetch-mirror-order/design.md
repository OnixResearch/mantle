# Design: preserve Nix fetch mirror order

## Context

Mantle's canonical foreign download path already accepts a bounded ordered candidate list. The compiler stores the primary address in `url`. It stores the full list in `__mantle_foreign_candidates`.

The fetch service validates that list and tries candidates in order. It continues only after transport, Git, or I/O failures. Fixed-output verification runs after acquisition and remains authoritative.

The Nix producer follows a different path. It copies concrete derivation environment fields and marks fixed-output nodes as `fixed-output-fetch`. It does not create canonical candidate data.

Nix has two relevant concrete forms:

- unstructured derivations can expose `url` or a space-separated `urls` value;
- structured derivations store typed values in the `__json` environment field, including a `urls` string array.

The consumer must not learn these Nix encodings. The producer adapter owns their interpretation.

## Evidence Basis

The plan follows these checked source facts:

- `src/foreign_derivation_import.rs` copies Nix environment data but creates source payloads with empty mirror lists.
- `src/foreign_graph_compiler.rs` creates ordered candidates only for explicit download and Git-download nodes.
- `crates/crunch-build/src/fetch_build_service.rs` reads `url` and the private Mantle candidate binding. It does not read Nix `urls` or `__json`.
- Nix structured derivations encode typed attributes as JSON in `__json`.
- The accepted foreign-import spec already requires deterministic mirror order and fixed-output verification.
- ADR 0046 already requires bounded deterministic source fallback through the receipt-bound foreign adapter.

## Decisions

### Decision 1: Use one canonical candidate list

**Choice:** Add an optional ordered candidate list to the foreign fixed-output fetch node model.

The list contains the primary address first. Later entries are fallbacks. Order is significant and must not be sorted.

The list has a named maximum. The initial limit remains 16 candidates to match the compiler and fetch service.

**Rationale:** Fetch addresses are acquisition policy for one node. They must not be spread across unrelated source payloads or inferred again during execution.

### Decision 2: Normalize Nix forms in a pure core

**Choice:** Add a pure normalization core over owned input data.

The core accepts these facts:

- optional direct `url`;
- optional unstructured `urls` text;
- optional bounded structured JSON text;
- the selected fetch kind and supported URL policy;
- named candidate and input-size limits.

The core returns one ordered list or one deterministic diagnostic. It performs no file, store, environment, process, network, or clock access.

Structured `__json.urls` is a JSON array of strings. An optional structured `url` must match its first item. Unstructured `urls` uses Nix's concrete whitespace-separated string form. A direct unstructured `url` becomes a one-item list when no list exists.

When `__json` exists, its candidate fields are authoritative. Candidate-like top-level environment fields alongside `__json` are conflicting input and fail closed. For unstructured input, `url` must equal the first `urls` item when both fields exist.

**Rationale:** The adapter understands Nix formats. The compiler and fetch service should understand only Mantle's canonical model.

### Decision 3: Keep Nix-specific parsing out of the fetch service

**Choice:** The fetch service will not parse `urls`, `__json`, nixpkgs mirror tables, or ambient Nix mirror variables.

The compiler creates `url` and `__mantle_foreign_candidates` from canonical node data. Foreign input cannot set that private field.

**Rationale:** Runtime parsing would mix frontend compatibility with network execution. It would also make derivation identity depend on hidden interpretation.

### Decision 4: Preserve graph and derivation identity

**Choice:** The canonical candidate list is serialized in graph artifacts and bound into target derivation and executable-plan identity.

Changing candidate order changes the target derivation and plan identities. It does not change the required fixed-output content digest.

Existing graphs without the new field retain the current legacy path. If a graph contains both canonical and legacy candidate data, both forms must agree.

**Rationale:** Two different fallback policies must not share one recipe identity. Existing single-address artifacts still need a bounded migration path.

### Decision 5: Keep fallback sequential and fail closed on content mismatch

**Choice:** Mantle continues to try candidates in declared order. It does not race them.

Transport, Git, and I/O failures can advance to the next candidate. A fixed-output mismatch stops the attempt and prevents PathInfo admission.

Attempt evidence records each unavailable address and the selected address. It does not record credentials or response bodies.

**Rationale:** This matches ADR 0046. Availability policy must not weaken content verification.

### Decision 6: Treat mirror aliases as producer work

**Choice:** A concrete direct URL can enter the canonical list when its scheme is supported by the selected Mantle fetch kind.

A Nix `mirror://` address or ambient mirror override cannot reach the consumer unchanged. The producer must expand it into explicit addresses with recorded provenance, or mark the node unsupported.

The first implementation can reject unresolved mirror aliases. It must not silently drop them or consult ambient nixpkgs state during consumption.

**Rationale:** Alias expansion depends on Nix or nixpkgs policy. That policy belongs before canonical artifact publication.

### Decision 7: Do not broaden fixed-output classification

**Choice:** Candidate normalization applies only after a node is classified as a supported fixed-output fetch.

Candidate fields alone do not turn an arbitrary fixed-output builder into a simple download. Builder-specific transforms, including unsupported post-fetch behavior, remain visible blockers.

**Rationale:** A content hash does not describe builder semantics.

## Data Flow

```text
concrete Nix ATerm or derivation JSON
  -> bounded Nix candidate extraction
  -> pure candidate normalization
  -> canonical foreign fetch node
  -> graph validation and receipt binding
  -> foreign graph compiler
  -> url + private ordered-candidate binding
  -> ordinary Mantle fetch service
  -> sequential acquisition
  -> shared fixed-output verification
  -> PathInfo admission or fail-closed result
```

## Failure Semantics

- Empty, invalid, duplicate, or oversized lists fail before artifact publication.
- Malformed or oversized structured JSON fails before graph publication.
- Non-string structured list items fail with a stable diagnostic.
- Conflicting `url`, `urls`, and canonical list values fail closed.
- Foreign use of `__mantle_foreign_candidates` fails before compilation.
- Unsupported URL schemes and unresolved mirror aliases remain explicit blockers.
- Exhausted candidates retain bounded attempt evidence and fail the fetch.
- A content mismatch does not try another candidate and does not enter PathInfo.
- Failure in one selected root remains visible under existing sibling-root reporting.

## Tests

Positive tests cover:

- one direct `url`;
- unstructured ordered `urls`;
- structured `__json.urls`;
- matching `url` and `urls`;
- ATerm and derivation-JSON parity;
- exact order in graph, compiled derivation, plan, and attempt log;
- first candidate unavailable and second candidate selected;
- unchanged fixed-output content identity across candidate reordering.

Negative tests cover:

- empty lists;
- malformed and oversized JSON;
- non-string array entries;
- duplicate and excess candidates;
- conflicting primary fields;
- reserved-field injection;
- unsupported URL schemes;
- unresolved `mirror://` aliases;
- fixed-output mismatch without fallback or PathInfo admission;
- arbitrary fixed-output builders that only resemble downloads.

## Alternatives Considered

### Teach the fetch service to parse Nix fields

Rejected. This couples network execution to a foreign frontend format and creates hidden identity rules.

### Keep only the first address

Rejected. This violates the accepted ordered-mirror requirement and loses declared availability policy.

### Store mirrors on unrelated source payloads

Rejected. A Nix fetch output is not one of its input source payloads. This would create false source relationships.

### Race all candidates

Rejected. Response timing is nondeterministic and would weaken attempt evidence.

### Continue after a content mismatch

Rejected. The existing foreign realization decision treats a mismatch as an integrity failure, not ordinary unavailability.

## Risks / Trade-offs

- Concrete Nix structured attributes add a bounded JSON parsing surface.
- Some nixpkgs fetchers rely on mirror aliases, hashed mirrors, or post-fetch transforms and will remain blocked initially.
- Adding canonical node data changes graph and plan digests when ordered candidates are present.
- Legacy graph support creates a temporary dual-representation validation path.
- Supporting more Nix fetcher shapes later will require separate, explicit semantics.

## Non-Claims

This change does not prove arbitrary Nix fetcher parity, nixpkgs evaluator parity, source correctness, mirror trust, package correctness, reproducibility, or release eligibility.
