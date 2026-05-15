## MODIFIED Requirements

### Requirement: Release verification consumes deterministic proof receipts without overclaiming

Release verification MUST promote a release artifact beyond `self-rebuild-match`
to a stronger deterministic-release claim only when it is backed by
deterministic build proof receipts whose verdicts are `deterministic-match` for
every required artifact output in the release digest set and whose deterministic
proof runs were executed through a supported sandbox envelope.

The deterministic proof receipt MUST record a canonical sandbox profile identity
for every proof run. The profile identity MUST cover the sandbox executor kind
and version, network policy, read-only and writable bind set, environment
allowlist, working-directory policy, rebuild recipe identity, logical store
prefix, and physical proof-store/output locations.

A deterministic-release claim MUST remain scoped to the named release artifacts,
workflow identity, derivation identities, toolchain/provider identities, sandbox
profile identity, and recorded proof matrix. It MUST NOT claim global Mantle
determinism, all-package reproducibility, or full-source bootstrap determinism
unless separate evidence proves those broader claims.

#### Scenario: Release has sandboxed deterministic proof for every artifact

- GIVEN a release evidence bundle with required artifact digests
- AND each required artifact has a deterministic-build proof receipt with verdict
  `deterministic-match`
- AND every receipt records a supported sandbox profile identity
- AND the receipt BLAKE3 digest set equals the release artifact digest set
- WHEN release verification evaluates deterministic claim eligibility
- THEN it may report a deterministic-release claim scoped to those artifacts and
  the recorded sandbox profile

#### Scenario: Deterministic proof receipt lacks supported sandbox evidence

- GIVEN a deterministic-build proof receipt whose output digests match the
  release artifacts
- BUT the receipt lacks a sandbox profile identity or records an unsupported,
  bypassed, or direct-host executor profile
- WHEN release verification evaluates deterministic claim eligibility
- THEN the release remains at the strongest lower satisfied proof class
- AND the report identifies unsupported deterministic proof sandbox evidence as
  the blocker

#### Scenario: Deterministic release claim remains bounded

- GIVEN a release artifact has a valid deterministic-build proof receipt
- WHEN Mantle renders human-readable release verification output
- THEN it states the claim scope as release-artifact determinism under the
  recorded workflow, sandbox profile, and proof matrix
- AND it does not call the whole build system Nix-like deterministic by default
