# Design: second-wave functional-core validation

## Context

`crunch-shell-core` and `crunch-release-core` already prove the FCIS pattern can
work beyond the first no-std pilot pair:

- `crunch-shell-core` owns shell sidecar JSON validation plus activation
  env/path/hook planning over owned UTF-8 data
- `crunch-shell` keeps `PathBuf`, `OsString`, `split_paths(...)`, non-UTF-8
  rejection, and `ExecTarget` reconstruction for `src/shell_cmd.rs`
- `crunch-release-core` owns canonical release-evidence manifest validation,
  bundle-member/path checks, proof-linkage validation, and full-proof identity
  parsing over owned bytes/strings
- `src/release_evidence.rs` and `src/release_cmd.rs` keep file copying,
  directory hashing, manifest I/O, proof-bundle loading, and CLI formatting in
  the std shell

The problem is not missing second-wave code. The problem is missing second-wave
spec and proof inventory. The current main specs and validation runner still
speak as if only the first-wave attestation/project pair exists.

That creates a brittle state where shell/release no-std work is real but not
part of the declared boundary. A later regression could land in those crates
without tripping the documented proof path.

A second retroactive wrinkle is API-shape enforcement. `crunch-shell-core` and
`crunch-release-core` already carried a few public borrowed/reference helpers
from their pre-spec landing. Once the widened checker makes owned-data core
surfaces normative, those helpers stop being mere implementation detail and
become validation failures.

## Goals / Non-Goals

**Goals:**

- extend the no-std core inventory from the first-wave pair to the current
  adopted four-core set
- describe the std shell/adaptor ownership for shell activation and release
  evidence in the same explicit way as the first-wave domains
- define deterministic validation for already-landed second-wave code
- make the no-std runner/checkers derive second-wave scope from a checked-in
  inventory instead of scattered first-wave-only constants
- require exact shell/release verification commands and ownership-review rules

**Non-Goals:**

- perform broad semantic redesign of the second-wave core APIs during this
  validation change; only minimal checker-driven boundary normalization that
  preserves behavior and shell ownership is in scope
- claim shell or release domains are now historical parts of the archived
  first-wave change
- broaden the no-std wave to build/store/eval/runtime crates here
- replace existing first-wave strict shim rules with a looser second-wave model

## Decisions

### 1. Treat shell activation and release evidence as explicit second-wave domains

**Choice:** the main specs will explicitly adopt `crunch-shell-core` and
`crunch-release-core` as second-wave no-std domains rather than leaving them as
out-of-band implementation details.

**Rationale:** the code already exists and already follows the FCIS boundary.
Keeping it outside the declared inventory only preserves drift.

**Implementation:** modify `functional-core`, `architecture`, and `portability`
so they name the second-wave cores, their std shell/adaptor owners, and the
exact proof commands.

### 2. Drive validation from one checked-in adopted-core inventory

**Choice:** later implementation MUST keep one checked-in adopted-core
inventory under `openspec/specs/functional-core/validation/` and make the
runner/checkers consume that one source of truth instead of hard-coding
first-wave-only package/file/export lists in multiple places.

**Rationale:** this change is retroactive. The shell/release code landed before
this OpenSpec change. A single inventory gives the runner, scope checker,
API-shape checker, dependency checker, and ownership review one source of truth
for the adopted core set.

**Implementation:** the inventory must name each adopted core crate, required
exports, and std adapter files only. The exact validation commands stay in the
spec requirement text, and dependency allowlist entries stay in
`openspec/specs/functional-core/validation/deps-allowlist.txt`. The scripts may
join those sources at runtime, but `scripts/no_std_core_checks.py` must not
maintain a separate mirrored inventory that can drift from the checked-in
source. The purity checker must derive its watched core source directories from
that same adopted-core inventory so `crunch-shell-core` and `crunch-release-core`
cannot be skipped while scope/API/ownership checks are widened.

### 3. Keep first-wave strict shim rules and add second-wave adapter-only review

**Choice:** first-wave legacy std shim files stay under the existing strict
re-export-only rule, while second-wave std adapter files use explicit
`adapter-only` ownership review instead of pretending they should collapse into
empty shims.

**Rationale:** `crunch-shell` and `src/release_evidence.rs` legitimately own
translation logic, path decoding, file copying, hashing, and CLI formatting.
Those are shell concerns, not violations. The right rule is to classify and
review them as adapter-only code, not to ban all functions in those files.

**Implementation:** `openspec/specs/functional-core/evidence/ownership-review.md`
should keep first-wave historical classifications and add explicit second-wave
adapter files such as `crates/crunch-shell/src/{lib.rs,adapter.rs,types.rs}` and
`src/{release_evidence.rs,release_cmd.rs}`. Each listed second-wave std adapter
file must be classified as `adapter-only` or `unrelated`, and the artifact must
record an explicit review verdict that shell/release business logic remains in
`crunch-shell-core` and `crunch-release-core`. The checker should fail if those
listed files are missing from review, use the wrong classification vocabulary,
or allow second-wave business logic to drift back into std files.

### 4. Extend the runner with exact shell/release proof commands

**Choice:** `functional.core.nostd.boundary.continuously.verified` will require
host+wasm checks for `crunch-shell-core` and `crunch-release-core`, plus exact
shell/release boundary commands and feature-level dependency audit.

**Rationale:** same-family review on this repo is literal. If the spec does not
name the exact commands and feature-failure semantics, the tasks and evidence
drift quickly.

**Implementation:** the modified requirement should name:

- host and `wasm32-unknown-unknown` checks for both second-wave core crates
- `cargo test -p crunch-shell adapter_preserves_path_order_and_appends_bin`
- `cargo test -p crunch-shell non_utf8_with_path_is_rejected`
- `cargo test -p crunch --bin crunch create_and_verify_release_bundle_round_trip`
- `cargo test -p crunch --bin crunch load_full_self_hosting_proof_identity_rejects_prerequisite_only_artifact`
- `cargo test -p crunch --test release_cli release_verify_rejects_manifest_schema_mismatch`
- `cargo test -p crunch --test release_cli release_verify_rejects_missing_workflow_provenance`
- `cargo test -p crunch --test release_cli release_verify_rejects_claim_boundary_violation`
- `cargo test -p crunch --test release_cli release_verify_rejects_proof_linkage_source_digest_mismatch`
- `cargo tree -e features` or equivalent metadata inspection so the dependency
  checker can fail when an allowlisted crate enables `std` or a default feature
  set that requires `std`

The umbrella runner remains the place that enforces the wasm prerequisite. It
must check the active rustup-managed toolchain first, run
`rustup target add wasm32-unknown-unknown` when the target is missing, and fail
with a clear prerequisite error before any second-wave checks start when the
target cannot be provided.

### 5. Allow minimal boundary-normalizing API reshapes when checker coverage exposes borrowed surfaces

**Choice:** this change may apply narrow public API normalization to
`crunch-shell-core` and `crunch-release-core` when widened API-shape
validation rejects existing borrowed/reference-based surfaces, but it must not
change domain semantics or move shell responsibilities across the boundary.

**Rationale:** the second-wave cores landed before this validation change.
Once `functional.core.apis.plain.data.typed.results` and the API-shape checker
become normative for those crates, public `&str`, `&[u8]`, `&self`, or similar
borrowed surfaces become proof failures. Narrow owned-data normalization keeps
spec, checker, and implementation aligned without treating shell/release as a
new semantic redesign project.

**Implementation:** allow minimal shifts such as owned-input free functions,
owned request structs, or visibility tightening for borrowed helper methods,
while preserving the same std-shell/core split, the same user-visible shell and
release CLI behavior, and the same core business logic ownership.

### 6. Retroactive validation must not depend only on new-change git history

**Choice:** second-wave validation must not rely only on files touched by this
new OpenSpec change, because the shell/release code already landed before this
change existed.

**Rationale:** a history-derived review model worked for first wave because the
spec and code moved together. That assumption is false for second wave.

**Implementation:** the ownership proof must derive first-wave touched std files
from the union of git history for the active path
`openspec/changes/no-std-functional-core/` and the archived path
`openspec/changes/archive/*-no-std-functional-core/`, when present, through
`HEAD`. The ownership proof and validation inventory must also treat the
second-wave adapter/core files as explicit reviewed scope even when the change
that adds validation rails edits only scripts/specs/evidence.

## Interaction Sequence

### Shell activation flow

1. `src/shell_cmd.rs` reads `.crunch-shell.json`, snapshots the host env, and
   validates `--with` paths on the std side.
2. `crunch-shell` converts `PathBuf` and `PATH` state into owned UTF-8 strings,
   rejects non-UTF-8 paths, and chooses `ExecTarget` from `ExecMode`.
3. `crunch-shell-core` validates sidecar structure and computes env/path/hook
   planning over owned data only.
4. `src/shell_cmd.rs` runs hooks and execs the final target.

### Release evidence flow

1. `src/release_cmd.rs` parses CLI input and calls `src/release_evidence.rs`.
2. `src/release_evidence.rs` reads proof bundles, copies files/directories,
   hashes trees, and writes/reads `manifest.json` on the std side.
3. `crunch-release-core` validates canonical manifest structure, bundle member
   paths, proof linkage, and full-proof identity bytes over owned data only.
4. `src/release_cmd.rs` formats success/failure output for the operator.

## Verification

- `openspec validate second-wave-functional-core-validation`
- `openspec_gate stage=proposal change=second-wave-functional-core-validation`
- `openspec_gate stage=design change=second-wave-functional-core-validation`
- `openspec_gate stage=tasks change=second-wave-functional-core-validation`
- later implementation proof must extend `./scripts/check-no-std-core.sh` so it
  runs the exact second-wave host+wasm and shell/release boundary commands named
  in the modified `functional-core` requirement

## Risks / Trade-offs

**Inventory drift** → Mitigation: keep the adopted-core inventory in one
checked-in place and make the runner/checkers consume it.

**Confusing first-wave shims with second-wave adapters** → Mitigation: keep the
strict first-wave legacy shim rule separate from explicit second-wave
`adapter-only` review.

**False green from partial rollout** → Mitigation: require the runner, allowlist,
scope/API-shape checks, ownership review, and exact command list to land in one
implementation slice.
