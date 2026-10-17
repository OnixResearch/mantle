# Specification: Current semantic reconciliation boundaries

## ADDED Requirements

### Requirement: One owning API per shared Cargo concept

r[mantle.semantic_reconciliation.api_cutover] The current integration MUST retain supported behavior from both named parents and migrate all affected consumers to one owning shared API, without obsolete aliases or parallel Cargo run-binding DTOs. Pure decisions MUST remain separate from shell effects.

#### Scenario: Native build-script inputs reach a real consumer

- GIVEN admitted native inputs, explicit runtime roots and a declared Cargo build-script run binding
- WHEN the helper executes the script and compiles the downstream consumer
- THEN the linked consumer MUST observe the declared native result
- AND missing native declarations MUST fail rather than discovering ambient host paths

### Requirement: Distinct Android routes preserve operation-specific bounds

r[mantle.semantic_reconciliation.android_boundaries] Published and reviewed Android APIs MUST retain their own source-authority checks and renderer-specific entry epoch ranges in Nickel and Rust. Both MUST start at 315532802; published MUST end at 4354819199 and reviewed MUST end at 4102444799. ZIP two-second representation MUST NOT be reported as one-second fidelity or SDK proof.

#### Scenario: A timestamp admitted by only the published route

- GIVEN integral epoch 4102444800, locale C, timezone UTC and each route's admitted source records
- WHEN both authoring contracts are evaluated
- THEN published authoring MUST admit the timestamp and reviewed authoring MUST reject it
- AND each route MUST reject epochs 315532800 and 315532801

### Requirement: Selected-prefix trust is consistent across inspection paths

r[mantle.semantic_reconciliation.prefix_list_trust] Layered get, composed list and referenced-path verification MUST use the selected logical prefix in signature authority. Visible base records MUST be listed in layer precedence; a raw overlay's empty stream MUST NOT stand in for composed inspection. Revoked destination Casita trust MUST NOT be authorized by cached metadata.

#### Scenario: A valid signature under the wrong prefix

- GIVEN a base record and reference signed under a different logical prefix
- WHEN a selected-prefix client requests layered get or composed listing
- THEN both inspection paths MUST reject visible untrusted records
- AND a correctly signed selected-prefix record and reference MUST remain observable

### Requirement: Generators own operator artifact identities

r[mantle.semantic_reconciliation.generated_authority] Reconciled operator JSON and generated docs MUST come from actual fresh generator output using current Clap and reviewed Nickel source. A successful Nickel export alone MUST NOT count as root build, generator success or contract mode-check success.

#### Scenario: Current root build is stopped before success

- GIVEN a resource-stopped root/generator attempt without successful fresh generator output
- WHEN integration readiness is recorded
- THEN generated installation and current contract checks MUST remain pending
- AND incompatible catalog identities MUST NOT be hand-spliced into a claimed output

### Requirement: Current evidence and resource epochs remain bounded

r[mantle.semantic_reconciliation.evidence_scope] The integration MUST own a unique current lifecycle record. It MUST distinguish selected smoke subjects from final executable/gate subjects, retain historical raw receipts unchanged, and leave unobserved root/generator/gates pending. Frozen resource caps and the original pool-availability baseline MUST NOT be redefined to conceal prior usage. Proven exclusive-growth credit remains zero and the uncaptured frees/concurrent-allocation correction remains unknown; unknown correction MUST NOT imply a new hard-quota requirement, because the original zero-credit legacy guard already exceeds its threshold. Neither process-leader exit nor a point owned-process observation MUST assert universal descendant/daemon emptiness.

#### Scenario: Selected smoke succeeds while whole-product validation is held

- GIVEN captured current helper/API smoke success and a resource-held root/generator/gate sequence
- WHEN the integration change, ADR and release note are written
- THEN the observed smoke MUST name its exact scope and source timing
- AND whole-product success, archive acceptance, bootstrap/release eligibility and main publication MUST NOT be inferred

#### Scenario: Historical and current lifecycle subjects coexist

- GIVEN retained archived/active prior change records and this uniquely named integration subject
- WHEN current observations are indexed
- THEN old receipt bytes and acceptance states MUST remain unchanged
- AND current evidence MUST NOT complete an older pending campaign task
