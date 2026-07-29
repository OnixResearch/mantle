# Tasks

## Phase 1: Bind source and target state

- [x] [serial] V1 Verify the recorded source commit, tree, parent, subject, and changed-file set. Record the result in integration evidence. r[build_correctness.dynamic_plan_nominal.integration.source] Evidence: `evidence/integration-validation.md` records the immutable source identity and all 16 changed files.
- [x] [serial] V2 Select a committed main target that preserves the unrelated StageX work. Record its commit and require a clean integration worktree. r[build_correctness.dynamic_plan_nominal.integration.target] Evidence: `evidence/integration-validation.md` records target `bb241ec422bc590693fc5eaab199730b3e4bf2fe` and its tree.
- [x] [serial] I1 Create a dedicated integration branch and worktree from the selected target commit. Do not use the current dirty main checkout. r[build_correctness.dynamic_plan_nominal.integration.target] Evidence: the same file records branch `integrate-nominal-dynamic-plan` and worktree `mantle-nominal-dynamic-plan-integration`.

## Phase 2: Apply and reconcile the source commit

- [x] [serial] I2 Apply commit `a2c872cfe6826df2d3ca7c85597365e6076fdfba` to the integration branch and record every conflict. r[build_correctness.dynamic_plan_nominal.integration.source] Evidence: resolved commit `a6bb77773f1458d5919a869ac2efc7edd8efaf70` records a zero-conflict cherry-pick.
- [x] [serial] I3 Reconcile accepted specs, archived evidence, README content, documentation, Cargo metadata, and Dylint policy as a semantic union. r[build_correctness.dynamic_plan_nominal.integration.preservation] Evidence: `evidence/integration-validation.md` records each decision and the exact changed-file set.
- [x] [serial] I4 Reconcile dynamic-plan and worker code while retaining checked constructors, private fields, explicit wire admission, typed graph storage, and digest roles. r[build_correctness.dynamic_plan_nominal.integration.preservation] Evidence: focused tests, compile-fail tests, structural searches, and Octet results are recorded in the evidence files.
- [x] [parallel] V3 Audit the resolved diff against both parents. Reject deleted target work, missing source files, raw-string graph fallbacks, and weakened constructors. r[build_correctness.dynamic_plan_nominal.integration.preservation] Evidence: the source and resolved commits have the same 16 changed paths. The StageX paths are byte-identical to the target.

## Phase 3: Revalidate the resolved integration tree

- [x] [parallel] V4 Run focused `crunch-build` positive, negative, graph, placeholder, producer, worker, and compatibility tests. r[build_correctness.dynamic_plan_nominal.integration.validation] Evidence: `cargo test -p crunch-build --lib` passed 652 tests, and `export_api` passed one test.
- [x] [parallel] V5 Run all compile-fail category tests and malformed wire admission tests. r[build_correctness.dynamic_plan_nominal.integration.validation.negative] Evidence: `cargo test -p crunch-build --doc` passed the positive doctest and all four compile-fail doctests. The 652 library tests include malformed wire rejection.
- [x] [parallel] V6 Compare the frozen canonical JSON and require plan digest `dc6814c1f500dc7e8575c3fd84a64ae78a70d38313ccbbfff4fbfcf7610f6750`. r[build_correctness.dynamic_plan_nominal.integration.compatibility] Evidence: the frozen-wire test passed with the required BLAKE3 value.
- [x] [parallel] V7 Run Octet and require zero `primitive_domain_alias`, `raw_domain_value`, and `newtype_invariant_bypass` findings in the declared scope. r[build_correctness.dynamic_plan_nominal.integration.policy] Evidence: `evidence/octet-nominal-domain-2026-07-28.json` records zero findings for all three denied lints.
- [x] [parallel] V8 Run `./scripts/check-first-party-clippy.sh`, `./scripts/check-first-party-tigerstyle.sh -p crunch-build -- --lib`, `nix build .#checks.x86_64-linux.fmt .#checks.x86_64-linux.clippy -L --option secret-key-files ''`, and broader workspace tests. Triage every failure against the selected target baseline. r[build_correctness.dynamic_plan_nominal.integration.validation] Evidence: `evidence/integration-validation.md` records successful quality checks, 1,760 serial binary tests, and exact target-baseline failure comparisons.

## Phase 4: Evidence and lifecycle closure

- [x] [serial] I5 Record source, target, resolved tree, conflict decisions, canonical digest, policy results, test results, and bounded blockers in `evidence/integration-validation.md`. r[build_correctness.dynamic_plan_nominal.integration.evidence] Evidence: `evidence/integration-validation.md` and `evidence/octet-nominal-domain-2026-07-28.json` contain the bounded record.
- [x] [serial] V9 Run `cairn validate --root .`, `cairn gate proposal integrate-nominal-dynamic-plan-branch --root .`, `cairn gate design integrate-nominal-dynamic-plan-branch --root .`, and `cairn gate tasks integrate-nominal-dynamic-plan-branch --root .`. Require all integration tasks complete before sync. r[build_correctness.dynamic_plan_nominal.integration.evidence] Evidence: pueue tasks `3465` and `3466` returned `valid: true` and `verdict: PASS` through the tasks gate.
- [ ] [serial] I6 Sync and archive this integration change, then commit the resolved integration on its dedicated branch. Do not push or move main without explicit instruction. r[build_correctness.dynamic_plan_nominal.integration.lifecycle]
