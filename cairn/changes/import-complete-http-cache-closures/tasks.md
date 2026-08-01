# Tasks

## Phase 1: Boundary and baseline

- [x] [serial] I1 Record ADR 0055 for strict metadata-first HTTP closure pull and the separate foreign-substitution identity lane. r[cache_substitution.complete_http_closure_pull]
  - Evidence: `adr/0055-plan-http-cache-closures-before-root-admission.md` and `adr/README.md`.
- [x] [serial] V1 Before implementation, run the focused baseline commands and record the result in `evidence/baseline-tests.md`. r[cache_substitution.complete_http_closure_pull]
  - `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -q -p crunch-store --lib closure`
  - `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p crunch-store --lib pull`

## Phase 2: Pure closure plan

- [ ] [serial] I2 Add a pure observation-driven closure planner with typed member, depth, metadata, aggregate NAR-size, duplicate, conflict, and finalization errors. r[cache_substitution.complete_http_closure_pull]
- [ ] [serial] I3 Compute a canonical BLAKE3 plan identity over authority, trust policy, store prefix, root, limits, and member facts. r[cache_substitution.complete_http_closure_pull]
- [ ] [parallel] I4 Add positive tests for one member, linear, diamond, cycle, shortest-depth, stable order, and stable identity. r[cache_substitution.complete_http_closure_pull]
- [ ] [parallel] I5 Add negative tests for every limit, duplicate references, conflicting path identity, unexpected observations, and incomplete finalization. r[cache_substitution.complete_http_closure_pull]

## Phase 3: HTTP discovery and strict admission

- [ ] [serial] I6 Add bounded signed narinfo discovery that completes the plan before any NAR request. r[cache_substitution.complete_http_closure_pull]
- [ ] [serial] I7 Reuse complete local members, refetch incomplete members, verify planned metadata during admission, and import the selected root last. r[cache_substitution.complete_http_closure_pull]
- [ ] [parallel] I8 Add positive HTTP tests for dependency-first import, diamond deduplication, local complete reuse, and stable plan reporting. r[cache_substitution.complete_http_closure_pull]
- [ ] [parallel] I9 Add negative HTTP tests for missing members, invalid signatures, path mismatch, malformed references, metadata overflow, NAR-size overflow, content failure, and zero root persistence after dependency failure. r[cache_substitution.complete_http_closure_pull]

## Phase 4: CLI and operator boundary

- [ ] [serial] I10 Add explicit `mantle store pull --closure <root>` dispatch for one HTTP root and preserve non-recursive explicit pull. r[cache_substitution.complete_http_closure_pull]
- [ ] [parallel] I11 Add positive and negative CLI tests for one HTTP root, zero roots, multiple roots, directory mode, `--all`, and ordinary explicit pull. r[cache_substitution.complete_http_closure_pull]
- [ ] [serial] I12 Update operator trust documentation with closure state and non-claims. r[cache_substitution.complete_http_closure_pull]
- [ ] [parallel] I13 Add implementation and verification requirement references. r[cache_substitution.complete_http_closure_pull]

## Phase 5: Validation and evidence

- [ ] [serial] V2 Rerun the focused baseline commands plus the new closure tests and record exact results in `evidence/focused-tests.md`. r[cache_substitution.complete_http_closure_pull]
- [ ] [serial] V3 Run focused Cargo check, formatting, and first-party Clippy, then record exact results in `evidence/package-checks.md`. r[cache_substitution.complete_http_closure_pull]
- [ ] [serial] H1 Import a recorded `cache.nixos.org` root closure into fresh store and state directories, verify complete local PathInfo/castore state, and record exact bounded evidence. If network access blocks the run, record the exact blocker and do not claim live closure proof. r[cache_substitution.complete_http_closure_pull]
- [ ] [serial] V4 Run Cairn validate, proposal, design, tasks, and Tracey coverage gates. Record exact results in `evidence/lifecycle-gates.md`. r[cache_substitution.complete_http_closure_pull]
- [ ] [serial] R1 Review the final diff and evidence against the requirement. Preserve package correctness, rebuild compatibility, evaluator parity, private-cache authentication, and release eligibility as non-claims. r[cache_substitution.complete_http_closure_pull]
