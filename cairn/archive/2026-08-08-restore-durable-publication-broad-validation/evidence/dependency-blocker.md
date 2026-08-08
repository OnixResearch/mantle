# Dependency blocker

Date: 2026-08-08

Status: resolved by canonical promotion `aa577374516e6b15c3c4ef59c71c74410c0d0fab` and archive commit `bd614a7fd6fa90a49e278cf5b683edc17fe518f6`.

`restore-durable-publication-broad-validation` depends on `promote-durable-publication-adoption`. The observations below record why the original promotion contract was not satisfiable.

## Mantle ancestry

Fresh remote observations produced these identities:

```text
origin/main: df79e1b7b546b1d67de0751a60862bf609efc51c
promotion candidate: d1f3d6d96e2b0d9cd8497cd89b8e5a93d4a7dfaf
merge base: 7875ec1c8b80662f183b76194ae4ef8e3cd52a28
main-only commits: 314
candidate-only commits: 1
origin/main is an ancestor of the candidate: false
accepted candidate is an ancestor of origin/main: false
```

The promotion specification requires the fetched canonical target to be an ancestor of the exact candidate. The design requires ancestry drift to stop promotion. A force push, silent merge, or rewritten candidate is forbidden.

## Onix Core ordering

Fresh remote observations produced these identities:

```text
Onix Core origin/main: bf7f37d34bb87b402bda275f09b36b097742805e
required accepted commit: b8387cd7d59fa3b0d4ea67646352dd27c4f7d7ed
required commit is an ancestor of origin/main: false
```

The promotion specification requires the accepted Onix Core commit to be on canonical `main` before Mantle promotion.

## Decision required

The broad-validation tasks remain unchecked. No specification sync or archive mutation ran.

An owner must revise or replace the stale promotion contract. The decision must define an authorized integration method for the accepted Mantle implementation and identify the canonical Onix Core ordering milestone. Broad validation can continue only after that prerequisite is complete.

This evidence does not claim broad validation success or release readiness.

## Resolution

The user authorized a reviewed merge candidate. Mantle canonical `main` now contains both the prior canonical history and accepted adoption `d1f3d6d96e2b0d9cd8497cd89b8e5a93d4a7dfaf` through merge `aa577374516e6b15c3c4ef59c71c74410c0d0fab`. Promotion was synchronized and archived by `bd614a7fd6fa90a49e278cf5b683edc17fe518f6`.

Onix Core canonical `main` is `bc4629c9e766d3db82e4dab9fe8c166c360b8435` and contains accepted admission `b8387cd7d59fa3b0d4ea67646352dd27c4f7d7ed`. The broad-validation prerequisite is satisfied.
