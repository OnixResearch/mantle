# ADR 0033: Preserve history when publishing Mantle

## Status

Accepted (2026-07-21)

## Context

The pre-publication privacy audit found historical workstation paths, host labels, private repository references, and the maintainer email in committed lifecycle evidence and Git metadata. The current-tree inventory places the personal path references almost entirely under archived Cairn/OpenSpec evidence, bootstrap/package evidence, and `AGENTS.md`. Rewriting those values would change commit identities, break links into historical evidence, and make the repository appear cleaner than the evidence actually was.

Gitleaks 8.30.1 scanned 2,278 commits and initially reported twelve generic-key candidates. Each candidate was reviewed as public Git revision material, BLAKE3 identity material, or prose rather than a credential. Exact commit/file/rule/line fingerprints are retained in `.gitleaksignore`; no path-wide or rule-wide exclusion is allowed. This scan is evidence of no unreviewed Gitleaks finding, not proof that no secret exists.

## Decision Drivers

- Preserve the provenance and review value of committed lifecycle evidence.
- Avoid destructive history rewriting when no credential has been identified.
- Make the privacy tradeoff explicit rather than silently redacting historical facts.
- Keep live commands, tests, and documentation portable and free of maintainer-specific defaults.
- Keep future secret findings fail-closed and eligible for rotation or emergency history rewriting.

## Decision

Mantle will preserve its existing Git history when the repository is made public. Historical absolute paths, host labels, repository references, and the existing author/committer email are accepted as public metadata and will not be rewritten solely for privacy or cosmetic portability.

Live source, scripts, tests, and operator documentation must not add maintainer-specific home paths or workstation defaults. Local agent permission files remain ignored. Secret scanning must cover history, tracked changes, and untracked publication inputs. Reviewed false positives must use exact Gitleaks fingerprints; broad path, rule, or entropy exclusions are forbidden.

This decision does not authorize publishing credentials, bearer material, signing secrets, private keys, or sensitive infrastructure details. Any credible secret finding still requires immediate validation, rotation, removal from the current tree, and a separate decision on history rewriting. Publication must also review which branches remain reachable; preserving history does not require retaining abandoned branch refs.

## Alternatives Considered

### Publish a squashed sanitized snapshot

Rejected because it would discard the development and lifecycle provenance that Mantle intentionally carries as review evidence.

### Rewrite every commit and branch

Rejected because no credential was identified, while the rewrite would invalidate commit identities and historical links across a large evidence corpus.

### Publish without cleaning live defaults

Rejected because preserving historical facts does not justify keeping non-portable workstation paths in current commands, tests, or operator guidance.

## Consequences

- Public readers can observe historical usernames, workstation paths, host labels, repository references, and the maintainer email.
- Existing evidence and commit identities remain stable.
- New live maintainer-specific defaults are regressions even when similar values remain in historical evidence.
- `.gitleaksignore` must remain narrowly fingerprinted and be reviewed whenever history or findings change.
- Gitleaks success remains bounded scanner evidence, not a confidentiality or credential-absence proof.
