## Context

Current Mantle evidence supports scoped claims:

- deterministic-release proof for a specific release bundle;
- provider fixed-point proof bound to a release artifact;
- external independent witness agreement for the 2026-07-02 source-policy-fixed release;
- action-correct build receipts for narrower build surfaces.

None of those prove global reproducibility for all Mantle builds, all frontends, all source transports, all target systems, or future releases. Global reproducibility needs a separate admission gate over an explicit universe.

## Decisions

### 1. Treat global reproducibility as an admitted universe, not a slogan

**Choice:** A global claim is valid only for a digest-bound universe document that enumerates included build surfaces, target systems, source modes, toolchain routes, cache/substitution modes, and excluded surfaces.

**Rationale:** The word "global" is otherwise ambiguous. Enumerating the universe makes the claim reviewable and lets unsupported surfaces become explicit blockers or non-claims.

### 2. Use a policy-driven witness matrix

**Choice:** The required independent operator domains, host classes, perturbation axes, and quorum come from a named policy artifact rather than hard-coded counts in implementation.

**Rationale:** Different release tiers may require different witness strength. The report must bind the policy digest so reviewers know which matrix was applied.

### 3. Keep the functional core pure

**Choice:** Report evaluation should be a pure core over loaded receipts, policy, universe entries, and witness results. CLI code should only discover files, parse artifacts, render output, and set exit status.

**Rationale:** Claim admission is business logic. It needs deterministic unit tests without standing up stores, networks, or release workflows.

### 4. Fail closed on incomplete or weaker evidence

**Choice:** Missing receipts, mismatched digests, weak hermeticity, unsupported source/toolchain surfaces, reused stores where freshness is required, absent independent witness coverage, or stale policy must produce a blocked/non-global report.

**Rationale:** A global claim is only useful if every known gap remains visible and cannot be silently downgraded to a success.

### 5. Preserve existing scoped claims

**Choice:** Existing release-specific claims remain valid when their evidence passes, but they must not be automatically promoted into global reproducibility.

**Rationale:** The Aspen witness agreement is strong for its release artifact set. It does not prove all Mantle build surfaces or all future releases.

## Risks / Trade-offs

- The first implementation will likely produce a blocked report until enough surfaces have receipts and witnesses. That is acceptable; a visible blocker inventory is progress.
- A universe manifest can grow large. The core should keep deterministic ordering and bounded limits so reports remain reviewable.
- Policy flexibility can weaken claims if misused. The report must always publish the policy identity, quorum, and non-claims.
