# wRPC producer

## ADDED Requirements

### Requirement: Real immutable component pair [r[materialize_wrpc_producer.component_pair]]

Mantle MUST materialize an open hello-importing client and a hello-exporting
server from the pinned source/WIT cohort. Every retained member MUST have its
measured BLAKE3 identity and size. Missing or substituted components MUST NOT
trigger a local guest build. Publication identity MUST come from the actual
published producer revision, never an invented revision or digest.

#### Scenario: Real pair is handed off

- GIVEN the immutable producer source and its locked offline dependencies
- WHEN the actual component exporter completes both pipelines
- THEN each standalone final component and its real stage receipts are published
- AND the public manifest binds both existing materialization bundle identities.

### Requirement: Runtime comparison binds the final client [r[materialize_wrpc_producer.runtime_binding]]

A smoke-only WAC composition MUST bind the exact final portable client once and
identity-check each external provider. Its source, invocation and output MUST be
retained. The final client MUST remain open for real remote RPC. All WASI
subsystems MUST be denied; only the declared non-WASI hello import may remain.

#### Scenario: Closed comparison cannot replace RPC

- GIVEN a measured open client and server
- WHEN the producer executes the WAC comparison
- THEN comparison evidence is explicitly labeled and excluded from remoted guest selection
- AND producer evidence MUST NOT claim real remote RPC execution.

### Requirement: Consumer identity boundaries [r[materialize_wrpc_producer.consumer_contract]]

The bundle MUST retain exact runtime profiles and reference encoding, full WIT,
component and wRPC binding generation evidence, tool/source identities and narrow
capability root layout. Consumer verification MUST reject missing, mismatched,
substituted and unsafe local-replacement members. Equal local bytes alone MUST
NOT be treated as proof of published producer provenance.

#### Scenario: Confined verification

- GIVEN an authorized immutable package capability and its declared capability root
- WHEN the consumer verifies and snapshots the selected bundles
- THEN member access remains beneath that held capability
- AND no ambient root or local guest compilation authority is required.
