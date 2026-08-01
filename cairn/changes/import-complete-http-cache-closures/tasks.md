# Tasks

## Phase 1: Boundary and baseline

- [x] [serial] I1 Record ADR 0055 for strict metadata-first HTTP closure pull and the separate foreign-substitution identity lane. r[cache_substitution.complete_http_closure_pull]
  - Evidence: `adr/0055-plan-http-cache-closures-before-root-admission.md` and `adr/README.md`.
- [x] [serial] V1 Before implementation, run the focused baseline commands and record the result in `evidence/baseline-tests.md`. r[cache_substitution.complete_http_closure_pull]
  - `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -q -p crunch-store --lib closure`
  - `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p crunch-store --lib pull`

## Phase 2: Pure closure plan

- [x] [serial] I2 Add a pure observation-driven closure planner with typed member, depth, metadata, aggregate NAR-size, duplicate, conflict, and finalization errors. r[cache_substitution.complete_http_closure_pull]
  - Evidence: `crates/crunch-store/src/http_closure.rs`.
- [x] [serial] I3 Compute a canonical BLAKE3 plan identity over authority, trust policy, store prefix, root, limits, and member facts. r[cache_substitution.complete_http_closure_pull]
  - Evidence: `HttpClosurePlanBuilder::finalize` and `compute_plan_blake3`.
- [x] [parallel] I4 Add positive tests for one member, linear, diamond, cycle, shortest-depth, stable order, and stable identity. r[cache_substitution.complete_http_closure_pull]
  - Evidence: positive tests in `crates/crunch-store/src/http_closure.rs`.
- [x] [parallel] I5 Add negative tests for every limit, duplicate references, conflicting path identity, unexpected observations, and incomplete finalization. r[cache_substitution.complete_http_closure_pull]
  - Evidence: negative tests in `crates/crunch-store/src/http_closure.rs` and bounded narinfo tests in `pull.rs`.

## Phase 3: HTTP discovery and strict admission

- [x] [serial] I6 Add bounded signed narinfo discovery that completes the plan before any NAR request. r[cache_substitution.complete_http_closure_pull]
  - Evidence: `discover_http_cache_closure` and `fetch_http_narinfo_text_bounded`.
- [x] [serial] I7 Reuse complete local members, refetch incomplete members, verify planned metadata during admission, and import the selected root last. r[cache_substitution.complete_http_closure_pull]
  - Evidence: `import_discovered_http_closure`, `local_member_is_complete`, and `import_planned_http_member`.
- [x] [parallel] I8 Add positive HTTP tests for dependency-first import, diamond deduplication, local complete reuse, and stable plan reporting. r[cache_substitution.complete_http_closure_pull]
  - Evidence: positive `http_closure_*` tests in `crates/crunch-store/src/pull.rs`.
- [x] [parallel] I9 Add negative HTTP tests for missing members, invalid signatures, path mismatch, malformed references, metadata overflow, NAR-size overflow, content failure, and zero root persistence after dependency failure. r[cache_substitution.complete_http_closure_pull]
  - Evidence: negative `http_closure_*` tests in `crates/crunch-store/src/pull.rs`.

## Phase 4: CLI and operator boundary

- [x] [serial] I10 Add explicit `mantle store pull --closure <root>` dispatch for one HTTP root and preserve non-recursive explicit pull. r[cache_substitution.complete_http_closure_pull]
  - Evidence: `src/main.rs` and `src/store_cmd.rs`.
- [x] [parallel] I11 Add positive and negative CLI tests for one HTTP root, zero roots, multiple roots, directory mode, `--all`, and ordinary explicit pull. r[cache_substitution.complete_http_closure_pull]
  - Evidence: focused `store_pull_*` tests in `tests/integration.rs`.
- [x] [serial] I12 Update operator trust documentation with closure state and non-claims. r[cache_substitution.complete_http_closure_pull]
  - Evidence: `docs/foreign-derivation-import-trust-model.md`.
- [x] [parallel] I13 Add implementation and verification requirement references. r[cache_substitution.complete_http_closure_pull]
  - Evidence: `r[impl ...]` and `r[verify ...]` markers in `crates/crunch-store/src/http_closure.rs` and `pull.rs`.

## Phase 5: Validation and evidence

- [x] [serial] V2 Rerun the focused baseline commands plus the new closure tests and record exact results in `evidence/focused-tests.md`. r[cache_substitution.complete_http_closure_pull]
  - Evidence: `evidence/focused-tests.md`; 13 core, 11 closure shell, 53 pull, and 10 CLI tests passed.
- [x] [serial] V3 Run focused Cargo check, formatting, and first-party Clippy, then record exact results in `evidence/package-checks.md`. r[cache_substitution.complete_http_closure_pull]
  - Evidence: `evidence/package-checks.md`; task `7152` completed successfully.
- [x] [serial] H1 Import a recorded `cache.nixos.org` root closure into fresh store and state directories, verify complete local PathInfo/castore state, and record exact bounded evidence. If network access blocks the run, record the exact blocker and do not claim live closure proof. r[cache_substitution.complete_http_closure_pull]
  - Evidence: `evidence/live-cache-nixos-hello/summary.md`; five members imported, five reused on repeat, isolated `Hello, world!` execution passed.
- [ ] [serial] V4 Run Cairn validate, proposal, design, tasks, and Tracey coverage gates. Record exact results in `evidence/lifecycle-gates.md`. r[cache_substitution.complete_http_closure_pull]
  - Blocked: the pre-existing generated policy lacks `nominal_identity_policy`; see `evidence/lifecycle-gates.md`.
- [x] [serial] R1 Review the final diff and evidence against the requirement. Preserve package correctness, rebuild compatibility, evaluator parity, private-cache authentication, and release eligibility as non-claims. r[cache_substitution.complete_http_closure_pull]
  - Evidence: `evidence/oracle-review.md`; final review retained all named non-claims and resolved the advisory aggregate-size question.
