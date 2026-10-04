# Design: Select the store backend explicitly

## Goal and scope

Every store open names its backend, the state directory records that backend,
and a mismatch fails before any effect. Each backend declares which optional
capabilities it supports. The change admits one backend, `snix`, and keeps its
behavior byte-compatible. `adopt-casita-store-backend` admits `casita` through
the same seam.

## Current behavior

- `StoreHandle::open(StoreConfig)` dispatches to `open_single` or
  `open_overlay` (`crates/crunch-store/src/handle.rs`). `open_single` creates
  the state directory, calls `ensure_store_identity`, and opens the Snix blob
  service (`blobs/`), directory service (`directories.redb`), and PathInfo
  service (`pathinfo.redb`).
- `ensure_store_identity` writes or compares `store-identity.json` with schema
  `mantle-store-state-v1`, logical prefix, and trust policy identity, and fails
  with `store-identity-mismatch` on difference
  (`crates/crunch-store/src/overlay.rs`).
- `--base-store` layers are opened read-only with the same Snix layout.
- Nario v2 import persists a batch through `PathInfoService::put_batch_atomic`
  (`crates/crunch-store/src/nario.rs`). Native store archive import persists
  one path at a time (`crates/crunch-store/src/archive.rs`).
- GC execution rewrites `pathinfo.redb` and `directories.redb` and sweeps
  `blobs/{blobs,chunks}/b3` (`crates/crunch-store/src/gc.rs`).
- Builds sign PathInfo with Ed25519. For `mantle build` the key comes from
  `--signing-key` or is loaded or generated as `signing-key` in the state
  directory, or in `CRUNCH_CONFIG_DIR` when that is set
  (`load_or_generate_signing_keypair` and `config_dir_or`,
  `src/build_cmd.rs:799-913`), so two fresh state directories sign with
  different keys unless one key is provisioned for both. `bootstrap --fetch`
  signs with a key generated for each run (`src/bootstrap.rs:873-874`).
- `src/main.rs` parses the global `--store`, `--store-prefix`, `--state-dir`,
  and `--base-store` options and builds a `RunContext`. Construction of
  `StoreHandle` outside the store shell is limited to declared owners by
  `tools/check_store_capability_boundary.rs`.
- `config/operator-surfaces.ncl` lists every global option for every command.
  `scripts/check-operator-command-contract.sh` checks the generated contract
  artifacts.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Global option and recorded identity | `--store-backend`, identity record, pre-open decision | Selected | Unknown, mismatch, foreign-state, legacy, and forwarding fixtures |
| Declared capability profile | Core capabilities plus declared optional ones, undeclared ones fail closed | Selected | Profile fixtures per backend |
| Require full parity for admission | Every backend implements every Snix behavior | Rejected: blocks a durable backend on unrelated optional features | None |
| Detect backend from directory contents | Infer the engine from files that exist | Rejected: partial or leftover state could select the wrong engine | None |
| Option on store commands only | Add the option to `store *` | Rejected: build, attest, remote, and cache commands also open stores | None |
| Environment variable | `MANTLE_STORE_BACKEND` | Rejected: ambient selection outside the operator contract | Architecture source check |
| Cargo feature | Compile one backend | Rejected: one binary must open either backend | None |
| Constructor default | `StoreConfig::new` keeps defaulting to Snix | Rejected: constructors would bypass the operator's selection | Compile failure on a missing backend |

## Contract and component ownership

- **Backend identifier**: a Mantle-owned type in `crunch-store` with a closed
  set of admitted identifiers. It carries no vendor type, so it can cross
  application ports (`application_architecture.application_owned_ports`).
- **Capability profile**: a static Mantle-owned profile per backend. Core
  capabilities are output admission, lookup, fresh-process reopen, closure
  resolution, export, store archive export and import, in-place signature
  update through `store sign`, PathInfo-backed action-result outputs through
  `ActionResultPort`, GC planning, and plan-bound GC execution.
  `store-repair-final-nar` is listed per backend: `snix` lists it, and a
  backend without it rejects repair with its own blocker before any state
  access. Optional capabilities are `overlay-composition`,
  `atomic-batch-import` with any bound the backend itself imposes,
  `unsigned-admission`, and `rust-unit-cache`. `snix` declares all four and
  imposes no batch bound. The Nario v2 reader's 100,000-record limit is an
  input-format limit that applies to every backend, not a profile bound.
- **Pure open decision**: input is the selected identifier, its profile, the
  requested optional capabilities, the observed identity record, and observed
  persistent-state markers of admitted backends. Output is `bind-new`,
  `reuse`, or a typed rejection. The function reads no files; the store shell
  supplies the observations.
- **Store shell**: observes the identity record and markers, applies the
  decision, and only then creates directories or opens services.
- **CLI composition root**: parses `--store-backend`, applies the `snix`
  default, and passes the identifier through `RunContext`. It selects a
  concrete adapter only and holds no mismatch or profile policy
  (`application_architecture.thin_composition_root`).
- **Child launchers**: the five launchers named in the proposal add
  `--store-backend <id>` next to `--state-dir`.
- **Capability views**: `BuildStore`, `OutputLookup`, `RootRegistry`,
  `ActionResultPort`, `SourceAdmission`, `StoreAdmin`, and
  `TransferObjectStore` keep their signatures.

## Decisions

### Decision: Selection is operator data at composition roots

**Choice:** One global `--store-backend <id>` option. The CLI composition root
defaults it to `snix`. `StoreConfig` requires the identifier, and every
construction site passes it explicitly.

**Rationale:** The operator contract shows the selection. Library code cannot
pick an engine by omission.

### Decision: The state directory records its backend

**Choice:** New state directories get a versioned identity record with a
`backend` field. A legacy `mantle-store-state-v1` record means `snix` and is
not rewritten by ordinary commands. A non-Snix backend never writes the legacy
schema.

**Rationale:** Legacy records are never rewritten, so a directory written by an
older binary and a read-only overlay base keep their identity bytes. The legacy
record cannot be misread as a non-Snix backend.

### Decision: Mismatch fails before effects

**Choice:** The decision runs before `create_dir_all`, lock acquisition,
identity writes, and service opens. A directory without an identity record gets
an asymmetric rule, chosen on 2026-09-30. With `snix` selected, a populated
directory without a `casita` repository marker (`<state-dir>/casita`) is legacy
Snix state: before the Snix services open, Mantle adds a `snix` identity record
and changes no existing file. The Snix services may then update their own redb
files while opening, so no byte-identity claim covers `pathinfo.redb` or
`directories.redb` after the open. At HEAD
`7ec5177718a6950297e04eb4eb957a10b02e23ce`, `ensure_store_identity`
(`crates/crunch-store/src/overlay.rs:156-193`, called from `handle.rs:679-697`)
likewise wrote a v1 record into any directory without one. A directory without
an identity record that holds the `casita` marker is rejected whichever backend
is selected. With `casita` selected, any entry other than
`store-mutation.lock`, `signing-key`, `casita-trusted-public-keys`, and
`overlay-trusted-public-keys` blocks binding. Mantle never infers `casita` from
file markers, and no operator procedure copies state or runs an older binary.

**Rationale:** A rejected open cannot leave partial state for the wrong engine.
Refusing identity-less Snix state would block `store archive export`, the only
migration path out of an existing Snix store. The `casita` marker is the one
piece of evidence Mantle can use to refuse. The rule has a risk. With `snix`
selected, unrelated content without a `casita` marker is claimed as Snix state,
as it was at HEAD. The store overlay policy only adds `mantle-store-state-v2`
to its allowed schemas, and the trust policy identity is unchanged, so v1
records written at HEAD still pass the prefix and trust checks. The cutover
landed in `preflight_store_identity` after the Casita change's Run 27. Its
first fixture run (Run 6 in this change's evidence) failed only because the
positive fixture compared the Snix redb files after the Snix services reopened
them: 18 header bytes of both `pathinfo.redb` and `directories.redb` changed,
and every other member stayed byte-identical. Run 9 passes the revised
fixtures.

### Decision: Backends declare optional capabilities

**Choice:** Every backend implements the core capabilities, including in-place
signature update through `store sign` and PathInfo-backed `ActionResultPort`
outputs. Final-NAR repair is listed per backend: a profile's core list names
`store-repair-final-nar` only when the backend implements it, and a backend
without it rejects `store repair-final-nar` and the library repair calls with
its own blocker before any state access or effect. Optional capabilities
(`overlay-composition`, `atomic-batch-import` with any bound the backend
imposes, `unsigned-admission`, and `rust-unit-cache`) are declared in the
backend's profile. A request for an undeclared one fails with a
backend-specific blocker before any state access, a request beyond a declared
bound fails before any state mutation, and `store info` and the documentation
list the profile and bounds.

**Rationale:** A backend can be admitted without full parity, and the missing
behavior is visible and fails closed instead of degrading silently. The parent
moved repair out of the shared core on 2026-09-30: a Casita repair cannot yet
keep the replaced envelope and the repair's artifact-attestation sidecar
recoverable together, so `casita` omits `store-repair-final-nar` until a
durable repair fence exists. `store info` keeps its existing core list, which
then omits the entry.

### Decision: No fallback across local backends

**Choice:** Lookup, closure, export, GC, repair, and inspection use only the
selected backend and same-backend overlay bases. Configured remote
substitution keeps its current policy, and substituted outputs are admitted
into the selected backend. Content moves between local backends only through
explicit operator commands.

**Rationale:** A miss or corruption report stays truthful for the selected
backend.

### Decision: One conformance rail for every backend

**Choice:** A backend-parameterized fixture set covers the core capabilities,
stale-plan rejection, and identity checks for every backend. It runs each
optional capability's fixtures where declared and the fail-closed fixture
where not. Every run uses an explicitly provisioned fixture signing key and a
recorded environment. Under `snix`, signed PathInfo, store paths, NARs, and
other deterministic facts match their first preserved historical captures.
The original T1.1 two-path fixture at the identical physical root reproduces
its exact numerical prechange GC plan ID. The supplemental first-7ec
three-path fixture compares full canonical GC consumer facts by sorting only
historical blob-index/blob-chunk observations within their categories;
the first raw and selected canonical plan IDs remain unequal evidence, never
an equality assertion or a favorable replacement capture. Signed fields are
compared only between runs that share one signing key; runs with different
keys compare unsigned fields and verify each signature under its own key.

**Rationale:** Ed25519 signatures depend on the signing key, so a signed
golden is reproducible only under the key that produced it. The dependent
backend change proves parity against the same fixtures instead of a new ad hoc
suite.

### Decision: Declared capabilities can carry bounds

**Choice:** A profile may declare an optional capability with a numeric bound
that the engine imposes, such as the Casita per-commit root limit for
`atomic-batch-import`. The profile and `store info` report the bound, and a
request beyond it fails closed before any state mutation. Input-format
limits, such as the Nario v2 reader's 100,000-record limit, apply to every
backend and are not profile bounds, so `snix` declares no batch bound.

**Rationale:** An engine limit is part of the capability. Splitting a request
to fit would silently weaken it.

## Failure behavior and ordering

Order: parse the option, check requested optional capabilities against the
profile, observe identity and markers, decide, then create state or open
services. Overlay composition decides for every layer before any base read.

Stable blockers:

- `store-backend-unknown`: the identifier is not admitted. No state is read or
  created.
- `store-backend-mismatch`: the recorded backend differs from the selection;
  a directory without an identity record holds the `casita` marker; `casita`
  is selected and a directory without an identity record holds any entry
  besides the lock, signing key, and trust-policy files; or an overlay layer
  records a different backend. The report names the selected and observed
  backends when both are known. No file changes.
- A backend-specific blocker for each undeclared optional capability, defined
  by the change that admits the backend.

## Tests

- Positive: default and explicit `snix` match the original two-path
  fixed-root exact numerical GC golden and signed PathInfo under the
  recorded fixture signing key and environment. The first preserved
  three-path 7ec capture matches exact signed/NAR/path facts and
  canonical GC consumer facts while retaining unequal raw plan IDs;
  new state records `snix`, legacy identity opens unchanged, all-`snix`
  overlay layers compose, and `snix` overlay, atomic batch, unsigned
  admission and Rust-cache profile fixtures pass. Each child launcher
  forwards the identifier (argument-capture fixture).
- Signing-key comparisons: two `snix` state directories with one provisioned
  signing key produce equal signatures; with different keys they produce equal
  unsigned fields and signatures that verify under their own keys. The
  two-backend form of these comparisons runs in `adopt-casita-store-backend`
  once a second backend exists.
- Negative: unknown identifier (`tape`); recorded mismatch using a fixture
  identity record for another backend; foreign markers without an identity
  record; mixed overlay layers; an undeclared optional capability, a batch one
  past a declared bound, and Rust unit cache use, each under a test-only
  profile; a launcher that drops the identifier; a `StoreConfig`
  without a backend fails to compile; an environment variable does not change
  the selection.
- Architecture: the store capability boundary checker still passes, and the
  identifier and profile types contain no vendor type.

## Lifecycle and archive

**Choice:** T4.4 is a pre-archive readiness milestone only. Once T3.1
and every prior implementation task is complete and the implementation
and revised change are committed, preview and sync accepted specs with
the explicit pinned Cairn policy; capture the sync mutation and gate
receipts before checking T4.4. Then, as a separate **required**
isolated-branch lifecycle step outside the pre-archive checklist,
preview and execute the named `cairn archive`, validate under the same
policy afterward, and append both the archive and post-archive receipts
to retained evidence. Until the actual archive and post-archive
validation succeed, do not report the change as archived.

**Rationale:** Cairn refuses to archive a change with any unchecked
task. Defining a task as its own archive operation creates an
impossible prerequisite; treating its *pre-archive* sync/readiness as
the task instead resolves that circular gate without weakening the
separately required actual archive, validation, or evidence.

## Risks / Trade-offs

- A new global option changes every command entry in the operator contract.
  The generated artifacts change in one reviewed diff.
- The constructor cutover touches about 30 files that
  `thin-cli-composition-root`, `resolve-content-addressed-inputs-before-dispatch`,
  `adopt-fault-injection-io-tests`, and `add-retention-interest-records` also
  touch. Land on current `origin/main` and rebase in order.
- The legacy interpretation "v1 means `snix`" is permanent.
- Remote worker state directories under `remote-workers/` receive the
  identifier explicitly from their launcher.
- Profiles let a backend omit optional behavior. Each omission must be listed
  and tested as a fail-closed path.
- Signed goldens depend on the fixture signing key. Losing that key means
  recording new goldens on the pre-change revision.

## Claim boundary

Selection, identity, and profile evidence proves which backend a state
directory records, which optional capabilities each backend declares, and that
the tested opens reject a mismatch or an undeclared capability without
mutation. Signature equality holds only for the fixture signing keys and
environments used. It does not prove store content correctness, durability,
crash safety, GC safety, sandboxing, or release eligibility.
