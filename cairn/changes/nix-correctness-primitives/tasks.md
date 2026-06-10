# Tasks

## Phase 1: Generic action and CAS contracts

- [ ] [serial] Define `mantle-action-spec-v1`, canonical BLAKE3 action refs, declared input/toolchain/output fields, sandbox policy, network policy, and expected reference policy r[build_correctness.action_spec]
- [ ] [serial] Define Nickel evaluation source-closure receipts for root `.ncl` source, transitive deps/import policy, evaluator identity, export/build-IR shape, and output digest r[build_correctness.nickel_eval_source_closure]
- [ ] [serial] Define `mantle-object://blake3/<digest>` CAS object refs and object manifests for files, directories, symlinks, generated payloads, and redacted secret descriptors r[build_correctness.cas_object_store]
- [ ] [parallel] Add pure canonicalization tests proving action refs, Nickel evaluation receipts, and object refs are deterministic and path views do not become identity r[build_correctness.action_spec] r[build_correctness.nickel_eval_source_closure] r[build_correctness.cas_object_store]

## Phase 2: Execution policy and reference scanning

- [ ] [serial] Add hermetic execution policy validation that fails closed when requested sandbox or network restrictions cannot be enforced r[build_correctness.hermetic_execution_policy]
- [ ] [serial] Add output reference scan DTOs and validation for declared refs, forbidden refs, duplicate conflicting views, path traversal, and secret-byte leakage r[build_correctness.output_reference_scanning]
- [ ] [parallel] Preserve the build-tool boundary by accepting frontend spec refs and expected refs as data without interpreting Onix/NixOS module semantics r[build_tool_boundary.correctness_primitives_frontend_neutral]

## Phase 3: Receipts, reuse, and substitution

- [ ] [serial] Define `mantle-action-receipt-v1` with action refs, produced object refs, reference-scan refs, sandbox reports, producer identity, signature refs, and build/reuse reason r[verification_evidence.build_correctness_receipts]
- [ ] [serial] Implement reuse/substitution admission checks that reject stale action refs, stale CAS refs, policy mismatch, missing signatures when required, and path-only identity r[build_correctness.reuse_and_substitution]
- [ ] [parallel] Add JSON/report rendering that keeps correctness claims bounded to declared action/object evidence r[verification_evidence.build_correctness_receipts]

## Phase 4: Tests and documentation

- [ ] [parallel] Add positive tests for action spec canonicalization, Nickel evaluation provenance, CAS admission, sandbox report binding, reference scan acceptance, receipt emission, and reuse admission r[build_correctness.action_spec] r[build_correctness.nickel_eval_source_closure] r[verification_evidence.build_correctness_receipts]
- [ ] [parallel] Add negative tests for digest mismatch, undeclared Nickel imports, undeclared input refs, unsupported sandbox policy, forbidden output refs, unsigned required receipts, stale substitution, path-only identity, and plaintext secret bytes r[build_correctness.nickel_eval_source_closure] r[build_correctness.output_reference_scanning] r[build_correctness.reuse_and_substitution]
- [ ] [serial] Update docs/examples to describe action refs, object refs, receipt policy, frontend-neutral boundaries, and non-goals r[verification_evidence.build_correctness_receipts]
- [ ] [serial] Run focused tests, `cargo fmt`, relevant workspace tests, `cairn validate --root .`, and proposal/design/tasks gates with evidence before archive r[verification_evidence.build_correctness_receipts]
