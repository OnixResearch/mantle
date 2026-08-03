# Design: Mantlepkgs package-impact reports

## Context

EkaCI compares base and head derivation graphs, classifies changed build results, and reports closure differences. Mantle already owns stronger catalog, action-result, store, and evidence boundaries.

Mantle can adapt the review model without adopting EkaCI service orchestration or GitHub authority.

## Decisions

### Decision: compare contracted snapshots

**Choice:** The core receives normalized base and head snapshots. Each snapshot binds catalog identity, system, store prefix, conversion policy, package records, and observation identities.

Compatibility is a pure decision. Incompatible global facts produce an explicit report-level rejection or typed non-comparability.

**Rationale:** A diff is meaningful only when both sides share the facts required by that comparison.

### Decision: separate catalog change from observed build outcome

**Choice:** Catalog dispositions include added, removed, unchanged, recipe changed, policy changed, blocker changed, and variant changed.

Build transitions use explicit observation states. They include newly successful, still successful, newly failed, still failed, blocked, not attempted, and unavailable.

An absent build observation never becomes a failure or success.

**Rationale:** Source change and observed execution are different evidence classes.

### Decision: compare complete closures only

**Choice:** Closure comparison requires matching system, store prefix, closure semantics, and size semantics. Both closures must report complete member facts.

Comparable results include added and removed members, member-count delta, logical byte delta, and retained dependencies. Otherwise the report contains a stable non-comparability reason.

**Rationale:** Partial or mismatched closure data can create false regression claims.

### Decision: preserve action-result authority

**Choice:** The shell can query existing admitted action results. It must use current request, policy, platform, signature, output, and CAS checks.

The impact core receives only normalized admitted observations. It does not use a derivation path or CAS object as result authority.

**Rationale:** EkaCI-style build reuse must not weaken Mantle ADR 0024.

### Decision: keep CI service behavior external

**Choice:** Mantle writes `mantle-package-impact-v1` to stdout or a selected file. An external adapter can publish forge comments, statuses, metrics, or dashboards.

Mantle does not read forge tokens, process webhooks, approve changes, or manage pull requests.

**Rationale:** CI presentation and credentials are not build-system authority.

### Decision: use a functional core and imperative shell

**Choice:** The core validates compatibility, joins package keys, derives dispositions, computes transitions, compares closure facts, orders diagnostics, and computes a BLAKE3 report identity.

The shell reads artifacts, verifies their digests, queries admitted action results, writes reports, and returns process status.

**Rationale:** Impact decisions remain deterministic and testable without files, stores, networks, or clocks.

### Decision: bound report work

**Choice:** Typed policy limits packages, variants, closure members, dependency edges, observations, diagnostics, and report bytes.

The core rejects over-limit inputs. It does not truncate a report and call it complete.

**Rationale:** Large external catalogs must not create unbounded memory or output work.

## Rollout

1. Define snapshot and report contracts.
2. Add pure catalog and outcome fixtures.
3. Add complete-closure comparison.
4. Add the shell adapter over stored Mantle artifacts.
5. Add one external adapter fixture without forge credentials.

## Risks and trade-offs

- Report completeness depends on retained base artifacts.
- Store-prefix or policy changes can make closure facts non-comparable.
- A large package set can produce a large report within configured limits.
- A favorable impact report does not prove that unchanged packages remain correct.
