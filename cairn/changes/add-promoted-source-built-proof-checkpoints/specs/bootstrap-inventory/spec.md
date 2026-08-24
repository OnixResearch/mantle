## ADDED Requirements

### Requirement: Promoted provider checkpoints compose prior execution evidence

r[bootstrap_inventory.source_built_mantle_checkpoint_reuse] Mantle MUST admit a promoted provider checkpoint only when its stage-specific source, policy, predecessor-output, resource, semantic-output, execution-evidence, and payload identities revalidate against the current proof authority. A restored stage MUST remain distinct from current-attempt execution.

#### Scenario: completed provider stages publish before stage1

GIVEN a promoted proof completes StageX transition, StageX provider publication, native-provider construction, and Rust-provider construction
WHEN Mantle validates those stages before stage1
THEN it MUST publish one immutable provider checkpoint with all four ordered stage records and every required payload identity, including native and Rust-provider action evidence plus source-built Rust host-tool bytes and receipts
AND it MUST bind the checkpoint to promoted origin, stage authorities, semantic provider identities, execution evidence, action-trust policy, resource bounds, and no-fallback status.

#### Scenario: exact checkpoint continues in a fresh root

GIVEN a complete promoted provider checkpoint exists
AND its relevant source, policy, predecessor-output, resource, semantic-output, execution-evidence, and payload identities match the current proof
WHEN a new proof selects the checkpoint store
THEN Mantle MUST restore the payload into a fresh staging directory
AND it MUST remeasure the restored content, preserve origin binding bytes, and validate the provider-relative relocation of current closure, Rust, native, host-tool, receipt, and attestation paths before continuing at Mantle stage1
AND the final receipt MUST label the first four stages as restored with the checkpoint digest.

#### Scenario: unrelated later source does not invalidate provider authority

GIVEN a promoted provider checkpoint is valid
AND only Mantle source or vendor input used by stage1 changes
WHEN Mantle derives the provider checkpoint lookup identity
THEN the provider identity MUST remain unchanged
AND stage1 MUST still bind the new Mantle source and vendor identities.

#### Scenario: relevant authority change rejects reuse

GIVEN a checkpoint differs from current StageX, native, Rust, policy, resource, predecessor-output, semantic-output, execution-evidence, action-trust, or payload authority
WHEN Mantle evaluates that candidate
THEN it MUST reject the checkpoint before restore or execution
AND a present malformed, partial, conflicting, dev-origin, or mismatched candidate MUST fail closed rather than become a silent cache miss.

#### Scenario: cold execution remains available

GIVEN no promoted checkpoint store is selected
WHEN the source-built proof starts
THEN Mantle MUST read no checkpoint state
AND it MUST execute the existing cold provider path and record all four stages as executed.

#### Scenario: checkpoint claim remains bounded

GIVEN a final fixed-point receipt composes restored and current execution
WHEN operators cite the proof
THEN the receipt MUST identify checkpoint digests, original execution evidence, restored payload identities, current stage executions, and the exact authority boundary
AND it MUST NOT claim that restored stages executed in the adopting attempt or that content identity alone proves provenance.
