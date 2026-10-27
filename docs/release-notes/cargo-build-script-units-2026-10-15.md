# Cargo unit-plan build scripts: bounded signed-client acceptance

Cargo unit plans execute build scripts as declared sandbox units with typed
`out`/`flags` outputs, declared compiler target facts and native input roles.
The supported directive vocabulary and fail-closed limits are documented in
`docs/operator-workflows.md`; ambient host discovery is not a native-input API.

The retained 2026-10-14/15 signed-client fixture executes proc-macro2,
serde-derive, libc and a native-sys static-link application and observes 42.
Changing only its declared Mantle-built prebuilt-C provider records exactly
31 independent positive cache-admission events and 3 affected actual builds,
then executes app42 with trusted physical verification. Nominal Cargo unit
labels remain stable; actual input-addressed derivation/output paths change.
See the T4.1/T4.2 exact-byte manifests and receipts under
`.cairn/changes/run-cargo-build-scripts-as-plan-units/evidence/`.

At this retained fixture's source subject, the matrix distinguished its narrow
`declared-native-static-link` surface from the broader native group: pkg-config
execution, C compilation inside a build script, and links metadata consumption
are not established by a prebuilt static archive. Reconciliation also retains
the independent reviewed links/pkg-config replay against Mantle-built libbz2;
the current `native-link-metadata` row is scoped to that declared prebuilt
provider, not C compilation inside `build.rs`. This does not broaden the
original app42 receipts or validate the reconciled source. Unexercised offline
Cargo/no-oracle surfaces do not cite the retired scanner or fake-Cargo output
as compilation/blocker evidence.

Supported `unit_plan` rows without a named fixture now fail the production
Nickel row contract with the row ID in the error. The drift rail mutates the
real matrix and exercises that boundary, rather than only checking that its
current fixture list happens to be nonempty. The helper fixture also links
and executes a real consumer whose generated source, cfgs and script-provided
environment combine to yield 84; library-file existence alone is insufficient.

These claims do not establish full Cargo compatibility, pkg-config discovery,
compiler correctness, global current-source reproducibility, baseline/archive
completion, bootstrap correctness or release eligibility. Ordinary tests do
not replay the preserved heavyweight signed-client proof.

## Current-source acceptance workflow

`scripts/prove-cargo-unit-acceptance.py` executes a new-directory-only campaign
from an explicitly pinned request. Its module help specifies the request fields
and signed-input prerequisites. It runs cold/warm, package-edit, unrelated-edit,
native-provider-change, actual offline-Cargo app comparison, sandbox/directive
negatives, signed publication and clean-client reuse. It retains exact commands,
exit/stdout/stderr, native plans, package slices, per-kind counts and observed
derivation identities. Changed execution sets must equal the affected package
units and their reverse dependencies; a reused label alone is not evidence.
The input cache must contain source/tool/vendor/config inputs, not earlier unit
results. The outside host/network controls must be reachable using the same
admitted probe executable used by the corresponding negative goal.

The current `fixtures/cargo-build-script-units` source additionally requires
declared `CC` and `AR` paths as native roles for `native-answer-sys`, alongside
`NATIVE_ANSWER_ROOT`. Its script compiles `native-zero.c`, archives it in
`OUT_DIR`, generates Rust source, emits ordered link directives and exports
`links` metadata consumed by the app's own build script. These source changes
are inputs to a new campaign, not an extension of the historical receipts.
It is now a virtual resolver-2 workspace with separate `app/` and
`native-answer-sys/` roots. Root-only edits are outside both package slices;
the app includes text and byte data from its own package. This permits a real
outside-package control without treating a root app's README as unrelated.
The registry versions remain fixed by this fixture's checked-in lockfile.

The producer now retains exact `cargo-metadata.json`, `cargo-unit-graph.json`,
`rust-plan-receipt.json` and `cargo-unit-bindings.json` alongside its summary.
The binding receipt maps actual wire unit IDs to Cargo package, feature, target,
host/target execution, source-slice and triple facts; auxiliary target-facts
units have no Cargo package and are not included in that mapping.

Nested package roots from Cargo metadata are omitted from parent slices,
including inactive nested members. In-package `include_str!`/`include_bytes!`
data remains part of its consumer's source identity. Undeclared external source
reads are not repaired by exposing the checkout: they fail at compilation or
script execution. No general external-source declaration syntax is added.

For bounded native consumers, optional `target_linker` configuration contains
`executable`, ordered `arguments` and whole-root `inputs`. Its exact JSON must
match the evaluated `MANTLE_UNIT_TARGET_LINKER` declaration. It applies only to
target compilation effects, never host build scripts or proc macros, even when
triples coincide. Paths in linker arguments must name declared inputs; response
files and ambient host paths are rejected. `cdylib` artifacts use the existing
dependency manifest. Cross-target compatibility requires a real pinned compiler
with the target standard library and separate sandbox evidence; neither the
configuration seam nor a host cdylib test establishes Android or universal cross
compilation support.

## Execution compiler boundary correction (2026-10-27)

Current-source lowering separates producer Cargo/oracle inputs from execution
compiler/runtime inputs. A mixed compiler/Cargo store root is indivisible and
now rejects instead of becoming a script execution input; removing Cargo from
PATH alone is not sufficient. The helper's compiler PATH comes from declared
`MANTLE_UNIT_RUSTC`, replacing `MANTLE_UNIT_TOOLCHAIN` with no compatibility alias.
The locked flake exposes `cargo-unit-compiler`, built from the same nightly's
independent compiler and standard-library components using rust-overlay's own
sysroot/RPATH construction. See operator workflows and ADR 0134 for explicit
adoption and transitive-closure review requirements.

This correction does not relabel historical receipts. The retained direct
current-helper bwrap observation found both PATH and absolute Cargo execution
successful with the mixed root; the separate Mantle import probe timed out
before script execution. Neither is a passing no-Cargo pipeline proof.

Fresh scoped verification is now retained in
[`compiler-only-boundary-2026-10-27.json`](../../.cairn/changes/run-cargo-build-scripts-as-plan-units/evidence/compiler-only-boundary-2026-10-27.json)
and
[`frozen-mantle-no-cargo-singleton-2026-10-27.json`](../../.cairn/changes/run-cargo-build-scripts-as-plan-units/evidence/frozen-mantle-no-cargo-singleton-2026-10-27.json).
Pueue 78 executes real compiler cfg, pkg-config and a native consumer returning
`no-cargo-native-answer=42` through the current helper's actual bwrap boundary;
both PATH and absolute Cargo launches fail. Pueue 79 then builds one fresh
signed-input no-substitute script unit through Mantle with verified frozen
CLI/helper copies: 1 built, 0 cached, 0 failed; real compiler cfg succeeds and
both Cargo routes remain unavailable. The earlier shared-CLI drift envelope
stays failed. These are not a full dynamic application/provider replay,
frozen G1/G2 admission, source/snapshot approval, T4.6 acceptance or archive.
