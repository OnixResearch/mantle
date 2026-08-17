## Context

The adversarial hermeticity gauntlet establishes broad behavior. A maintained regression suite turns that contract into an edit-time and release-readiness rail with stable fixtures, expected outcomes, and durable evidence summaries. The suite should stay narrow enough to run regularly while still covering each major leak class.

## Decisions

### 1. Suite cases are declared data

**Choice:** The suite inventory lists each perturbation case, mode, expected verdict, required blocker or audit class, and whether unsupported hosts should produce a non-claim.

**Rationale:** Reviewers can see coverage without reverse-engineering test code.

### 2. Positive and negative cases are paired

**Choice:** Every clean success path has adjacent negative cases for env, PATH, host exec, network, closure, umask, temp-root, and nondeterminism boundaries.

**Rationale:** A suite with only successful builds does not prove fail-closed behavior.

### 3. Evidence is bounded to tested axes

**Choice:** The suite report may support release/global readiness only for the axes it ran and must list skipped or unsupported axes as non-claims.

**Rationale:** Hermeticity evidence should be strong but not overbroad.

## Risks / Trade-offs

- Some negative tests require Linux sandbox features and must report unsupported cases clearly on other hosts.
- The suite can become expensive if it grows into full self-hosting proof; keep heavyweight rails separate.
