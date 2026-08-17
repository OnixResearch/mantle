### Requirement: Native Rust registry source planning fragment

r[rust_package_planning.native_registry_source] Mantle MUST represent supported registry-backed Rust package sources as explicit native source facts before those packages participate in Cargo-free native unit or topology claims.

#### Scenario: Lockfile registry identities are recorded

r[rust_package_planning.native_registry_source.lockfile_identity]

- GIVEN a Cargo lockfile contains a supported registry package entry
- WHEN Mantle computes native Rust source-planning facts
- THEN Mantle MUST record the package name, version, source URL/class, checksum material, lockfile digest, and package identity used by downstream unit graph evidence.
- AND missing checksum or unsupported lockfile source material MUST make the native registry source fragment not ready.

#### Scenario: Declared vendor source roots are digest-bound

r[rust_package_planning.native_registry_source.vendor_digest]

- GIVEN a supported registry package has declared local vendor/source material
- WHEN Mantle computes native Rust source-planning facts
- THEN Mantle MUST bind the package to a deterministic source-root reference and BLAKE3 source-tree digest.
- AND Mantle MUST NOT read `$CARGO_HOME`, Cargo registry caches, Cargo git checkouts, target directories, or network locations as undeclared source material.

#### Scenario: Native registry source facts are compared with Cargo oracle material

r[rust_package_planning.native_registry_source.oracle_compare]

- GIVEN Cargo oracle material is retained for the same workspace and lockfile
- WHEN Mantle supports a registry package in the native source-planning fragment
- THEN Mantle MUST compare native package identity, source class, checksum material, and package/source membership against the Cargo oracle evidence.
- AND native-vs-oracle divergence MUST produce deterministic blockers instead of silently falling back to Cargo-derived source paths.

#### Scenario: Unsupported or stale registry source material fails closed

r[rust_package_planning.native_registry_source.blockers]

- GIVEN a package source is registry-backed or vendor-backed
- WHEN the lockfile checksum is missing, the vendor/source root is missing or unreadable, the source digest mismatches recorded material, the source kind/layout is unsupported, or required source material would come from an ambient Cargo cache
- THEN Mantle MUST emit deterministic native registry source blockers before claiming native unit graph readiness or Cargo-free execution for the affected package.

#### Scenario: Registry source facts are receipt-bound

r[rust_package_planning.native_registry_source.receipts]

- GIVEN Mantle emits `rust-plan` JSON evidence for a workspace with supported registry source material
- WHEN native registry source planning finishes
- THEN the receipt MUST include ready status, ordered registry source facts, lockfile/source digests, oracle comparison evidence, blockers when present, and a stable receipt hash.
- AND the receipt MUST keep the bounded claim explicit: declared local/vendor source planning only, with no network fetch, version solving, remote cache, or general Cargo registry compatibility claim.

#### Scenario: Registry source planning is covered by focused fixtures

r[rust_package_planning.native_registry_source.tests]

- GIVEN Mantle includes focused Rust planner or CLI fixtures for registry source planning
- WHEN tests exercise a supported vendored registry package and missing or stale vendor/source material
- THEN positive fixtures MUST prove ready native source facts with explicit BLAKE3-bound source material.
- AND negative fixtures MUST prove deterministic fail-closed blockers without consulting ambient Cargo caches.

#### Scenario: Registry source planning change closes with lifecycle evidence

r[rust_package_planning.native_registry_source.verify]

- GIVEN the native registry source planning implementation tasks are complete
- WHEN Mantle accepts the change
- THEN focused Rust verification, Cairn validation, and proposal/design/tasks gates MUST pass before sync, archive, commit, and push.
