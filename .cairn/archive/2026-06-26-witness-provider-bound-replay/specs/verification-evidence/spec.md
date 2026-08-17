# Verification Evidence Specification Delta

## MODIFIED Requirements

### Requirement: Release witness rebuild binds all signed outputs [r[verification_evidence.release_witness_rebuild_multi_output]]

Mantle MUST make `mantle release witness-rebuild` bind every published release output in a witness attestation to a rebuilt proof artifact whose BLAKE3 digest matches the corresponding release output. Multi-output release requests MUST be supported when the workflow proof bundle and any workflow-produced provider fixed-point proof bundle contain trustworthy artifacts for every expected digest, and the command MUST fail closed before signing witness sidecars when any expected output is missing, ambiguous, path-escaping, invalid, or digest-mismatched.

#### Scenario: multi-output witness rebuild signs matching proof artifacts

- GIVEN a witness request for a release whose signed release attestation names multiple published binary digests
- AND the completed witness workflow proof bundle contains rebuilt proof artifacts whose BLAKE3 digests match every published binary digest
- WHEN `mantle release witness-rebuild` completes
- THEN the command MUST create one witness attestation whose rebuilt digest set covers every published binary in release order
- AND it MUST write audit metadata naming the rebuilt proof artifact path and digest for each signed output.

#### Scenario: provider-bound witness rebuild uses witness-produced provider proof

- GIVEN a witness request for a provider-bound release whose signed release attestation names a provider fixed-point binary digest
- AND the witness workflow produces a provider fixed-point proof bundle under the witness-owned provider proof directory
- WHEN `mantle release witness-rebuild` evaluates rebuilt outputs
- THEN the command MUST validate the provider proof bundle before accepting its stage binary as a rebuilt output candidate
- AND the provider candidate MUST be selected only when its BLAKE3 digest matches the signed provider release artifact.

#### Scenario: incomplete proof artifacts fail closed

- GIVEN a witness request for a release whose signed release attestation names multiple published binary digests
- AND the completed witness workflow proof bundle and provider proof bundle lack a trustworthy rebuilt artifact for at least one published digest
- WHEN `mantle release witness-rebuild` evaluates rebuilt outputs
- THEN the command MUST exit non-zero before writing a witness attestation
- AND the diagnostic MUST identify the missing or mismatched expected output.

#### Scenario: escaping proof artifact paths fail closed

- GIVEN a witness workflow proof manifest or provider proof metadata names a rebuilt artifact path
- AND that path escapes its witness-owned proof bundle root
- WHEN `mantle release witness-rebuild` evaluates rebuilt outputs
- THEN the command MUST exit non-zero before signing witness material
- AND the diagnostic MUST identify the path containment failure.
