# Design: Forge-agnostic VCS project inputs

## Architecture

Each VCS source kind has pure lock/manifest types and a shell fetcher adapter.

- Pure core: manifest validation, selector normalization, lock metadata validation, mirror compatibility planning, freshness observation comparison, source identity classification, generated-input rendering data, and unsupported/non-claim diagnostics.
- Imperative shell: invoking or embedding VCS tooling, resolving selectors, materializing checkouts, reading context/state files, and verifying fetched content digests.

Fetchers should plug into Mantle's source/fetch BuildService boundary so VCS support is scheduled and reported like other source actions.

## Source kinds

Darcs inputs should support repository URL, mirrors, tag or context selector, weak-hash freshness facts when available, and context-file lock metadata. Pijul inputs should support remote URL, mirrors, channel, state, and change selectors. Fossil inputs should support repository URL, branch/tag/check-in selectors, and lock metadata that identifies the selected checkout.

All VCS kinds must lock a content digest for the materialized source tree in addition to VCS-native identity. Mirrors are acceptable only when the fetched material verifies against the same locked identity and content digest.

## Forge neutrality

Manifest schema should use explicit repository/remote URLs and VCS selectors. Mantle should not add host-specific shorthand semantics for GitHub, GitLab, Codeberg, Darcs hubs, Pijul nests, or Fossil hosts. Any host-specific scraping belongs in a freshness probe, not in the source identity model.

## Validation strategy

Pure tests should cover selector defaults, lock metadata validation, mirror identity matching, generated input rendering, and unsupported selector combinations. Negative tests should cover missing VCS identity facts, mirror mismatch, unsupported subfeatures, ambiguous selectors, and stale lock entries.

Shell/fixture tests should use local repositories where practical. Remote or tool-dependent tests must be bounded and skippable only with explicit unsupported diagnostics; skipped tooling must not be reported as feature support.
