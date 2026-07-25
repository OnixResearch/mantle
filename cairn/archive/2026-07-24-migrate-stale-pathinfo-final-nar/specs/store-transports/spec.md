## ADDED Requirements

### Requirement: Mantle explicitly migrates stale signed final-NAR metadata [r[store_transports.pathinfo_final_nar_migration]]

Mantle MUST provide an explicit dry-run-first migration for a single exact signed PathInfo whose recorded final NAR size or SHA-256 is stale while its local castore node and CA-derived store-path identity remain valid. Migration MUST independently measure complete local content, preserve store path, node, references, CA metadata, deriver, and output bytes, replace all signatures over stale facts with exactly one selected local signature, preserve existing artifact-attestation claims and graph facts while refreshing its observed content digest, and emit a bounded structured report. It MUST NOT mutate during dry run, generate a signing key during dry run, accept fragments or batch selection, silently repair ordinary archive operations, or claim recovered historical signer authority.

#### Scenario: Dry run reports an exact stale candidate without mutation [r[store_transports.pathinfo_final_nar_migration.scenario.dry-run]]

- GIVEN one exact full logical store path selects an already-signed PathInfo with complete local content and valid CA-derived path identity
- AND a fresh final NAR render differs from its recorded size or SHA-256
- WHEN the operator runs `store repair-final-nar` without `--execute`
- THEN Mantle MUST report old and observed final-NAR facts, signature count, sidecar disposition, and `would-repair`
- AND it MUST NOT write PathInfo, artifact sidecars, signing keys, or output bytes.

#### Scenario: Execution replaces stale facts and authority [r[store_transports.pathinfo_final_nar_migration.scenario.execute]]

- GIVEN the dry-run candidate remains unchanged under the store mutation lock
- AND the operator supplies `--execute` with an available selected signing key
- WHEN Mantle executes migration
- THEN it MUST persist the freshly measured final NAR size and SHA-256 while preserving every non-signature PathInfo field
- AND it MUST discard all old signatures, add exactly one signature over the repaired Nix fingerprint, refresh an existing artifact attestation without discarding claims or graph facts, verify the persisted result, and report `repaired`.

#### Scenario: Current metadata is an idempotent no-op [r[store_transports.pathinfo_final_nar_migration.scenario.current]]

- GIVEN the exact selected PathInfo already matches a fresh final NAR render
- WHEN the operator runs dry-run or execute mode
- THEN Mantle MUST report `current`
- AND it MUST NOT rewrite PathInfo, signatures, sidecars, signing keys, or output bytes.

#### Scenario: Unsafe candidate fails before mutation [r[store_transports.pathinfo_final_nar_migration.scenario.reject]]

- GIVEN selection is not an exact full logical store path, PathInfo is missing or unsigned, castore content is missing or incomplete, CA metadata is unsupported or does not derive the selected path, an existing artifact attestation is malformed or identifies a different logical path, or staged sidecar preparation fails
- WHEN Mantle plans or executes migration
- THEN it MUST reject with a deterministic reason before PathInfo mutation
- AND it MUST NOT change signatures, metadata, sidecars, content, or output files.

#### Scenario: Persistence failure is not reported as success [r[store_transports.pathinfo_final_nar_migration.scenario.persistence-failure]]

- GIVEN all preflight checks pass but PathInfo persistence, sidecar publication, rollback, or post-write verification fails
- WHEN Mantle executes migration
- THEN it MUST return a non-success result naming the failed phase and whether rollback restored the prior PathInfo
- AND it MUST NOT claim repaired, recovered historical authority, archive compatibility, content correctness, or release eligibility.
