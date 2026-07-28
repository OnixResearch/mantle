# Tasks

## Phase 1: Bind source and target state

- [ ] [serial] V1 Verify the recorded source commit, tree, parent, subject, and changed-file set. Record the result in integration evidence. r[build_correctness.dynamic_plan_nominal.integration.source]
- [ ] [serial] V2 Select a committed main target that preserves the unrelated StageX work. Record its commit and require a clean integration worktree. r[build_correctness.dynamic_plan_nominal.integration.target]
- [ ] [serial] I1 Create a dedicated integration branch and worktree from the selected target commit. Do not use the current dirty main checkout. r[build_correctness.dynamic_plan_nominal.integration.target]

## Phase 2: Apply and reconcile the source commit

- [ ] [serial] I2 Apply commit `a2c872cfe6826df2d3ca7c85597365e6076fdfba` to the integration branch and record every conflict. r[build_correctness.dynamic_plan_nominal.integration.source]
- [ ] [serial] I3 Reconcile accepted specs, archived evidence, README content, documentation, Cargo metadata, and Dylint policy as a semantic union. r[build_correctness.dynamic_plan_nominal.integration.preservation]
- [ ] [serial] I4 Reconcile dynamic-plan and worker code while retaining checked constructors, private fields, explicit wire admission, typed graph storage, and digest roles. r[build_correctness.dynamic_plan_nominal.integration.preservation]
- [ ] [parallel] V3 Audit the resolved diff against both parents. Reject deleted target work, missing source files, raw-string graph fallbacks, and weakened constructors. r[build_correctness.dynamic_plan_nominal.integration.preservation]

## Phase 3: Revalidate the resolved integration tree

- [ ] [parallel] V4 Run focused `crunch-build` positive, negative, graph, placeholder, producer, worker, and compatibility tests. r[build_correctness.dynamic_plan_nominal.integration.validation]
- [ ] [parallel] V5 Run all compile-fail category tests and malformed wire admission tests. r[build_correctness.dynamic_plan_nominal.integration.validation.negative]
- [ ] [parallel] V6 Compare the frozen canonical JSON and require plan digest `dc6814c1f500dc7e8575c3fd84a64ae78a70d38313ccbbfff4fbfcf7610f6750`. r[build_correctness.dynamic_plan_nominal.integration.compatibility]
- [ ] [parallel] V7 Run Octet and require zero `primitive_domain_alias`, `raw_domain_value`, and `newtype_invariant_bypass` findings in the declared scope. r[build_correctness.dynamic_plan_nominal.integration.policy]
- [ ] [parallel] V8 Run first-party Clippy, Tiger Style, relevant Nix checks, and broader workspace tests. Triage every failure against the selected target baseline. r[build_correctness.dynamic_plan_nominal.integration.validation]

## Phase 4: Evidence and lifecycle closure

- [ ] [serial] I5 Record source, target, resolved tree, conflict decisions, canonical digest, policy results, test results, and bounded blockers in `evidence/integration-validation.md`. r[build_correctness.dynamic_plan_nominal.integration.evidence]
- [ ] [serial] V9 Run Cairn validation and proposal, design, and tasks gates. Require all integration tasks complete before sync. r[build_correctness.dynamic_plan_nominal.integration.evidence]
- [ ] [serial] I6 Sync and archive this integration change, then commit the resolved integration on its dedicated branch. Do not push or move main without explicit instruction. r[build_correctness.dynamic_plan_nominal.integration.lifecycle]
