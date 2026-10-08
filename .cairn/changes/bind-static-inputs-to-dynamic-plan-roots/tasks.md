# Tasks: Bind static inputs to dynamic-plan roots

All tasks were re-verified on published main (port onto
`integ/baseline-health` `9a76abe6`) on 2026-10-07; exact commands and
outputs are in `evidence/finish-2026-10-07.md`. The 2026-09 evidence files
record the earlier unpublished copy and are kept as history only.

## Phase 1: Baseline and contract

- [x] [serial] T1.1 Record the baseline in an isolated worktree: Nickel input variants, conversion identities for representative derivations, plan-root handling, and focused `crunch-glue`, `crunch-eval`, and `crunch-build` test output. r[mantle.dynamic_plan_output_inputs.request_identity]
- [x] [serial] T1.2 Define the Nickel `plan_output` input, the placeholder marker grammar, the canonical binding record, the reference limit, and the failure catalog. r[mantle.dynamic_plan_output_inputs.typed_reference] r[mantle.dynamic_plan_output_inputs.binding_failures]
- [x] [serial] T1.3 Record the request-identity and bind-at-acceptance decisions in an ADR with an index row in `adr/README.md`. r[mantle.dynamic_plan_output_inputs.request_identity]

## Phase 2: Evaluation and conversion

- [x] [serial] T2.1 Add the input to `lib/derivation.ncl` and the crunch-glue input model, and validate declared plan outputs at evaluation. r[mantle.dynamic_plan_output_inputs.typed_reference]
- [x] [serial] T2.2 Convert references into an input edge on the producer's plan output, a canonical binding record, and deterministic placeholders without a derivation-format change. r[mantle.dynamic_plan_output_inputs.request_identity]
- [x] [parallel] T2.3 Add identity fixtures: stable paths across evaluations, a changed root or output changes identity, and unchanged identities for derivations without the input. r[mantle.dynamic_plan_output_inputs.request_identity]
- [x] [parallel] T2.4 Add negative evaluation fixtures: an undeclared plan output, a marker without an input, and over-limit references. r[mantle.dynamic_plan_output_inputs.typed_reference] r[mantle.dynamic_plan_output_inputs.binding_failures]

## Phase 3: Worker binding

- [x] [serial] T3.1 Implement the pure binding core over accepted plans and binding records with typed failures. r[mantle.dynamic_plan_output_inputs.binding_failures]
- [x] [serial] T3.2 Add wait edges from consumers to bound roots after plan acceptance, with failure propagation through the existing path. r[mantle.dynamic_plan_output_inputs.binding_failures]
- [x] [serial] T3.3 Replace placeholders at dispatch, mount bound paths with their closures, add them to reference-scan needles, and refuse dispatch while any reference is unbound. r[mantle.dynamic_plan_output_inputs.dispatch_binding]
- [x] [serial] T3.4 Record bindings in build reports and provenance. r[mantle.dynamic_plan_output_inputs.provenance]
- [x] [parallel] T3.5 Add worker negative controls for a failed producer, a rejected plan, a missing root, a missing output, and a failed root, each without a successful consumer output. r[mantle.dynamic_plan_output_inputs.binding_failures]

## Phase 4: Verification and documentation

- [x] [serial] T4.1 Run an end-to-end fixture where one Nickel wrapper consumes an input-addressed plan root and another consumes a content-addressed root in one `mantle build` run. r[mantle.dynamic_plan_output_inputs.dispatch_binding]
- [x] [serial] T4.2 Run an unchanged second build and prove the consumer is reused without execution. r[mantle.dynamic_plan_output_inputs.request_identity]
- [x] [serial] T4.3 Document the input in the `lib/derivation.ncl` field docs and the dynamic-plan documentation with failure reasons and non-claims. r[mantle.dynamic_plan_output_inputs.provenance]
- [x] [serial] T4.4 Run focused `crunch-glue`, `crunch-eval`, `crunch-build`, and `crunch-pipeline` suites before and after the change, strict Clippy for touched first-party packages, Cairn validation, and the proposal, design, and tasks gates. Preserve exact output in `evidence/`. r[mantle.dynamic_plan_output_inputs.dispatch_binding]
- [ ] [serial] T4.5 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.dynamic_plan_output_inputs.provenance]
