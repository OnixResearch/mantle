# Specification: Store backends

## ADDED Requirements

### Requirement: Store backend selection is explicit

r[mantle.store_backends.explicit_selection] Every Mantle store open MUST carry an explicit backend identifier chosen at a composition root. The CLI MUST expose one global `--store-backend <id>` option whose default is `snix`, applied only at the CLI composition root. Library store constructors MUST require the identifier and MUST NOT supply a default. Every child process that Mantle launches against the same state directory MUST receive the selected identifier explicitly. Mantle MUST reject an identifier that is not admitted with `store-backend-unknown` before it reads, creates, or modifies any state. Selection MUST NOT come from environment variables, directory contents, or build features.

#### Scenario: Default selection is Snix

- GIVEN an operator runs a store-opening command without `--store-backend`, with the fixture signing key and environment recorded with the pre-change goldens
- WHEN the composition root constructs the store
- THEN Mantle MUST open the store with the `snix` backend
- AND store paths, NAR SHA-256 values, signed PathInfo, and GC plan identities MUST equal the goldens recorded before this change

#### Scenario: Child process receives the selection

- GIVEN an operator selects a backend for a command that launches a Mantle child process against the same state directory
- WHEN Mantle launches the child process
- THEN the child arguments MUST carry the same backend identifier
- AND a launcher that omits the identifier MUST fail the forwarding fixture

#### Scenario: Unknown identifier

- GIVEN an operator passes `--store-backend tape`
- WHEN Mantle parses the command
- THEN it MUST fail with `store-backend-unknown`
- AND it MUST NOT create, open, or modify the state directory

#### Scenario: Library constructor cannot omit the backend

- GIVEN a Rust caller constructs `StoreConfig` without its `backend` field while an existing selected store state has been seeded
- WHEN the caller is compiled against `crunch-store`
- THEN compilation MUST fail with `E0063`, missing field `backend`
- AND the seeded state MUST remain byte-identical because the invalid caller cannot execute
- AND this compile-time rejection MUST NOT be represented as a runtime backend blocker

### Requirement: The state directory records its backend

r[mantle.store_backends.state_identity] Mantle MUST record the backend identifier in a versioned state identity record when it creates a state directory. A legacy identity record without a backend field MUST be interpreted as `snix` and MUST NOT be rewritten by ordinary commands. A backend other than `snix` MUST NOT write the legacy record schema.

#### Scenario: New state directory

- GIVEN an empty state directory and the selected backend `snix`
- WHEN Mantle opens the store
- THEN the identity record MUST carry the versioned schema, logical prefix, trust policy identity, and backend `snix`

#### Scenario: Legacy state directory

- GIVEN a state directory whose identity record uses `mantle-store-state-v1` without a backend field
- WHEN Mantle opens it with the backend `snix`
- THEN the open MUST succeed
- AND the identity record MUST remain byte-identical

### Requirement: Mixed opens fail closed

r[mantle.store_backends.mixed_open_rejection] Mantle MUST reject with `store-backend-mismatch` a state directory whose recorded backend differs from the selected backend, and a state directory without an identity record that contains the declared persistent-state markers of another admitted backend. For a state directory without an identity record, the rule is asymmetric. With `snix` selected, a directory that holds entries but no `casita` repository marker (`<state-dir>/casita`) is legacy Snix state: before any Snix service opens, Mantle MUST add an identity record that names `snix`, as the pre-selection code did, and the identity decision and write MUST NOT change any existing file. The Snix services MAY then update their own database files (`pathinfo.redb`, `directories.redb`) while opening, and Mantle MUST NOT claim those files stay byte-identical after the open. A directory without an identity record that holds the `casita` marker MUST be rejected with `store-backend-mismatch` whichever backend is selected. With `casita` selected, a directory without an identity record MUST be rejected with `store-backend-mismatch` when it holds any entry other than the store mutation lock, the local signing key, and the trust-policy files that a caller or operator may provision before the first open. Mantle MUST NOT infer `casita` from file markers. The decision MUST run before any storage service opens and before any file or directory is created or modified. Every overlay layer MUST record the same backend as the writable store, and a differing layer MUST fail the composition before any base read.

#### Scenario: Recorded backend differs

- GIVEN a state directory whose identity record names a backend other than the selected one
- WHEN Mantle opens the store
- THEN it MUST fail with `store-backend-mismatch` naming the selected and recorded backends
- AND every file in the state directory MUST remain byte-identical

#### Scenario: Foreign state without an identity record

- GIVEN a state directory without an identity record that contains the `casita` repository marker, alone or beside Snix files
- WHEN Mantle opens it with `snix` or `casita` selected
- THEN it MUST fail with `store-backend-mismatch`
- AND it MUST NOT create an identity record, lock file, or storage file

#### Scenario: Legacy Snix state without an identity record

- GIVEN a state directory without an identity record that holds Snix persistence files or legacy root records and no `casita` marker
- WHEN Mantle opens it with `snix` selected
- THEN before any Snix service opens, every existing file MUST remain byte-identical and the only new entry MUST be an identity record that names `snix`
- AND after the open, existing files other than the Snix database files MUST remain byte-identical, and existing PathInfo MUST still resolve

#### Scenario: Unidentified content under Casita

- GIVEN a state directory without an identity record that holds any entry besides the store mutation lock, the local signing key, and trust-policy files
- WHEN Mantle opens it with `casita` selected
- THEN it MUST fail with `store-backend-mismatch` before any effect
- AND it MUST NOT create an identity record or change any file

#### Scenario: Overlay layers disagree

- GIVEN a writable store and a declared base whose identity record names a different backend
- WHEN Mantle composes the overlay
- THEN the composition MUST fail with `store-backend-mismatch` before any base read
- AND neither state directory MUST be modified

### Requirement: No silent fallback across local backends

r[mantle.store_backends.no_silent_fallback] Lookup, closure resolution, export, GC, repair, and inspection MUST use only the selected backend and explicitly declared overlay bases that record the same backend. A miss, corruption, or failure in the selected backend MUST be reported as such and MUST NOT read another local backend's state, in the same state directory or elsewhere. Configured remote substitution MUST keep its existing admission policy, and substituted outputs MUST be admitted into the selected backend. Moving content between local backends MUST require explicit operator commands.

#### Scenario: Miss with leftover files of another backend

- GIVEN a state directory recorded for one backend that also contains leftover files in another backend's layout
- WHEN a lookup misses in the selected backend
- THEN Mantle MUST report the miss or realize the output through the selected backend
- AND it MUST NOT read, import, or report the leftover files as store content

#### Scenario: Corruption is not masked

- GIVEN the selected backend reports an integrity failure for a requested path
- WHEN Mantle resolves that path
- THEN it MUST fail with the selected backend's integrity failure class
- AND it MUST NOT use content from another local backend

#### Scenario: Remote substitution keeps its policy

- GIVEN a configured substituter and trusted keys that admit a requested output under the existing substitution policy
- WHEN the selected backend misses that output
- THEN Mantle MAY substitute it under the existing policy
- AND it MUST admit the result into the selected backend only

### Requirement: Admission invariants are backend-neutral

r[mantle.store_backends.admission_invariants] Every backend MUST preserve the logical store prefix, derivation hashes, output store paths, NAR SHA-256 and size, references, deriver and content-address facts, trusted-key admission, action refs, report schemas, and the store capability views `BuildStore`, `OutputLookup`, `RootRegistry`, `ActionResultPort`, `SourceAdmission`, `StoreAdmin`, and `TransferObjectStore`. For the same signing key, the same deterministic derivation and output, and the same build environment, backend choice MUST NOT change PathInfo signatures or attestation digests. Ed25519 PathInfo signatures depend on the signing key, so a comparison across state directories with different signing keys MUST compare only unsigned fields and MUST verify each signature under its own key. A backend MUST NOT admit an output whose measured NAR facts differ from its signed PathInfo, and backend choice MUST NOT change any Mantle-owned identity.

#### Scenario: Same derivation and signing key under two backends

- GIVEN one deterministic derivation fixture, one explicitly provisioned signing key, and one fixed build environment used by two state directories with different admitted backends
- WHEN Mantle builds and admits the output in each state directory
- THEN the store path, NAR SHA-256, NAR size, references, and PathInfo signatures MUST be equal
- AND the build and store report schemas MUST be equal

#### Scenario: Different signing keys

- GIVEN the same deterministic derivation fixture admitted in two state directories whose signing keys differ
- WHEN the conformance rail compares the admitted outputs
- THEN the store path, NAR SHA-256, NAR size, references, deriver, and content-address facts MUST be equal
- AND each PathInfo signature MUST verify under its own state directory's signing key

#### Scenario: Measured NAR facts disagree

- GIVEN a backend measures NAR facts for an output that differ from its signed PathInfo
- WHEN admission runs
- THEN Mantle MUST reject the output before it becomes visible to lookup
- AND it MUST NOT report the output as admitted

### Requirement: Backends declare optional capabilities and bounds

r[mantle.store_backends.capability_profile] Every admitted backend MUST implement the core capabilities: output admission, lookup, fresh-process reopen, closure resolution, export, store archive export and import, in-place signature update through `store sign`, PathInfo-backed action-result output storage and reuse through `ActionResultPort`, GC planning, and plan-bound GC execution. Final-NAR repair through `store repair-final-nar` is listed per backend: a profile MUST list `store-repair-final-nar` among its core capabilities only when the backend implements it. A backend without it MUST reject `store repair-final-nar`, as a dry run or with `--execute`, with a stable backend-specific blocker before it creates or opens the state directory, and MUST reject the library repair calls on an open store with the same blocker before any effect. Each backend MUST declare in a Mantle-owned capability profile whether it supports each optional capability and any bound that the backend itself imposes on it. Limits of an input format, such as the Nario v2 reader's record limit, apply to every backend and are not profile bounds. The optional capabilities are overlay composition with `--base-store`, atomic multi-path batch import such as Nario v2 import, unsigned admission through `--trust-unsigned` or `--nario-trust-unsigned`, and the standalone Rust unit cache `rust-unit-cache`, which keeps castore blob and directory references across sessions. A request for an undeclared optional capability MUST fail with a stable backend-specific blocker naming the capability before any state access, and a request beyond a declared bound MUST fail with a stable backend-specific blocker before any state mutation. Store reports and documentation MUST list the selected backend's declared profile and bounds, and no report, document, or status reply MUST describe backends as interchangeable beyond their declared profiles.

#### Scenario: Declared optional capability passes its fixtures

- GIVEN a backend that declares an optional capability as supported
- WHEN the conformance rail runs
- THEN the capability's positive and negative fixtures MUST pass for that backend

#### Scenario: Undeclared optional capability fails closed

- GIVEN a backend that does not declare overlay composition
- WHEN an operator selects that backend with one or more `--base-store` layers
- THEN Mantle MUST fail with the backend's stable blocker naming overlay composition
- AND it MUST NOT open or modify any layer

#### Scenario: Request beyond a declared bound

- GIVEN a backend that declares atomic batch import with a maximum batch size N
- WHEN an operator imports a batch of N paths and a batch of N + 1 paths
- THEN the batch of N paths MUST pass the capability's positive fixture
- AND the batch of N + 1 paths MUST fail with the backend's stable blocker before any state mutation

#### Scenario: Profile is visible to operators

- GIVEN two admitted backends with different profiles
- WHEN an operator reads `store info` output or the store documentation
- THEN the selected backend's declared optional capabilities and bounds MUST be listed
- AND the output MUST NOT claim full interchangeability between the backends

#### Scenario: Rust unit cache on a backend without it

- GIVEN a backend whose profile does not declare `rust-unit-cache`
- WHEN a command opens the persistent Rust unit cache
- THEN it MUST fail with the backend's stable blocker before any effect
- AND PathInfo-backed action-result outputs MUST still be stored and reused through `ActionResultPort`

#### Scenario: Final-NAR repair on a backend without it

- GIVEN a backend whose profile does not list `store-repair-final-nar`
- WHEN an operator runs `store repair-final-nar` under that backend, as a dry run or with `--execute`
- THEN it MUST fail with the backend's stable blocker before it creates or opens the state directory
- AND a library repair call on an open store of that backend MUST fail with the same blocker before any effect

### Requirement: Every admitted backend passes one conformance rail

r[mantle.store_backends.conformance_rail] Mantle MUST maintain one backend-parameterized conformance rail that every admitted backend passes before its admission. The rail MUST cover the core capabilities, stale-plan rejection, and the identity and mismatch checks for every backend. It MUST run each optional capability's fixtures, including its bound, for backends that declare it and the fail-closed fixture for backends that do not. Every run MUST use an explicitly provisioned fixture signing key and a recorded environment. The `snix` run MUST equal the goldens recorded before this change when it uses the fixture signing key and environment recorded with those goldens. The rail MUST compare signed fields only between runs that share one signing key.

#### Scenario: Snix parity with the baseline

- GIVEN goldens recorded on the pre-change revision with a recorded fixture signing key and environment
- WHEN the conformance rail runs with backend `snix`, the same fixture signing key, and the same environment
- THEN every positive fixture MUST reproduce its golden, including signed PathInfo
- AND every negative fixture MUST fail with its declared stable blocker

#### Scenario: Unrelated failure is not rejection evidence

- GIVEN a negative fixture that fails for a reason other than its declared blocker
- WHEN the rail evaluates the result
- THEN the fixture MUST fail
- AND the rail MUST NOT count it as expected rejection evidence

### Requirement: Store backend evidence keeps local claims

r[mantle.store_backends.claim_boundary] Evidence for backend selection, identity, capability profiles, and conformance MUST be limited to the recorded backend of each tested state directory, the declared profiles and bounds, the fixture signing keys and environments, the tested rejection paths, and the fixture results. It MUST NOT claim store content correctness, durability, crash safety, GC safety, sandboxing, or release eligibility.

#### Scenario: All selection checks pass

- GIVEN the selection, identity, profile, mismatch, and conformance fixtures pass
- WHEN documentation, tasks, or status replies summarize the result
- THEN they MUST name the backends, profiles and bounds, fixture signing keys, and fixtures covered
- AND they MUST NOT claim durability, crash safety, GC safety, or release eligibility
