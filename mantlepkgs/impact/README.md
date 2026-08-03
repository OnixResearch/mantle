# Mantlepkgs impact reports

The `mantle-package-impact-v1` report compares two exact local snapshots.

## Comparable snapshot facts

The base and head must use the same values for these facts:

- system;
- store prefix;
- conversion-policy identity;
- catalog schema;
- package-record schema;
- observation schema;
- identity domain;
- comparison-policy identity.

The catalog identities can differ. The report binds both identities.

## Package decisions

Each public package key gets one primary disposition. The report also lists every changed dimension.

An observation is separate from a catalog change. Missing, blocked, not-attempted, and unavailable states do not become failures.

An admitted action-result success binds the current request, policy, platform, signature, output set, CAS fact, action, and selected result.

A realization receipt can supply an observed success or failure. An explicit status can supply only a non-build state.

## Closure decisions

A numeric closure delta requires complete matching semantics. Each member must include its identity, PathInfo identity, logical bytes, and resolved references.

A non-comparable delta contains ordered reason codes. Its member lists, count delta, and byte delta are `null`.

Retained dependencies are member identities present in both closures, excluding both declared root identities.

## External adapter boundary

`external-adapter.ncl` is a pure example. It reads one report and produces a local summary.

The adapter has no token, webhook, approval, comment, or network behavior. A separate service owns those effects.

## Non-claims

A favorable report does not prove correctness, unchanged behavior, reproducibility, deployment safety, or release eligibility.
