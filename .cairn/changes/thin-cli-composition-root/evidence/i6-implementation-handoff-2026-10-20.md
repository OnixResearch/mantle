# I6 implementation and behavioral acceptance handoff (2026-10-20)

## Status and scope

This is an implementation/acceptance handoff, **not executed verification evidence**.
I6 and V1–V5 remain unchecked. No build, test, formatter, linter, CLI smoke,
architecture check, Cairn gate, or fault campaign was run for this integrated
source while sibling owners were editing it. Historical evidence files retain
their own narrower source identities and outcomes; their passing results do not
apply automatically to this source.

The historical 151-leaf note is not the current inventory. The checked
`config/operator-command-catalog.json` has 154 terminal public descriptor paths
(identity_blake3 `12e74923e80c41b28cede8a16c566e129a9ebce7b1fa3bbffe2a908b876fb7c7`).
Every one is listed below, including later already-implemented families. That
catalog is an inventory input, not proof that effects ran. Current source adds
the separately enumerated public/hidden surfaces below; generated operator
contracts were deliberately not changed by this slice. Modes such as dry-run,
connected fetch, local versus HTTP transport, plan versus realize, source-built
fixed point, and cache selection are branches of their owning operation plans,
not permission to omit those execution paths.

Main owns final integration, immutable BLAKE3 source manifest, final gate runs,
parity output capture, publication and any acceptance/checklist changes. The
source-built owner owns `src/source_built*` and its exact proof-phase observations.
This CLI slice owns root composition, non-source-built command shells, the
application contract and this change's handoff. Existing per-family evidence
under this directory remains the detailed record for source, project, artifact,
attest, release, import, filegen/log, Mantlepkgs, foreign-import, demo, export,
bootstrap/transcript, portable receipt and component boundaries.

## Boundary key: execution authority and independently classified facts

The leaf table references this key. A key is a source location and its actual
operation-selected plan/classifier, not a static capability annotation. Every
leaf selects its own action branch. Error ownership remains in its domain/shell;
contract cores remain no_std and do not contain RunError. A composite authority
covers its documented whole operation and does not imply rollback, durability,
or that every optional write occurred.

| Key | Owning shell / contract | Effects admitted before the selected port | Required observation before terminal success |
| --- | --- | --- | --- |
| A | src/artifact_cmd.rs; contract artifact.rs | Selected source/store/layout authority; export attestation/content/optional receipt; registry input/network/receipt phases | Adapter-owned kind, output identity and measured usage; optional writes are not inferred from a returned call |
| AT | src/attest_cmd.rs; contract attest.rs | plan_attest selects StoreAccess, ReadFiles or composite PublishFiles for the exact AttestOperation | classify_attest binds actual authority, ordered input identity, selected input/byte counts, output authority, result-entry bound and real progress/failure stage |
| B | src/bootstrap_operation.rs, src/bootstrap_effect_port.rs and selected provider modules; contract bootstrap_effects.rs | plan_bootstrap_effects selects concrete BootstrapEffect boundaries before capability calls; filesystem, process, socket, environment, clock and connected fetch authority remain explicit | Provider artifact/readback facts; replay frame/ACK observations; requested versus independently returned report identities; see source-built owner handoff for proof phases |
| BP | src/bootstrap_pin_cmd.rs; contract bootstrap_effects.rs | Check reads declarations, resolves bounded release/cache inputs and publishes a plan; apply validates reviewed input, hashes all sources before writes, then publishes the pin set | Actual parsed release/source inputs and measured transport/hash progress; cached bytes, published plan and applied pin replacements independently read back before report |
| BU | src/build_application.rs, src/build_cmd.rs, src/build_observation.rs; contracts planning.rs, realization.rs | Separate evaluation preparation, admitted local realization or remote dispatch, and evidence publication | Evaluated root/output facts; pipeline target/result accounting; committed report bytes/identity; domain rejection cannot become successful completion |
| P | src/project_cmd.rs; contract project_lifecycle.rs | project_effect_plan selected operation; Inspect; optional Resolve with independent process/network permissions; conditional persistence only when selected | ProjectEffectPort records actual target/calls/items; init has file readback; conditional persistence is explicitly skipped rather than fabricated; resolver outcomes respect admitted cardinality |
| G | src/semantic_graph_cmd.rs; contract graph_query.rs | Selected graph query read and bounded query result | Requested query/selector versus actual result identity, graph result counts and read outcome before rendering |
| E | src/evaluation_cmd.rs, src/evaluator_budget.rs and evaluation-worker root entry; contract evaluation.rs | Selected root evaluation, worker IPC and optional report publication | Actual JSON parseability, admitted source/import confinement and budget facts; worker terminal/teardown records and report readback; native stream terminal records are evaluation evidence, not build success |
| EX | src/nickel_export.rs; contract output_format.rs | Selected Nickel root read/evaluation plus output publication when requested | Validated format and actual rendered/published bytes, including output-device failure before success |
| F | src/filegen_cmd.rs; shared envelope.rs | Plan: input read, optional plan write/readback. Apply: reviewed-plan/input reads, admitted generated writes and state readback | Actual loaded inputs; written plan exact bytes; applied file/state identities match the reviewed plan; blocker exit remains domain-owned |
| FO | src/foreign_import_cmd.rs; shared envelope.rs | foreign_effect_plan selects bounded read/write/process/network authority for the exact action | Real port progress, published plan/receipt bytes and retained realization records; composite-call failure does not claim rollback |
| I | src/pin_import.rs and cargo import shell; shared envelope.rs | Plan: bounded checked workspace reads. Apply: selected no-follow output publication and readback | Actual input usage, admitted workspace identity, output bytes read back beneath selected root; symlink escapes fail before publication |
| L | src/log_cmd.rs; shared envelope.rs | Bounded ReadFiles byte budget including over-limit probe | LogReadPort returns actual consumed bytes and read outcome for empty/list/matched paths; wrong unit/authority/over-budget observations reject |
| M | src/mantlepkgs_cmd.rs; owning mantlepkgs core and shared envelope.rs | Action-selected EffectSession read/write/process/network plan before each capability phase | Measured adapter facts, sealed artifact identities, publication readback and realization/update receipts; selected failure status cannot silently become success |
| N | src/nix_free_demo_cmd.rs; contract nix_free_demo.rs | Exact summary/document reads and optional bounded demo-bundle publication | Read summary identity and independent bundle document count/bytes/readback; diagnostic demonstration is not release eligibility |
| RC | src/portable_receipt.rs; shared envelope.rs | receipt_effect_plan selects clock plus archive/store/filesystem effects for the bundle action | Archive schema/record accounting, imported durable records and requested retained state; consumed bytes and real partial failure retained |
| RF | src/structured_refactor.rs; contract diagnostics.rs | List is pure. Plan/check read bounded root metadata. Apply admits each rename and destination readback before mutation | Actual metadata call counts; no inspection fault can yield a plan; destination inode/size and removed source checked before successful rename report |
| RE | src/release_cmd.rs and selected release proof adapters; contract release.rs | plan_release_operation selects ReleaseEffectOperation and operation-specific expected output/budget | Selected identity and bounded actual published report/receipt facts; transport/reproduction/witness observations remain limited to their declared evidence |
| RM | src/main.rs remote dispatch/replay; src/remote_build.rs, src/remote_credentials.rs and src/remote_service_secrets.rs; contract remote_execution.rs | plan_remote_operation selects the exact operation; subordinate state/clock/random/worker/socket phases are separately admitted before their capabilities | classify_remote_operation checks independently reloaded ticket/revocation/migration state, endpoint/source/job/worker/ticket accounting, rotation readback, listener inode and actual worker outcome |
| IN | src/shell_cmd.rs, src/child_exec.rs and root run/develop dispatch; contracts planning.rs, realization.rs | Admitted shell/root preparation then explicit child execution | Child ExitStatus is independently classified (success, nonzero, signal, spawn error); requested nonzero child cannot become successful CLI output |
| RU | src/rust_plan.rs, root run_rust_cache_command and src/cargo_build_bundle_cmd.rs; contract rust_operations.rs | RustOperation selected before acquisition/capture/execution/publication or cache serving; bundle options select exact trust/read/process/store authorities | classify_rust_operation matches actual captured workspace/plan/bundle/compiler identities, execution receipts/failed units/output artifacts and retained reports; cache observes real listener startup, successfully flushed request count and shutdown |
| SB | src/source_built_fixed_point* and self-build root; contracts rust_operations.rs, bootstrap_effects.rs plus source-built owner contracts | Selected ordinary self-build or source-built fixed-point phase effects before execution | Actual realized roots and bounded phase/proof readbacks; delegated source-built owner controls exact bundle entry_count and framed replay evidence |
| S | src/source_bundle.rs, src/source_observation.rs; contract source_provenance.rs | plan_source_bundle_effects selected SourceBundleEffectAction; network only for connected export; import-store-path uses Import with two declared inputs | Actual bundle format/bytes/record count, connected fetch calls, published manifest, retained import/pin and hydrated tree readback; signed logical/physical root import retains its trust admission |
| ST | src/protected_exec.rs and stage0 root adapter; contract bootstrap_effects.rs | Selected inventory environment/file reads and optional output publication | Generated inventory identity, bounded bytes, closure-risk report and actual output readback before report |
| SO | src/store_cmd.rs; contract store_administration.rs | StorePlan::effects for exact StoreLeaf, including dry-run/execute, HTTP/directory transport and archive stream choices | classify_store_facts checks selected Rows/Roots/Usage/Retention/Gc/Verify/Repair/Sign/Push/Pull/Composition/Archive facts; see detailed store observations below |
| T | src/transcript_cmd.rs and src/bootstrap_operation.rs; contract bootstrap_effects.rs | Selected transcript/input read, isolated working directory, child process and evidence output authority | Actual transcript command exit/expected-output checks and published transcript report readback; transcript eligibility remains separately admitted |
| W | src/wasm_component_cmd.rs; shared envelope.rs | Preparation ReadFiles, pipeline RunProcess, evidence WriteFiles and ReadFiles readback | Published component report exact bytes, requested produced records and real pipeline outcome before terminal report |
| D | src/operator_diagnostics.rs and root doctor dispatch; contract diagnostics.rs | Bounded doctor plan: selected ambient environment/files and explicit SelfBuild child process only for that profile | Observed check statuses/failed count, requested profile/report identity, actual readiness socket result and child exit outcome; failed checks preserve exit 3 |

## Checked public leaf inventory

Each row inherits the exact owner/effect/observation contract in its boundary key;
the operation spelling selects the owning planner branch, not a generic success
wrapper. No ancestor/group-only help node is counted as an executable leaf.

| Operation | Boundary key |
| --- | --- |
| `artifact export` | A |
| `artifact import` | A |
| `artifact oci-export` | A |
| `artifact oci-import` | A |
| `artifact oci-pull` | A |
| `artifact oci-push` | A |
| `attest closure` | AT |
| `attest diff` | AT |
| `attest key-show` | AT |
| `attest policy-init` | AT |
| `attest project` | AT |
| `attest release-show` | AT |
| `attest release-verify` | AT |
| `attest show` | AT |
| `attest verify artifact` | AT |
| `attest verify closure` | AT |
| `attest verify project` | AT |
| `attest witness-create` | AT |
| `attest witness-import` | AT |
| `attest witness-show` | AT |
| `bootstrap capabilities` | B |
| `bootstrap elf-relocate` | B |
| `bootstrap full-source-provider-admit` | B |
| `bootstrap full-source-rust-host-tools` | B |
| `bootstrap native-toolchain-closure` | B |
| `bootstrap parity-report` | B |
| `bootstrap rust-source-provider` | B |
| `bootstrap validate` | B |
| `bootstrap-pin apply` | BP |
| `bootstrap-pin check` | BP |
| `build` | BU |
| `check` | P |
| `dependents` | G |
| `develop` | IN |
| `doctor` | D |
| `eval` | E |
| `export` | EX |
| `filegen apply` | F |
| `filegen plan` | F |
| `foreign-import audit` | FO |
| `foreign-import plan` | FO |
| `foreign-import prepare-sources` | FO |
| `foreign-import produce-aterm` | FO |
| `foreign-import produce-backend` | FO |
| `foreign-import produce-nix` | FO |
| `foreign-import realize` | FO |
| `foreign-import validate` | FO |
| `graph` | G |
| `import cargo` | I |
| `import pins apply` | I |
| `import pins plan` | I |
| `init` | P |
| `list-stale` | P |
| `log` | L |
| `mantlepkgs build` | M |
| `mantlepkgs corpus-verify` | M |
| `mantlepkgs domain-adapt` | M |
| `mantlepkgs domain-compose` | M |
| `mantlepkgs generate` | M |
| `mantlepkgs impact` | M |
| `mantlepkgs plan` | M |
| `mantlepkgs prepare-sources` | M |
| `mantlepkgs publish` | M |
| `mantlepkgs update-advisory-observe` | M |
| `mantlepkgs update-execute` | M |
| `mantlepkgs update-plan` | M |
| `mantlepkgs update-policy-seal` | M |
| `mantlepkgs update-source-observe` | M |
| `mantlepkgs validate` | M |
| `mantlepkgs validation-build` | M |
| `mantlepkgs verify` | M |
| `mantlepkgs version index` | M |
| `mantlepkgs version recheck` | M |
| `mantlepkgs version resolve` | M |
| `nix-free-demo generate` | N |
| `nix-free-demo readme` | N |
| `nix-free-demo validate` | N |
| `receipt bundle export` | RC |
| `receipt bundle import` | RC |
| `receipt bundle list` | RC |
| `receipt bundle verify` | RC |
| `refactor apply` | RF |
| `refactor check` | RF |
| `refactor list` | RF |
| `refactor plan` | RF |
| `refresh` | P |
| `release attest` | RE |
| `release create` | RE |
| `release function-address-bind` | RE |
| `release gauntlet canonicalize` | RE |
| `release gauntlet continuous` | RE |
| `release gauntlet strict-hermeticity-regression` | RE |
| `release global-reproducibility` | RE |
| `release global-reproducibility-evidence` | RE |
| `release nix-witness` | RE |
| `release reproduce` | RE |
| `release transport inspect` | RE |
| `release transport pack` | RE |
| `release transport unpack` | RE |
| `release verify` | RE |
| `release witness-export` | RE |
| `release witness-rebuild` | RE |
| `remote debug gc` | RM |
| `remote debug inspect` | RM |
| `remote debug replay` | RM |
| `remote debug replay-plan` | RM |
| `remote live serve` | RM |
| `remote live subscribe` | RM |
| `remote serve` | RM |
| `remote status` | RM |
| `remote ticket create` | RM |
| `remote ticket inspect` | RM |
| `remote ticket list` | RM |
| `remote ticket migrate-legacy` | RM |
| `remote ticket revoke` | RM |
| `remote ticket rotate-keys` | RM |
| `run` | IN |
| `rust-cache serve` | RU |
| `rust-plan` | RU |
| `self-build` | SB |
| `shell` | IN |
| `show` | P |
| `source bundle bootstrap-profile` | S |
| `source bundle export` | S |
| `source bundle hydrate-self-build` | S |
| `source bundle import` | S |
| `source bundle import-store-path` | S |
| `source bundle list` | S |
| `source bundle plan` | S |
| `source bundle preflight` | S |
| `source bundle refresh-mantle-source` | S |
| `source bundle verify` | S |
| `stage0-inventory` | ST |
| `store archive export` | SO |
| `store archive import` | SO |
| `store archive list` | SO |
| `store composition plan` | SO |
| `store composition realize` | SO |
| `store gc` | SO |
| `store info` | SO |
| `store list` | SO |
| `store pin` | SO |
| `store pull` | SO |
| `store push` | SO |
| `store repair-final-nar` | SO |
| `store roots` | SO |
| `store sign` | SO |
| `store unpin` | SO |
| `store usage` | SO |
| `store verify` | SO |
| `transcript run` | T |
| `upgrade` | P |
| `wasm-component build` | W |
| `why` | G |

## Current source surfaces outside the checked terminal catalog

These rows supplement the checked 154 paths; the preparation row is an execution
mode of self-build rather than an additional command leaf. GW is owned by
`src/nix_gateway/{api_host,native,campaign}.rs` and contract
`gateway_operations.rs`. No checked-catalog regeneration or inventory acceptance
is claimed here.

| Surface | Boundary | Observation / claim boundary |
| --- | --- | --- |
| `bootstrap (default package resolution)` | B | Requested packages versus independently returned store paths; written profile reloaded before completion |
| `bootstrap trust-report` | B / source-built owner | Actual copied bundle receipt entry_count and verified selected identities; verification does not execute a proof |
| `test` | RU | Admitted target/input identity, realized test report identity/schema and outcome-specific exit; cancellation classified before reporting |
| `profile install` | RU | Package admission or exact profile operation; inspection/dry-run invariants, active/retained/pending generation readback under owning locks, or inactive-generation removal preserving active pointer |
| `profile remove` | RU | Package admission or exact profile operation; inspection/dry-run invariants, active/retained/pending generation readback under owning locks, or inactive-generation removal preserving active pointer |
| `profile list` | RU | Package admission or exact profile operation; inspection/dry-run invariants, active/retained/pending generation readback under owning locks, or inactive-generation removal preserving active pointer |
| `profile generations` | RU | Package admission or exact profile operation; inspection/dry-run invariants, active/retained/pending generation readback under owning locks, or inactive-generation removal preserving active pointer |
| `profile switch` | RU | Package admission or exact profile operation; inspection/dry-run invariants, active/retained/pending generation readback under owning locks, or inactive-generation removal preserving active pointer |
| `profile rollback` | RU | Package admission or exact profile operation; inspection/dry-run invariants, active/retained/pending generation readback under owning locks, or inactive-generation removal preserving active pointer |
| `profile remove-generation` | RU | Package admission or exact profile operation; inspection/dry-run invariants, active/retained/pending generation readback under owning locks, or inactive-generation removal preserving active pointer |
| `cargo-build-bundle inspect` | RU | Bounded manifest identity or actual payload/native-plan/signed-receipt validation; replay additionally requires observed execution without planner rerun |
| `cargo-build-bundle verify` | RU | Bounded manifest identity or actual payload/native-plan/signed-receipt validation; replay additionally requires observed execution without planner rerun |
| `cargo-build-bundle export` | RU | Bounded manifest identity or actual payload/native-plan/signed-receipt validation; replay additionally requires observed execution without planner rerun |
| `cargo-build-bundle replay` | RU | Bounded manifest identity or actual payload/native-plan/signed-receipt validation; replay additionally requires observed execution without planner rerun |
| `remote gateway read` | GW | The exact GatewayOperation selects gateway_operations.rs plan and GatewayFacts variant; operation-specific requirements below |
| `remote gateway serve` | GW | The exact GatewayOperation selects gateway_operations.rs plan and GatewayFacts variant; operation-specific requirements below |
| `remote gateway binding` | GW | The exact GatewayOperation selects gateway_operations.rs plan and GatewayFacts variant; operation-specific requirements below |
| `remote gateway call` | GW | The exact GatewayOperation selects gateway_operations.rs plan and GatewayFacts variant; operation-specific requirements below |
| `remote gateway nix-once` | GW | The exact GatewayOperation selects gateway_operations.rs plan and GatewayFacts variant; operation-specific requirements below |
| `remote gateway nix-import-once` | GW | The exact GatewayOperation selects gateway_operations.rs plan and GatewayFacts variant; operation-specific requirements below |
| `remote gateway nix-store-read-once` | GW | The exact GatewayOperation selects gateway_operations.rs plan and GatewayFacts variant; operation-specific requirements below |
| `remote gateway nix-store-read-binding` | GW | The exact GatewayOperation selects gateway_operations.rs plan and GatewayFacts variant; operation-specific requirements below |
| `remote gateway nix-upload-binding` | GW | The exact GatewayOperation selects gateway_operations.rs plan and GatewayFacts variant; operation-specific requirements below |
| `remote gateway provision-quota` | GW | The exact GatewayOperation selects gateway_operations.rs plan and GatewayFacts variant; operation-specific requirements below |
| `remote gateway quota-fault-campaign` | GW | The exact GatewayOperation selects gateway_operations.rs plan and GatewayFacts variant; operation-specific requirements below |
| `self-build --source-built-fixed-point --prepare-source-programs` | SB / parent source authoring owner | Source-only preparation: same independently supplied profile/lineage/native identities and resource bounds; conflicts with dev cache/resume/fast-fail; --out is a new source cohort with source-profile.json, never proof success |

Gateway observations are distinct rather than an already-emitted success wrapper:

- read/serve: actual bound listener inode, selected shutdown and joined worker;
  serving retains owner-authenticated protocol admission. Nix-once instead
  checks actual dispatched/completed/failed build counts, attempt result evidence
  and independently verified retained outputs; a disconnect without a build or
  a swallowed dispatch rejection cannot produce success.
- Nix import once: declared versus actual verified NAR bytes within bound, and
  independently queried retained paths before acknowledgement.
- Nix store read once: actual authorized successful socket writes and response
  bytes through the revalidating writer. Pure store-read binding validates the
  selected path/opcode/generation digest; zero generation is rejected. Upload
  binding checks actual private header bytes and the existing size-bound/digest
  admission, without inventing a transport operation.
- binding: actual bounded request/project/generation tuple digest. Call: actual
  bounded owner response; error envelopes cannot be completed calls.
- quota provisioning: independently reopened ledger and validated snapshot
  schema, preserving existing ledgers. Quota fault campaign: actual fixture
  result/error counts and reconciled entry/charged/reserved ledger totals; it is
  the isolated journal fixture, **not a VM fault campaign**.

Hidden/internal reachable surfaces are not omitted: `__operator-contract`
(raw-descriptors/catalog/reference/workflow/check modes) uses pure descriptor
projection, explicit generated-file check/readback when selected, and fallible
terminal output; `__evaluator-worker` and `__evaluator-worker-fixture` use E;
`__remote-secret-worker` and `remote usage-executor` use RM's actual credential,
worker result and framed protocol facts; `bootstrap source-built-replay` uses
B and the source-built owner's actual framed IPC/ACK observations.
`remote serve --binding metadata` is intentionally pure static metadata, not
socket-service completion. Root source cwd/current-executable, clock, state-dir
environment export and inline-expression staging now have explicit host ports;
the exported environment value is read back and inline staged bytes are compared
with the expression before evaluation. Those context operations do not prove a
subsequent application operation succeeded.

### Rust-plan and build branch observations

The RU key covers capture/PrintOnly and all eight execution modes:
FirstSupported, FirstDependencyChain, TargetTopology, HostArtifactTopology,
Topology, DevDependencyTestTopology, WorkspaceDependencyTopology and
PatchSourceTopology. `src/rust_plan_execution.rs` admits the selected effect
before execution and classifies actual receipt disposition, failed units and
output artifacts before rendering. Blocked or stale receipts retain their
diagnostic content but cannot return execution success. Local-cache/toolchain
opening, shared directory source opening and signing-key reads have their own
boundaries; an explicit workspace path does not trigger an eager cwd fallback.

BU separately observes local output materialization (without following a
logical store symlink), selected run-output consistency, causal-trace persisted
readback and typed interchange persistence. A failed root prevents execution of
another successful root. Evaluation stream run-summary is deferred until those
observations and output publication complete, with owned workers drained even
after output/classification failure. Watch settlement is output-verified;
lifecycle completion requires actual requested shutdown and awaited teardown.
Observe-only evaluation admits no worker process. Self-build legacy and
Cargo-free execution classify binary/smoke/guard/fixed-point facts inside their
own functions; source-only preparation is outside these proof classifiers.

### Store branch observations

The store key is not a generic one-call fact. List/info/selection reject invalid
rows (info also requires the selected record); roots validates its durable
readback; usage checks actual byte-accounting equality; pin/unpin rechecks the
selected retention state; GC compares requested dry-run/execute mode and exact
candidate accounting and rejects incomplete execution/failures; verify rejects
any mismatch; repair compares selected path/mode/mutation and final repair need;
sign requires selected transitions to be trusted afterward; push accounts for
pushed/unsigned/already-present roots; pull compares imported records, actual
payload bytes, retained state and requested closure-root admission; composition
compares requested bindings/decisions and durable identity; archive operations
compare reported records, actual rows/bytes and requested readback. HTTP authority
exists only for HTTP pull, not directory-cache push. Stream/archive format and
selection-only branches remain explicit StoreLeaf choices.

## Behavioral acceptance preparation (not executed)

| Requirement | Prepared behavioral evidence to run on the final immutable source |
| --- | --- |
| V1 valid invocation / human and JSON / exit | Existing family command suites below, plus actual refactor plan/apply smoke in both modes; applied source content, removed old source and absent unrelated state are checked |
| V1 invalid DTO / missing authority | Source-program preparation rejects missing source-built authority and each dev-cache/resume/fast-fail conflict; existing admission suites retain invalid selector/DTO/domain-blocker coverage |
| V1 adapter fault | Refactor root that is a regular file returns exit 3 without a success plan or mutation; absent child executable preserves spawn error; generated-file read failures remain errors; real owner-peer gateway error envelope cannot become a successful call |
| V1 wrong observation / false success | Contract regressions reject wrong authority/unit/identity/budget or missing/duplicate effects; Rust regressions reject missing output artifacts, unequal fixed point, no consumed daemon requests, replay without execution, forbidden Cargo invocation and impossible profile transitions; store regressions reject incomplete accounting/readback |
| V1 rendering failure | All five operator contract modes first reach successful reporting with a normal output device, then return exit 3 rather than panic on /dev/full without state creation; the CLI test owns this transition rather than diagnostic wording |
| V2 architecture positive and negative | Existing scripts/check-cli-architecture.rs rail and --self-test; host-authority matcher includes qualified paths, grouped imports and file-local import/extern aliases, with accepted pure-alias fixtures and rejected authority-alias fixtures; commented/string-only no_std markers cannot satisfy the declaration check |

Permanent tests target consumer-visible boundaries/transitions/errors. No new
wording/source-text/wiring/mock-echo suite establishes operation completion.
Architecture source fixtures are the existing architecture rail, not a second
framework. Removed refactor naming/session wording tests do not substitute for
its real apply/fault behavior. Existing Rust-plan blocked/stale CLI cases now
require non-success exit while retaining receipt/blocker assertions: printing a
blocked receipt is allowed; silently treating it as execution success is not.

### 2026-10-23 reachable corrections and review boundary

These are source changes, not fresh acceptance receipts. `bootstrap-pin` was
absent from the application family's actual match table despite the real CLI
root; both taxonomy directions and its existing contract fixture now include it
under Bootstrap. The lexical architecture rail expands file-local import/extern
aliases before matching authority. ADR 0129 adds a separate compiler gate through
the same Rust entry point and existing no_std Python runner: macro-expanded HIR,
typed HIR and actual package-selected normal runtime dependency edges enforce
cross-module/re-export/macro authority and renamed/transitive dependency cases.
The Rust guard supplies the one authority inventory; there is no second source
parser. Pure crypto/RNG traits remain permitted; ambient entropy is not a trait.

The 44-versus-41 profile discrepancy is **not** evidence for deleting three core
profiles or changing `ROOT_COMMAND_COUNT`. Before final regeneration, the core
and Nickel inventory each declare 44 profiles, while the checked JSON declares
41 and omits exactly `profile`, `test` and `cargo-build-bundle`. The private
`nix-gateway` profile is deliberate server admission, not a public Clap root.
Regenerate the JSON from the reviewed Nickel inventory together with final raw
descriptors/catalog/reference/workflow outputs; generation must not replace
reviewed platform policy with guessed defaults.

Reference, workflow and generated-file-check reports previously used infallible
root print macros. They now use the presentation writer, preserve exact rendered
bytes/newline framing and propagate flush/write errors. The output-fault CLI
regression covers these plus catalog/raw descriptors; its normal-device baseline
prevents an unrelated inventory/admission failure from satisfying the fault case.
The obsolete descriptor-count floor and two PASS-wording/self-test wrapper tests
were removed; generator/contract checks remain explicit commands below.

I6 and V1–V5 stay open until execution. The lexical scan is not substituted for
the compiler gate: both positive/negative fixtures and actual first-party-owner
compiler/dependency checks below must pass on the final source. The compiler gate
uses the current admitted cargo/rustc, records their versions, excludes build/dev
and proc-macro host execution from runtime edges, and compiles no vendored owner
as a new protected layer. Overflow (256 MiB stdout/8 MiB stderr), deadline expiry
(600 seconds per command/3600 per campaign), unsupported/empty compiler records
or ambiguous Cargo package identities are nonpassing. Failure scratch retains
bounded compiler views with source spans and resolved package/alias evidence;
successful scratch is removed. This enforces declared authority on the admitted
configuration, not universal semantic/domain correctness or every feature/target
combination. Root policy ownership remains a source-review responsibility, not
an authority assigned by the compiler. The genuine runtime/parity/service/cold
proof acceptance obligations below are unchanged.


### Exact verification commands for the integrator

Run only after sibling edits and dependency pins are coherent. Choose the final
immutable source manifest first and record its exact BLAKE3 identities with the
captured command outputs. These commands are a handoff, not a claim they passed.

```sh
nix develop --offline --no-write-lock-file -c cargo test --locked --offline -p mantle-application-contract
nix develop --offline --no-write-lock-file -c cargo test --locked --offline -p mantle --bin mantle
nix develop --offline --no-write-lock-file -c cargo test --locked --offline -p mantle --lib user_profile_cmd::tests
nix develop --offline --no-write-lock-file -c cargo test --locked --offline -p crunch-rustc-wrapper run_daemon
nix develop --offline --no-write-lock-file -c cargo test --locked --offline -p mantle --test operator_diagnostics --test rust_plan_cli --test cargo_free_self_build_cli --test remote_credentials_cli --test remote_stdio_cli --test project_cli --test project_refresh_cli --test project_build_smoke --test source_bundle_cli --test source_bundle_hydration_cli --test release_cli --test store_gc_cli --test store_archive_cli --test bootstrap_validate_cli --test bootstrap_parity_cli --test transcript_cli --test kernel_bundle_oci_cli --test kernel_bundle_oci_registry_cli
nix develop --offline --no-write-lock-file -c cargo test --locked --offline -p mantle --test evaluator_budget_cli --test causal_build_trace --test integration --test example_projects
nix develop --offline --no-write-lock-file -c cargo -Zscript scripts/check-cli-architecture.rs --self-test
nix develop --offline --no-write-lock-file -c cargo -Zscript scripts/check-cli-architecture.rs
nix develop --offline --no-write-lock-file -c cargo -Zscript scripts/check-cli-architecture.rs --semantic --self-test
nix develop --offline --no-write-lock-file -c cargo -Zscript scripts/check-cli-architecture.rs --semantic
nix develop --offline --no-write-lock-file -c cargo run --locked --offline -q -p mantle --bin generate-operator-command-contract -- --self-test
nix develop --offline --no-write-lock-file -c bash scripts/check-operator-command-contract.sh
nix develop --offline --no-write-lock-file -c cargo check --locked --offline -p mantle-application-contract --target wasm32-unknown-unknown
nix develop --offline --no-write-lock-file -c cargo fmt --check -p mantle -v
nix develop --offline --no-write-lock-file -c cargo clippy --locked --offline -p mantle-application-contract -p mantle --all-targets --no-deps -- -D warnings
git diff --check
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal thin-cli-composition-root --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design thin-cli-composition-root --root .
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks thin-cli-composition-root --root .
nix flake check -L
```

The full contract command includes bootstrap_effects, diagnostics,
remote_execution, rust_operations and gateway_operations boundary regressions.
The bin command includes expression_scratch, generated_file_check, child_exec,
refactor and gateway actual-peer tests. The parity command includes the existing
build/plan/Rust-plan/remote/store/source/release/project/bootstrap/artifact
surfaces, without replacing an external authenticated service or cold-proof run
with a mock. Preserve V3 outputs in this change's evidence/command-parity.md and
V4 outputs in evidence/architecture-validation.md only when actually observed.
Cold source preparation/proof campaigns and the live authenticated service need
the parent-owned exact source cohort and credentials; they are separate runtime
acceptance, not covered by a successful compile or pure binding command.

Additional direct evaluation smokes (use fresh report destinations):

```sh
target/debug/mantle eval fixtures/evaluation-budget/positive-evaluation.ncl
target/debug/mantle eval fixtures/evaluation-budget/positive-evaluation.ncl --budget-policy config/evaluation/default-policy.json --budget-report /tmp/mantle-i6-strict-report.json
target/debug/mantle eval fixtures/evaluation-budget/positive-evaluation.ncl --budget-policy fixtures/evaluation-budget/policy-observe-only.json --budget-report /tmp/mantle-i6-observe-report.json
```

The parent must compare each persisted budget report to the actual execution
mode and capture its output/exit on the final source; a file's mere existence is
not budget/report correctness. Build/run/shell tests require the existing Linux,
bwrap, store and static-BusyBox prerequisites; report any skipped surface rather
than widening passing test results to an unexercised runtime.

### Direct CLI runtime smoke (after the final binary is built)

Run from the repository root. This does not write repository fixtures. It checks
actual CLI JSON/human behavior, renamed filesystem content, an adapter error,
fallible stdout, and generation/opcode binding; it does not prove authenticated
remote serving or source-built proof eligibility.

```sh
python3 - <<'PY'
import json, os, pathlib, subprocess, tempfile
binary = str(pathlib.Path(os.environ.get("MANTLE_BIN", "target/debug/mantle")).resolve())
with tempfile.TemporaryDirectory(prefix="mantle-i6-smoke-") as scratch:
    root = pathlib.Path(scratch)
    for machine in (False, True):
        work = root / ("json" if machine else "human")
        work.mkdir()
        source = work / "crunch-project.ncl"
        source.write_text('{ identity = "original" }')
        base = [binary, "--state-dir", str(work / "state")]
        if machine:
            base.append("--json")
        for action in ("plan", "apply"):
            result = subprocess.run(base + ["refactor", action,
                "crunch-to-mantle-project-identity", "--root", str(work)], capture_output=True)
            print("refactor", machine, action, result.returncode, result.stdout.decode(), result.stderr.decode())
            assert result.returncode == 0
            if machine:
                report = json.loads(result.stdout)
                assert report["dry_run"] == (action == "plan")
            if action == "plan":
                assert source.read_text() == '{ identity = "original" }'
                assert not (work / "mantle-project.ncl").exists()
        assert not source.exists()
        assert (work / "mantle-project.ncl").read_text() == '{ identity = "original" }'
        assert not (work / "state").exists()
    invalid = root / "not-a-directory"
    invalid.write_text("original")
    result = subprocess.run([binary, "--state-dir", str(root / "state"), "--json",
        "refactor", "plan", "crunch-to-mantle-project-identity", "--root", str(invalid)], capture_output=True)
    print("adapter-fault", result.returncode, result.stdout.decode(), result.stderr.decode())
    assert result.returncode == 3 and result.stdout == b""
    assert invalid.read_text() == "original"
    for mode in ("raw-descriptors", "catalog", "reference", "workflow", "check"):
        args = [binary, "--state-dir", str(root / "state"), "__operator-contract", "--mode", mode]
        baseline = subprocess.run(args, capture_output=True)
        print("report-baseline", mode, baseline.returncode, baseline.stderr.decode())
        assert baseline.returncode == 0
        with open("/dev/full", "wb") as full:
            result = subprocess.run(args, stdout=full, stderr=subprocess.PIPE)
        print("render-fault", mode, result.returncode, result.stderr.decode())
        assert result.returncode == 3 and b"panicked" not in result.stderr
    path = "/nix/store/00000000000000000000000000000000-fixture"
    bindings = []
    for generation, operation in ((1, "is-valid-path"), (2, "is-valid-path"), (1, "query-path-info")):
        result = subprocess.run([binary, "--state-dir", str(root / "state"), "remote", "gateway",
            "nix-store-read-binding", "--path", path, "--operation", operation,
            "--endpoint-generation", str(generation)], capture_output=True)
        print("gateway-binding", generation, operation, result.returncode, result.stdout.decode(), result.stderr.decode())
        assert result.returncode == 0
        bindings.append(json.loads(result.stdout)["request_digest"])
    assert bindings[0] != bindings[1] and bindings[0] != bindings[2]
    result = subprocess.run([binary, "--state-dir", str(root / "state"), "remote", "gateway",
        "nix-store-read-binding", "--path", path, "--operation", "is-valid-path",
        "--endpoint-generation", "0"], capture_output=True)
    print("gateway-zero-generation", result.returncode, result.stdout.decode(), result.stderr.decode())
    assert result.returncode != 0 and result.stdout == b""
    assert not (root / "state").exists()
PY
```

## Remaining integration gates and non-claims

No CLI operation is intentionally deferred by this handoff. The source-only
preparation implementation and cold-proof assembly/admission/readback are owned
by Main, not replaced with a new proof selector or a fake successful operation.
The following remain **unexecuted acceptance gates**, not checked acceptance:

1. Coherent dependency pins and the final immutable integrated BLAKE3 source
   manifest; this worker did not modify Cargo manifests/lock, source admissions,
   generated operator contracts or sibling repositories.
2. Shared compile/no_std, scoped behavioral suites, exact human/JSON/exit parity,
   architecture positive/negative fixtures, strict lint/formatting and all Cairn
   and flake gates on that same source. A previously reported failure was not
   rerun merely to confirm it.
3. Real authenticated remote-service exercise, imported durable-state checks and
   resource-journal fixture results; no observation here promotes the journal
   fixture into a deterministic VM campaign.
4. Parent-owned source-program preparation and cold source-built proof campaigns
   against exact admitted source profiles, lineage/native identities and bounds.
   Publishing a source cohort or verifying a copied receipt is not proof success,
   fixed-point eligibility, build reproducibility or release eligibility.

Only after the parent observes those gates may evidence be recorded, I6/V1–V5
be assessed, or this change be archived. This document records no such outcome.
