# Design: Resolve Mantlepkgs versions across Nixpkgs revisions

## Context

`nixpkgs-multiverse` maps a package attribute and reported version to a recent Nixpkgs revision. It materializes only revisions selected by evaluation.

Mantlepkgs already accepts exact source locks and concrete package selectors. Its consumer path runs without Nix after explicit catalog production.

The adaptation belongs before production. It must preserve exact source evidence and keep frontend package policy outside Mantle build semantics.

## Decisions

### Decision: own revision resolution in Mantlepkgs producer planning

**Choice:** Mantlepkgs will accept typed package-version requests before existing generation manifests are produced.

Each request binds a target system, Nix attribute path, reported version, public selector, selection policy, and limits.

The initial policy is `newest-published-revision-for-reported-version`. It selects the newest sampled channel revision that reported the requested version.

**Rationale:** Mantlepkgs owns exact Nixpkgs source selection for its producer. OnixOS can remain a policy client of published Mantle artifacts.

### Decision: generate per-system indexes from declared revision cohorts

**Choice:** A bounded shell will observe an ordered cohort of published channel revisions for one target system and declared attribute allowlist.

The shell records each revision, Nix source `narHash`, package attribute, version observation method, reported version, status, and diagnostic.

The initial version observation method requires the package `version` attribute. Package-name parsing is not an accepted fallback.

A pure core compacts successful observations into one deterministic index. The index retains the newest accepted revision for each attribute and reported version.

The generation receipt binds the complete observation-set BLAKE3 identity, cohort bounds, index identity, generator identity, and explicit unavailable results.

**Rationale:** Package availability and version facts can differ by system. A compact index still needs evidence for its selection process.

### Decision: keep external indexes as comparison evidence

**Choice:** A pinned `nixpkgs-multiverse` index may enter fixtures as an external observation.

It cannot make a Mantle resolution ready by itself. Native version recheck and exact source admission remain required.

The reference audit records its repository revision, observed license, generator shape, target system, and artifact digest.

**Rationale:** The upstream design is useful, but its flake API and generation authority do not transfer to Mantle.

### Decision: emit a separate resolution receipt

**Choice:** The resolver emits `mantlepkgs-version-resolution-v1` instead of changing Mantlepkgs v1 source-lock meaning.

The receipt binds these facts:

- request and policy identities;
- target system, attribute, reported version, and public selector;
- revision-index and observation-set BLAKE3 identities;
- selected exact Nixpkgs revision;
- Nix SHA-256 `narHash` with an explicit algorithm tag;
- observed package version and observation method;
- status, ordered blockers, and non-claims.

Mantle-owned receipt identity uses BLAKE3. SHA-256 remains only where the Nix source contract requires it.

**Rationale:** `lock_digest_blake3` and Nix `narHash` have different meanings and algorithms.

### Decision: recheck selected revisions before generation

**Choice:** The producer shell materializes the exact selected source and checks the requested attribute for the target system.

It compares the observed version with the request and receipt. It also computes the source-tree BLAKE3 required by the existing Mantlepkgs source lock.

A mismatch, missing attribute, changed `narHash`, evaluation failure, or unavailable source blocks generation.

**Rationale:** A compact index is discovery evidence, not final producer admission.

### Decision: group requests by selected revision

**Choice:** The pure planner groups accepted requests by exact selected revision.

Each revision group becomes one existing Mantlepkgs generation manifest. Requests in one group share source evaluation and graph production.

Each resulting generation becomes one domain shard. Existing domain composition then creates the combined public catalog.

Public selectors include the requested version. An unversioned alias exists only when policy declares one unique default.

**Rationale:** Existing manifests bind one source lock. Domain composition already combines package sets from several exact sources.

### Decision: preserve the functional core and imperative shell

**Choice:** The core validates contracts, compacts observations, resolves requests, groups revisions, orders diagnostics, and computes BLAKE3 identities.

The shell reads files, runs Nix, observes channels, materializes sources, evaluates attributes, writes artifacts, and invokes existing Mantlepkgs production.

**Rationale:** Resolution policy must replay from saved observations without files, processes, networks, clocks, or ambient state.

## Rollout

1. Add contracts, saved observations, and pure resolver tests.
2. Add one `x86_64-linux` cohort with a small attribute allowlist.
3. Add bounded index production and selected-revision recheck.
4. Emit existing Mantlepkgs manifests grouped by revision.
5. Compose two reported versions of one leaf package into one domain catalog.
6. Add other systems only after system-specific negative coverage passes.
7. Expose accepted resolution receipts to OnixOS after the schema is stable.

## Risks and trade-offs

- Historical Nixpkgs revisions can fail under the current Nix implementation.
- A package `version` field can be absent, malformed, or semantically weak.
- Channel publication does not prove that cache objects remain available.
- The newest revision for one reported version can contain unrelated source changes.
- Per-system observation can require substantial producer work.
- Versioned selectors increase public catalog size and conflict risk.

## Claim boundary

A successful receipt proves only the recorded resolution and recheck under exact inputs. It does not prove package correctness or compatibility.

It also does not prove binary-cache retention, evaluator parity, reproducibility, deployment safety, or release eligibility.
