# Tasks

## Contract

- [ ] [serial] Define input and patch trust policy schema, verifier kinds, signature refs, trusted public key identities, signer/quorum decisions, and digest binding. r[project_workflows.input_trust_policy]
- [ ] [serial] Define bounded evidence and non-claim wording for trusted input refresh results. r[verification_evidence.project_input_trust_claims]

## Implementation

- [ ] [serial] Implement pure trust policy validation, verified-fact normalization, lock update acceptance/rejection, and attestation data shaping. r[project_workflows.input_trust_policy] r[verification_evidence.project_input_trust_claims]
- [ ] [serial] Implement shell trust verification for supported local fixture verifier paths without key-server or forge trust. r[project_workflows.input_trust_policy]
- [ ] [serial] Thread trust results through `mantle refresh`, patch resolution, lock update reports, and project attestations where applicable. r[project_workflows.input_trust_policy]

## Verification

- [ ] [serial] Add positive pure tests for accepted signer facts, quorum satisfaction, input and patch policy application, lock update acceptance, and bounded claim shaping. r[project_workflows.input_trust_policy] r[verification_evidence.project_input_trust_claims]
- [ ] [serial] Add negative pure tests for missing signatures, wrong keys, bad signatures, digest mismatch, unsupported verifier kinds, malformed key refs, and overbroad claims. r[project_workflows.input_trust_policy] r[verification_evidence.project_input_trust_claims]
- [ ] [serial] Add shell fixture tests using local keys/signatures proving invalid trust blocks lock writes and no key-server lookup occurs. r[project_workflows.input_trust_policy]
- [ ] [serial] Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and proposal/design/tasks gates before marking implementation tasks complete. r[project_workflows.input_trust_policy]
