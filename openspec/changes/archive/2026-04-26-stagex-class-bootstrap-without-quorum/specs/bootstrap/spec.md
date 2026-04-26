## ADDED Requirements

### Requirement: StageX-class bootstrap lineage root

Crunch MUST define a StageX-class bootstrap lineage whose trusted bootstrap root is an auditable seed plus source artifacts, not a prebuilt compiler, prebuilt build tool, Nix store path, or musl.cc-derived binary provider.
ID: bootstrap.stagex.lineage.root

The lineage MUST name every source artifact, generated artifact, patch,
transition tool, seed byte sequence, digest, provenance note, and expected
normalized provider output from the audited seed to the first provider that
satisfies `bootstrap/seed.ncl`. An accepted audited seed MUST declare a
`seed_class` from the closed allowlist `hex0-seed`, its instruction set or
bytecode language, entry point, I/O contract, allowed syscall or host-interface
surface, checked-in human-readable source representation, source-to-byte
reproduction transcript, manual audit note, and `audit_seed_max_bytes` budget.
The `hex0-seed` class is limited to a hand-audited byte seed whose checked-in
hex0 source can rebuild the first hex0 compiler before any general compiler or
shell is trusted. The default `audit_seed_max_bytes` budget is 4096 bytes; any
larger seed or new seed class requires a separate OpenSpec change and ADR before
it can satisfy this profile. Crunch-owned fingerprints MUST use BLAKE3 unless an
upstream interoperability check requires another algorithm and records the
reason. Host kernel, CPU, firmware, container runtime, and filesystem behavior
MAY remain environmental assumptions, but they MUST be recorded outside the
source lineage so they do not masquerade as source-built inputs.

#### Scenario: Auditable seed lineage is accepted

- GIVEN a lineage manifest with seed bytes, stage0 source artifacts, patches,
  generated artifact rules, BLAKE3 digests, provenance, and expected normalized
  provider outputs
- WHEN lineage validation runs
- THEN validation succeeds
- AND the output lists the complete source-to-provider stage graph

#### Scenario: Seed without audit bounds is rejected

- GIVEN a lineage manifest whose seed omits `seed_class`, entry point, I/O
  contract, allowed host-interface surface, human-readable source, reproduction
  transcript, audit note, or `audit_seed_max_bytes`
- WHEN lineage validation runs for the StageX-class profile
- THEN validation fails
- AND the diagnostic names the missing audit-bound field

#### Scenario: Unsupported seed class is rejected

- GIVEN a lineage manifest whose `seed_class` is not `hex0-seed`
- WHEN lineage validation runs for the StageX-class profile
- THEN validation fails
- AND the diagnostic says new seed classes require separate OpenSpec and ADR
  approval

#### Scenario: Oversized seed needs separate approval

- GIVEN a lineage manifest whose seed byte length exceeds its declared
  `audit_seed_max_bytes` budget
- WHEN lineage validation runs for the StageX-class profile
- THEN validation fails
- AND the diagnostic says a larger seed requires separate OpenSpec and ADR
  approval

#### Scenario: Prebuilt compiler root is rejected

- GIVEN a lineage manifest that declares a host `cc`, `c++`, `make`, archive
  tool, Nix store path, or musl.cc binary tarball as a trusted bootstrap root
- WHEN lineage validation runs for the StageX-class profile
- THEN validation fails
- AND the diagnostic names the forbidden prebuilt root

#### Scenario: Environmental assumptions stay separate

- GIVEN a lineage manifest that records Linux kernel or CPU assumptions
- WHEN lineage validation runs
- THEN those assumptions are reported as environment assumptions
- AND they are not counted as source-built lineage nodes

### Requirement: Stage0 lineage produces normalized seed provider

Crunch MUST materialize the normalized seed provider through the declared StageX-class lineage before later bootstrap derivations consume `bootstrap/seed.ncl`.
ID: bootstrap.stagex.lineage.provider

The provider output MUST satisfy the existing normalized seed contract fields
used by later bootstrap derivations, including target-prefixed tools, headers,
libraries, provider metadata, retained-tool metadata, reduction metadata, and
provider notes. Later bootstrap derivations MUST consume only the normalized
contract, not stage0-posix, live-bootstrap, container, or provider-specific raw
layouts. A legacy fetched provider MUST remain available as seed-assisted
fallback evidence, but it MUST NOT satisfy this requirement.

#### Scenario: Lineage provider feeds existing bootstrap stages

- GIVEN a valid StageX-class lineage manifest
- AND the lineage materializes a normalized provider output
- WHEN `crunch build bootstrap/make.ncl` consumes the provider through
  `bootstrap/seed.ncl`
- THEN `make` builds successfully
- AND no later bootstrap derivation reads provider-specific raw paths

#### Scenario: Legacy provider cannot satisfy lineage provider requirement

- GIVEN a self-build or bootstrap run selected the legacy fetched provider
- WHEN StageX-class provider evidence is requested
- THEN the run is classified as seed-assisted only
- AND the StageX-class provider requirement remains unsatisfied

#### Scenario: Raw layout coupling is rejected

- GIVEN a later bootstrap derivation reads a live-bootstrap, stage0-posix, OCI,
  or temporary provider raw path directly
- WHEN provider-boundary validation runs
- THEN validation fails
- AND the diagnostic names the derivation and forbidden raw path

### Requirement: StageX-class self-build proof binds lineage evidence

Crunch MUST require a full self-build proof with the StageX-class lineage provider before reporting StageX-class bootstrap evidence.
ID: bootstrap.stagex.selfbuild.proof

The proof metadata MUST bind the audited seed digest, lineage manifest digest,
stage graph digest, normalized provider digest, staged source digest, stage1 and
stage2 crunch binary digests, bootstrap-tool digests, protected execution audit
digest when used, and final proof bundle digest. The release profile MUST bind
that proof bundle digest to the canonical reproducibility report digest before
any StageX-class verified-build claim is emitted. The proof MUST fail closed
when the lineage provider is missing, legacy-fetched, unvalidated, or when
forbidden host executables run during the protected stage. A prerequisite-only
check MUST NOT count as StageX-class proof evidence.

#### Scenario: Successful proof records lineage evidence

- GIVEN a validated StageX-class lineage provider
- AND a full self-build proof completes with that provider
- WHEN proof metadata is written
- THEN it records the audited seed digest, lineage manifest digest, stage graph
  digest, provider digest, stage1 digest, stage2 digest, and proof bundle digest
- AND it records that the provider kind is StageX-class lineage

#### Scenario: Host executable escape blocks the proof

- GIVEN the protected stage observes an undeclared host compiler, build tool,
  archive tool, shell, Nix command, or legacy provider executable
- WHEN the proof run evaluates StageX-class evidence
- THEN the proof fails closed
- AND no StageX-class claim is emitted

#### Scenario: Prerequisite-only check stays insufficient

- GIVEN `./scripts/prove-self-hosting.sh --check` succeeds
- WHEN bootstrap maturity is reported
- THEN StageX-class proof evidence remains absent
- AND the report names the missing full proof run
