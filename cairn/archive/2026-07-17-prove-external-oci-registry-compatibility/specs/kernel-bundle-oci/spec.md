## ADDED Requirements

### Requirement: Registry compatibility is tested against an independent implementation

r[kernel_bundle_oci.registry_external_compatibility] Mantle MUST retain an executable compatibility rail that exercises the production signed OCI registry workflow against one repository-pinned independent OCI Distribution implementation.

#### Scenario: Independent registry round trip succeeds

GIVEN the pinned registry fixture, a gallery OCI layout, typed trust policy, and explicit signing key
WHEN the external compatibility rail runs signed push followed by immutable-digest pull into fresh state
THEN the production CLI MUST publish, verify, reconstruct, and admit the exact layout
AND ordinary image/index plus metadata/signature companion manifests MUST use OCI image-manifest schema version 2 accepted by the independent implementation
AND contracted push, pull, and import reports MUST pass their ordinary validators.

#### Scenario: Wrong immutable signature digest fails closed

GIVEN a successful external-registry publication and a pull request naming a different signature-manifest digest
WHEN the negative compatibility path runs
THEN Mantle MUST reject the pull before local layout publication or admission
AND no import report or successful pull receipt MAY exist.

#### Scenario: Compatibility claim remains bounded

GIVEN the pinned external fixture passes
WHEN documentation or lifecycle evidence describes compatibility
THEN it MUST identify the tested implementation and version
AND it MUST NOT infer arbitrary registry compatibility, registry authorization, authenticated deployment, redirect/proxy behavior, TLS/PKI correctness, tag immutability, Referrers API support, artifact correctness, bootability, deployability, or release eligibility.
