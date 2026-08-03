# Tasks: add explainable store retention

## Phase 1: Baseline and policy

- [ ] [serial] I1 Run current root, project retention, GC, CA mapping, action-result, and store CLI tests before core changes. Record current permanent-build-root and unexplained-retention behavior. r[store_lifecycle.retention_policy]
- [ ] [serial] I2 Add a typed Nickel retention policy with named root classes, generation limits, lease rules, owner scopes, transition rules, and collection bounds. r[store_lifecycle.retention_policy]
- [ ] [parallel] I3 Add positive policy fixtures and negative unknown-class, missing-owner, invalid-generation, invalid-lease, duplicate-rule, and limit fixtures. r[store_lifecycle.retention_policy]

## Phase 2: Pure retention and usage cores

- [ ] [serial] I4 Add versioned root, owner, generation, lease, policy, storage-observation, and retention-decision types. r[store_lifecycle.root_provenance]
- [ ] [serial] I5 Implement pure keep, expire, migrate, quarantine, and remove planning with stable reason codes. r[store_lifecycle.retention_policy]
- [ ] [serial] I6 Implement pure total, inclusive, unique, shared, unknown, retained, and reclaimable usage aggregation with checked arithmetic. r[store_lifecycle.usage_report]
- [ ] [parallel] I7 Add property tests for deterministic ordering, duplicate rejection, monotonic live closure, checked totals, bounded decisions, and equivalent-fact replay. r[store_lifecycle.retention_validation]

## Phase 3: Root registry and project integration

- [ ] [depends:split-store-authority-capabilities] I8 Add versioned root records behind the accepted `RootRegistry` and `StoreAdmin` capabilities. Migrate path-only records to protected `legacy-unmanaged` records without invented ownership. r[store_lifecycle.root_provenance]
- [ ] [serial] I9 Register project output generations from successful selected-root builds and replace superseded generations only through policy. r[store_lifecycle.root_provenance]
- [ ] [serial] I10 Add active development-shell leases and bounded renewal without changing package action identity. r[store_lifecycle.retention_policy]
- [ ] [parallel] I11 Add interrupted migration, corrupt registry, clock rollback, expired lease, branch generation, duplicate checkout, and missing project fact fixtures. r[store_lifecycle.retention_validation]

## Phase 4: Operator commands and safe GC

- [ ] [serial] I12 Add deterministic human and JSON `mantle store usage` reports with bounded class and owner summaries. r[store_lifecycle.usage_report]
- [ ] [serial] I13 Add explained root listing and explained GC plans with stable keep and removal reasons. r[store_lifecycle.gc_explanation]
- [ ] [serial] I14 Make ordinary GC non-mutating and require explicit execution against an unchanged plan identity under the store mutation lock. r[store_lifecycle.safe_gc_execution]
- [ ] [parallel] I15 Add stale-plan, root-change, PathInfo-change, closure-gap, shared-path, symlink, interrupted rewrite, and deletion-failure fixtures. r[store_lifecycle.safe_gc_execution]

## Phase 5: Documentation and validation

- [ ] [serial] I16 Document root classes, project generations, leases, usage fields, explained GC, legacy migration, rollback, and non-claims. r[store_lifecycle.gc_explanation]
- [ ] [serial] V1 Run `nix develop -c cargo test -p crunch-store roots::`, `nix develop -c cargo test -p crunch-store gc::`, focused project-retention tests, and focused store CLI tests. r[store_lifecycle.retention_validation]
- [ ] [serial] V2 Run focused positive and negative plan-execution process fixtures and record exact plan identities, mutation results, and retained paths. r[store_lifecycle.safe_gc_execution]
- [ ] [serial] V3 Run focused formatting and Clippy with warnings denied, Nickel contract checks, machine-contract checks, and `git diff --check`. r[store_lifecycle.retention_validation]
- [ ] [serial] V4 Run `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .` plus proposal, design, and tasks gates for this change. Record exact outputs before archive. r[store_lifecycle.retention_validation]
