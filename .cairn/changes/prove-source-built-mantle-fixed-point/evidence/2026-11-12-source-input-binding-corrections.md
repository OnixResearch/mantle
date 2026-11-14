# Source input binding corrections before the next cold run

## Observed failure (proof v26, remote leviathan)

The v26 cold proof advanced through the Rust stage (5h33m) and then failed
source admission of `bootstrap/source-built/mantle-stage1.json`:

```text
source_built_fixed_point: blocked: source input validation at bootstrap/source-built/mantle-stage1.json: step unit-XXXXX-link.native-entry references unknown input mantle-source
```

## Defects and fixes

1. `source_built_rust_programs_translation::append_link_response` emitted a
   native entry prepare step whose `cwd` word names the authenticated compiler
   source but whose declared inputs did not bind that name. The step now binds
   every input name its own `cwd` word addresses from the values it already
   receives, and fails admission when the name is absent from them.
   Commit `9c04d710a`.
2. Mantle stage authoring attached shared original environment words (for
   example the produced Rust provider) to steps that declared only their own
   operands. `Stage::finish` now binds every name an operation addresses by
   name from `Stage::compiler_inputs()`, so a step declares exactly the
   artifacts its words consume. Commit `16ee6bca2`; unit coverage for the
   referenced-name inventory in commit `0475a09f8`.

## Preparation and preflight evidence

Prepared cohort `prepared-cohort-v20` with the corrected binary:

```text
output_profile_blake3 = 35a3574b2cb3215ae4a87e83e7f9932c417bb7e3c90bdce2932946f1ecc82743
```

An independent scan of every program in the cohort (steps whose operation
addresses a name absent from its declared inputs) reported:

```text
unknown-input refs 0
```

The previous cohort (`prepared-cohort-v19`) reported 2230 references, all
`rust-provider` on script steps in `mantle-stage1` and `mantle-stage2`.

The corrected binary and cohort were transferred as `bin/mantle-v33` and
`profile-v20/`; proof `v27` was launched against them.

## Non-claims

This note records authoring corrections, a prepared cohort identity, and a
static input-binding scan. It is not a fixed-point proof, bootstrap, or release
claim; only the captured `proof-v27` run and its reports can support one.

## Native probe state authority (2026-11-13)

The existing Leviathan proof v27 exited 1 at recipe admission:

```text
source recipe: guarded outputs require explicit phase observation or a dominating command guard
```

Its failed attempt is retained at
`/home/brittonr/sbfp-20261109/.proof-v27.source-built-fixed-point-staging-2353882`.
A targeted scan of prepared cohort v20 found 53 rejected edges in each Mantle
stage: 37 driver-probe result consumers and 16 skipped-assembler observations.
Both read ordinary file trees from commands admitted to return a nonzero status.
The earlier StageX, native-provider and Rust-provider recipes had no such edges
in that scan; this is not complete recipe or action-trust verification.

`append_native_request` now forwards the prior admitted `cc-work` state for
source-selected driver rejections and for the skipped-assembler branch after
a compiler capability rejection. Actual command stdout, stderr and status
remain unchanged. Successful archive/assembler state still comes from its
actual producer. Guard, path, source-pin and resource admission were not relaxed.

Pueue 28 exercised the positive/negative graph-admission regression:

```text
test source_built_mantle_programs::native::tests::rejected_native_probes_retain_admitted_state_not_optional_outputs ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2949 filtered out; finished in 0.00s
```

The nightly leaf formatter passed in Pueue 27. The regression used actual
stable Rust 1.96.0 (`ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96`) with
`RUSTC_BOOTSTRAP=1`, the existing clang/mold environment, and the captured
vendor inputs. Locked package identities and registry checksums match the
current lock; the Mantle package's dependency list differs from the older
frozen source. The checkout's ignored `vendor-deps/cc/src/target` is missing,
so that checkout snapshot was not repaired or substituted into proof authority.

Raw commands and output remain in
`/home/brittonr/mantle-guard-check-20261113.6nAonq/` (`verify.sh`,
`checks.log`, `checks-2.log`, `checks-3.log`). Strict Clippy in Pueue 28 did
not pass: its external Cargo subcommand did not retain the global source
override and read the incomplete checkout vendor tree. No release build ran
in that task. This regression and the retained failure are not a prepared
cohort, provider execution, fixed-point proof, or release acceptance.

## Current-tree qualification (2026-11-13)

The corrected Cargo source override was retained by the real external Clippy
subcommand. `verify.sh` then completed focused strict release Clippy
(`cargo clippy --release -p mantle --bin mantle --no-deps -- -D warnings`) and
the release build; the captured output is `checks-4.log` in the run directory
above. The earlier Clippy failure remains a failed attempt, not a passing gate.

Pueue task 27 (`guard-proof-qualification-20261113`) exited 0. Its captured
`qualification.log` records these per-invocation results:

| Focused leg | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| `cargo_free_self_build::tests` | 59 | 0 | 0 |
| `source_toolchain_closure::tests` | 52 | 0 | 0 |
| `self_build::tests` | 172 | 0 | 0 |
| `source_built_fixed_point_action_trust::tests` | 20 | 0 | 0 |
| release-core `determinism::tests` | 22 | 0 | 0 |
| release-core `genuine_rebuild::tests` | 6 | 0 | 0 |
| `release_cli` | 154 | 0 | 0 |
| `bootstrap_parity_cli` | 21 | 0 | 0 |

Substring filters overlap; these counts are not additive. The task also built
and executed `mantle --version` from the release artifact with BLAKE3
`24a193a8b1ebf77aab12cb9fab1868f16ebf71ca6956fe2a170612f55d35ff07`.
That exact retained artifact was copied to the private run directory
`/datapool/mantle-store/sbfp-20261113.berdKH/bin/mantle`, rather than relying on
the target-directory executable that integration builds can replace.

Pueue task 29 (`guard-source-contract-qualification-20261113`) exited 0.
`source-contract-qualification.log` records:

| Focused leg | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| generated Nickel positive/adversarial fixture evaluation | 1 | 0 | 0 |
| `source_built_` | 243 | 0 | 27 |
| `source_bundle::tests` | 96 | 0 | 0 |
| `crunch-source-core --lib` | 1 | 0 | 0 |

The 27 retained-input/protected-execution smoke tests were ignored, not run.
The source-built leg includes the native probe-state regression and plan,
action-trust, reconciliation, receipt, strict-policy, resource and publication
rejection tests. These are focused contract checks, not a real cold proof.

The machine-contract gate also exposed stale producer classification and
`crunch-build-report-v1` schema coverage. The schema now covers the existing
optional `finish_gates`, `plan_output_bindings`, `pool_goal_observations`, and
native-plan `source_slices`. The original base fixture remains compatible; a
populated fixture and four adversarial cases exercise the added projections.
Generated contracts and freshness were regenerated through the repository
checker. `machine-contracts-regenerated.log` records
`machine schema contract check: PASS (24 contracted, 65 classified)`; the
Nickel fixture-evaluation result above independently exercises the predicates.

Maintenance owners remain the native source-program author, `BuildJsonReport`
and its Rust DTO owners, and the repository machine-contract checker. The
regression and schema fixtures are repeatable checks of those boundaries.
No prepared cohort, provider execution, fixed point, release eligibility or
Cairn completion is accepted by these checks.

## Frozen-input review and failed preparation preflight

Captured commands and reports are now retained beside this note under
[`2026-11-13-qualification/`](2026-11-13-qualification/), not only in the host
scratch directory. The installed Cairn tool's read-only validation and proposal,
design and tasks gates each returned exit 0 and `valid: true`; the exact JSON,
stderr and exit records are retained there. These structural gates do not accept
the still-unexecuted fixed point. The Rust checks above used the actual stable
1.96.0/bootstrap environment described earlier, not a claim that the canonical
`nix develop` route was exercised.

Source refresh preserved 65 records, added none, and replaced only Mantle's
record. Its manifest identity is
`de163b8deb0b14afd58469f0c66f8dc62714117b4706323d5e419cf37cd03c6c`; the
replacement Mantle content identity is
`51b3da0e5d3a15f690aec40a41763ff7b656eee03b2e399d97b5d705e6005194`.
The real source-bundle verifier reported `ready` with empty missing, stale,
unsupported and untrusted record lists.

An independent review found the same 66 record identities in the baseline and
refreshed profiles, with only Mantle's projected identity changed. It also decoded
the baseline lock: all 1,060 package identities, all 974 registry/Git
identity/source/checksum tuples, and all non-Mantle dependency lists match the
frozen/current lock. The sole semantic lock change adds the already-present
`async-trait` package to Mantle's root dependencies. The vendor record remains
content-bound to `719e1550…`; this comparison is not an independent walk of every
vendor payload byte. The StageX lineage pin remains `e477ab39…`; `f36d3759…`
remains an independently supplied expected native output, not an identity derived
from this run.

Preparation task 32 did not succeed. It passed source verification, then the real
CLI rejected the unchanged 45,000,000,000-byte preparation budget with only
41,865,969,664 bytes available. No staging/output tree was created. The retained
`preparation.log` is negative preflight evidence, not a prepared cohort. Final
preparation will use Leviathan's observed capacity without lowering that bound.

The independent V1 review also found unexercised receipt, authority-matrix,
runtime and post-commit publication boundaries. Follow-up tests and a directory
sync injection seam are being verified separately. Those `src/` changes are
inside the captured Mantle record even when guarded by `cfg(test)`: the
`de163b8d…` input is therefore a preserved earlier snapshot, not the final-tree
proof input. A final source refresh and fresh program cohort are required after
the integrated checks pass. No current cohort, provider, fixed point or release
is accepted here.

## Complete receipt fixture and source identity correction

The measured synthetic fixture now exercises the real v2 writer and verifier
against a complete proof-root layout. Its execution events are supplied test
facts, not observations of launched providers or protected commands. The
retained `2026-11-13-qualification/v1-receipt-corrections.log` exposed a
production source-binding defect: the writer used the aggregate source-authority
digest for `source_blake3`, but the generic genuine-rebuild validator requires
that digest to name an individual source in the rebuild descriptor.

Plan construction, pure validation, receipt production and verification now
select the actual `MantleSource` record. The separate plan/extension aggregate
still binds all seven roles. The generic validator remains unchanged; the
descriptor still binds each declared source's name, role, kind, digest and size.
An additional rejection case substitutes another genuinely declared source for
Mantle's source, so generic closure membership alone cannot authorize a receipt.
[ADR 0148](../../../../adr/0148-bind-receipt-source-to-mantle-source-root.md)
records the distinction and regeneration requirement.

Local Pueue 26 (`guard-v1-source-binding-20261113`) executed the source-built
contract leg:

```text
test result: ok. 254 passed; 0 failed; 27 ignored; 0 measured; 2680 filtered out; finished in 4.26s
```

The same captured task passed `bootstrap_parity_cli` (21 tests),
`machine_schema_contracts` (4), and `release_cli` (154), then exited 101 in
Clippy: the complete receipt fixture imported a binary-only writer into library
tests, and its linkage filename depended on the shell module. The release-build
leg did not run. The failed transcript is retained as
[`v1-source-binding.log`](2026-11-13-qualification/v1-source-binding.log).

The library test target now includes the real writer using the existing
test-only module convention. Linkage output filenames belong to that writer,
not the shell; no producer, validation, or fixture path is stubbed or skipped.
Dual-target fixture execution, strict Clippy, formatting and the release build
are qualified separately. No fixture or passing contract test proves provider
execution, stage equality, a current frozen cohort, a real fixed point or release
eligibility. I3–I5 and V1–V3 remain unaccepted here.

## Dual-target receipt qualification and retained executable

Pueue 28 (`guard-v1-target-coverage-20261113`) exited 0. Its complete command
transcript is retained in
[`v1-target-coverage.log`](2026-11-13-qualification/v1-target-coverage.log).

| Executed leg | Observed result |
| --- | --- |
| Strict root-package Clippy, binary and test targets, `-D warnings` | Passed |
| Library `source_built_fixed_point_trust_report::receipt_tests::` | 18 passed, 0 failed |
| Binary `source_built_fixed_point_trust_report::receipt_tests::` | 18 passed, 0 failed |
| `cargo fmt --check -p mantle -v`, explicit pinned nightly rustfmt | Passed |
| Release build and real `mantle --version` | Passed; `mantle 0.1.0` |

These receipt filters overlap the earlier source-built leg; counts are not
additive. The retained release executable's BLAKE3 is
`9f82ff2e5e4e92cb62479bbbed437eb8214edca635e63ce7940e34dd16826a4f`.
The actual compiler remains stable 1.96.0 with `RUSTC_BOOTSTRAP=1`, not a claim
that `nix develop` succeeded. The prior failed task remains preserved.

The current machine-contract checker independently reported
`machine schema contract check: PASS (24 contracted, 65 classified)` after the
module cutover. Installed Cairn validation and proposal/design/tasks gates each
returned exit 0 and `valid: true`; exact `final-cairn-*` records are retained
alongside `machine-contracts-final.log`. These are structural results, not
fixed-point completion authority.

The source-plan, writer, verifier and their dual-target fixtures maintain the
repeatable receipt boundary. The earlier `de163b8d…` profile and the old
`24a193a8…` executable are not this final source/executable pair. A fresh source
snapshot, profile refresh, program preparation and independently verified cold
proof on Leviathan are still required. I3–I5 and V1–V3 remain unchecked.

## Frozen source and canonical Leviathan launch

The verified fixes are pushed as `a0f82178c` (machine contracts) and
`b0773d60f2a63e6f21f4627d0d97dcff4b4a9211` (source-built authority and receipt
boundaries). The latter is the frozen code revision for this attempt. Its Git
archive BLAKE3 is
`fd6d174d090d0ad42930597311d78c9968ed62f4318200168cd52ecec98d3d87`;
Leviathan independently checked that archive and the `9f82ff2e…` runner after
transfer. The real transferred executable reports `mantle 0.1.0`.

Exact baseline, host, sandbox-tool, launch and command facts are retained under
[`2026-11-13-leviathan-b0773d60/`](2026-11-13-leviathan-b0773d60/).
The 11,094,548,410-byte baseline transferred with raw BLAKE3 `6254f87f…` is
still the earlier source profile, not the new frozen source. Pueue 1540 now
refreshes only Mantle's record and authors a fresh program cohort on Leviathan.
Its 200 GB / 86,400 second / 16 job policy matches proof v27; the StageX
`e477ab39…` and independently supplied native-output `f36d3759…` pins remain
unchanged. This is not a promotion of a same-run native output into a pin.

Pueue 1541 runs the required current-source checks inside the unchanged pinned
`nix develop` shell, with remote builders disabled and the shell's own immutable
Crane source map. The earlier tooling-only shell probe succeeded in task 1539;
that older snapshot did not qualify the current source. The selected BusyBox
also executed successfully in a minimal namespace root containing only that
executable; this is a tool smoke, not a kernel-isolation or proof claim.

The cached Nix input transport is a bounded one-time operator exception to
missing remote private-Git credentials and the stalled local Nix route. It
preserves exact locked Git objects and NAR identities without transferring
credentials, rewriting URLs/pins, editing SQLite records, or bypassing GC/cache
policy. It is not a new dependency-fetch service. The Mantle operator owns this
run-local cache; a changed lock or evicted input triggers fresh admission, not
silent reuse. Its non-claim is tooling availability only.

Both launched jobs and independent acceptance reviews are still pending at this
checkpoint. A reviewed fresh cohort and an actual cold v2 fixed point remain
required; I3–I5 and V1–V3 are not accepted from transfer or launch facts.

## Independent prelaunch review corrections

The independent V1 review mapped every named criterion to behavioral assertions
and found no further substantive coverage gap. Its report does not count the
old `stagex_native_handoff_uses_only_the_fresh_reserved_provider_path` test:
that test inspected Nickel source text rather than executing an authority
transition. The incidental test is removed, not repinned; normalized-provider
admission and mismatched replay-payload rejection retain behavioral coverage.
The changed test source requires a new final source identity before promotion.

The independent prelaunch review also found a real provenance gap: the
`9f82ff2e…` executable was built from a mutable checkout before `b0773d60` was
committed. Transfer equality and `--version` alone do not bind it to that
revision. No mismatch was observed, but it cannot yet authorize a current-source
runner claim. The correction is a separate build from an exact frozen-archive
extraction, with retained pre/post source-tree identities and canonical build
output, before any cold-proof launch. Earlier preparation remains unpromoted.

Both full review reports are retained in `2026-11-13-leviathan-b0773d60/`.
They are review evidence, not completion authority; real current-source checks,
the final prepared cohort, and independently verified v2 execution remain
required.

The incidental-test removal was checked in local Pueue 26
(`behavioral-handoff-coverage-20261113`, a later task than the earlier task 26).
Its exact command, timestamps and output are retained in
`2026-11-13-leviathan-b0773d60/behavioral-handoff-coverage.log`:

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2958 filtered out; finished in 0.00s
```

These tests exercise normalized-provider admission and replay-payload mismatch;
the leaf formatter also passed. This does not add a new proof result or qualify
an old source identity as the changed final tree.

## Source-bound runner for the final revision

The test-only correction and its evidence are pushed as
`72f6d21f290b7be6c885c98a2a3433416b7d03be`. Its exact Git archive transferred
with BLAKE3
`6f5d2b2483b9f09884945d82cc09d7278fb42f5f5294f470b16edb2a84c85629`.
Leviathan Pueue 1542 extracted that archive into a separate build source root,
ran package formatting and a locked, offline release build inside the canonical
pinned `nix develop` shell, and exited 0.

The compiler was `rustc 1.96.0-nightly (c75612477 2026-04-07)`, exact commit
`c756124775121dea0e640652c5ee3c89e3dd0eb4`, for
`x86_64-unknown-linux-gnu`, LLVM 22.1.2. The normalized source-tree tar BLAKE3
was unchanged before and after build:
`d7ec503dbf044f929064aecbdcef6767ccb4b03c21045e721901382656ee56e9`.
The retained runner is
`e7bfd87c096cb569ce6488710ac7e58be5b981e9e6f3d8fd899d68a81ddd9247` and its
actual version output is `mantle 0.1.0`. Full commands and captured output are
retained under
[`2026-11-13-leviathan-72f6d21f/`](2026-11-13-leviathan-72f6d21f/).

Pueue 1543 prepares a separate final-source cohort with that runner. Pueue 1544
checks the affected source-built/receipt tests and strict root-package Clippy
under the canonical shell. The earlier preparation 1540 was explicitly
cancelled as superseded; its transcript and cancellation reason are preserved,
and none of its output is final-cohort authority. The earlier canonical test
task 1541 remains separate supporting evidence for the pre-removal source.

A successful conventional runner build is not a Cargo-free, source-built,
fixed-point or release claim. Final cohort admission and actual cold execution
remain outstanding; I3–I5 and V1–V3 remain unchecked.

## Final-source canonical contract verification

Leviathan Pueue 1544 exited 0 against the exact `72f6d21f` source in the
canonical pinned shell. Its source-built leg records:

```text
test result: ok. 253 passed; 0 failed; 27 ignored; 0 measured; 2680 filtered out; finished in 7.09s
```

The count is one lower than the earlier 254 because the incidental source-text
test was removed. Library receipt tests then recorded:

```text
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 233 filtered out; finished in 0.54s
```

Strict release Clippy for the root package's binary and test targets also
passed with `-D warnings`. The 27 ignored retained-input/protected smokes were
not executed or counted as success. The independent V1 map and the previously
captured focused Cargo-free, closure, self-build, release-core and CLI legs
complete the focused proof-contract verification; none is actual cold execution.

The byte-exact retrieved runner-build transcript has BLAKE3
`a09e7562b100905cc9ebb1ee84a5c0424185929913ac992440b3fd746d265994`;
the final contract transcript has BLAKE3
`ec366512996cf72f4c6ab0f317f6cd9d77421ec7339d5c6457b2ce4f95dc5e71`.
Both match their Leviathan originals. The raw transcripts retain SSH carriage
returns in lossless `.log.gz` files in the final-source evidence directory;
decompression preserves the identities above without rewriting captured output.
The serial Cairn checklist stays unchecked until its
preceding actual provider/stage/receipt obligations are accepted; passing V1
checks alone do not close I3–I5, V2 or release-evidence requirements.

## Canonical CLI, focused compatibility and lifecycle gates

Leviathan Pueue 1541 exited 0 on its frozen `b0773d60` supporting source. Its
complete byte-exact raw log is retained as
`2026-11-13-leviathan-b0773d60/canonical-checks.log.gz`; decompressed BLAKE3
`3312047c57fc8842a2a510373286db1a9c30c9a5e8dbf210b50a47f69a4357fc`
matches the Leviathan original. The source-built and library receipt legs overlap
the final-source 1544 results and are not counted again here.

| Executed canonical leg | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| `release_cli` | 154 | 0 | 0 |
| `bootstrap_parity_cli` | 21 | 0 | 0 |
| `machine_schema_contracts` | 4 | 0 | 0 |
| `cargo_free_self_build::tests` | 59 | 0 | 0 |
| `source_toolchain_closure::tests` | 52 | 0 | 0 |
| `self_build::tests` | 172 | 0 | 0 |
| release-core `determinism::tests` | 22 | 0 | 0 |
| release-core `genuine_rebuild::tests` | 6 | 0 | 0 |

Package formatting, strict root-package Clippy, and the machine-contract checker
also passed. Canonical Cairn validation and proposal/design/tasks gates each
reported `valid: true` and returned success. The pinned Nickel command reported
1.17.0. These are current mechanism/CLI and structural gate observations over
the pre-removal supporting source; final-source task 1544 independently covered
the one test-only source change. They are not a real bootstrap-parity or
release-bundle verification, and V3 remains open for the real proof evidence.

The independent reviewer accepted the corrected runner-to-frozen-archive binding
with no remaining actual runner-provenance correction. Its full response is
retained as `2026-11-13-leviathan-72f6d21f/runner-provenance-review.log`.
It explicitly keeps the final prepared-program cohort and cold proof unaccepted.

## Final-source inert program cohort

Leviathan preparation task 1543 exited 0. Its report binds input manifest
`739b904eb559eadc1d1ea2bf697a819620ce1e0ccb6d3d386eebdeacdb6c9770` and
input plan `a4787e32d83e5c8fa5c911d8e587eba69116c8e6bfbb789873707a4a6c16e17a`
to the prepared output manifest
`47c446b61809bc1f968a065969431af0bb3d7791227e8d18e0bacf95bcad6d77`.
The four native/Rust/Mantle/StageX program-record identities and their inert-only
non-claim are retained in `source-program-preparation.json` in the final-source
evidence directory. This output manifest, not `739b904e…`, was the candidate
cold-proof pin before the independent graph review rejected launch.

The raw prepared profile JSON has BLAKE3
`0b63f4bcacadf36f904559c63793836d2cf17003aad3faeec66f940573139372`;
the raw preparation report has BLAKE3
`ccb08afa9a223386dd3a8f8853068fb39f75c26f2ad43e2d9b3736b95ac1398a`.
These byte checksums are not the semantic manifest identity. Pueue 1545 checked
the retained runner and both files, then the real source-bundle verifier reported
the prepared manifest `ready` with empty missing, stale, unsupported and
untrusted record lists. It exited 0; its exact command and results are retained.

The independent identity review accepted this exact inert profile's record and
payload bindings, with the parent's successful source verifier covering its
canonical manifest check. The independent graph review rejected cold launch:
Mantle-local translation/link references incorrectly named the Rust-provider
stage, and the final Rust-provider export declared output `tree` instead of the
canonical `full-source-rust-provider` required by the unchanged prior-stage
consumer. Both complete reviews are retained alongside the preparation evidence.

No cold proof launched from this candidate. The retained cold-proof script is an
unexecuted planned command, not execution evidence. This inert cohort is
superseded for launch; source/program preparation and qualification must be
repeated from the corrected frozen source before accepting a replacement pin.
No provider, stage, receipt, parity or release acceptance follows from successful
preparation or source availability.

## Owning-stage and canonical-publication correction cohort

Source revision `07ccbf782ca5f60114222b19222ee944bdcd4cba` fixes the two
independent graph blockers without changing the prior-stage admission rule.
Shared translation and macro/link helpers derive produced references from the
owning recipe stage. Each final Rust-provider publication declares the canonical
`full-source-rust-provider` output instead of a generic `tree`. Mantle owns these
source-authoring invariants; the intended durable contribution is correct stage1
and stage2 graph construction, not a provider or compiler correctness claim.

The added regressions exercise local-root admission and cross-stage-root
rejection for all three consuming stages, and canonical versus noncanonical
bound Rust outputs through the actual prior-stage consumer. Their execution is
not claimed at this checkpoint.

The frozen raw tar BLAKE3 is
`6f6223e3c4da94c8338093ba9e158df40baa02db3d2ae82087f6374dffd018c1`;
the transferred gzip BLAKE3 is
`5a4987b7f26f59d698018d764ba154d6214fec92c5524b861f78de1189cb0eeb`.
Leviathan task 1546 is the source-bound qualification/build command; task 1547
depends on its success and performs fresh source refresh, inert preparation and
source verification. Their exact scripts and frozen inputs are retained in
`2026-11-13-leviathan-07ccbf78/`. No replacement profile pin, cold execution,
receipt, parity or release result is accepted until the corresponding results
are inspected.

Task 1546 returned 101 during test compilation: three pre-existing callers of
the extended `stage_binding_fixture` lacked its explicit canonical-output
argument. The full compiler output and terminal Pueue status are retained in
the same evidence directory. No tests executed, no runner was installed, and
task 1547 ended `DependencyFailed` without starting preparation. The three
fixture callers are corrected for the next frozen-source qualification; no
test or production admission rule was weakened.

The complete fixture migration is frozen as
`4e0f7ad0f228773710e57ea8938e1d70d73fdc20`. Its raw tar BLAKE3 is
`88fc87d89af06757004943586df32b910330d745fb57b58b2793c784ec7212b8`;
its transferred gzip BLAKE3 is
`abecc71c89e6a0bba7d7fc19beeb29a062d6f64928ef9b986bf0972739609a7f`.
Task 1548 qualifies/builds this exact source; task 1549 depends on its success
for inert source-program preparation. Their retained commands and input binding
are in `2026-11-13-leviathan-4e0f7ad0/`. These are submission facts, not
successful-check or proof observations.

Task 1548 completed successfully on Leviathan. The captured execution summary is
`test result: ok. 255 passed; 0 failed; 27 ignored; 0 measured; 2680 filtered out; finished in 6.70s`.
The cross-stage macro graph regression and strict canonical Rust-provider
handoff regression executed within that leg. The ignored protected smokes were
not executed. Package formatting, strict root-package release Clippy and the
locked offline release build also succeeded.

The normalized source-tree identity before and after qualification is
`0e14303c729cb55b11f082aab3285526fe52965e0e92a6b151f0c31ca3ef60a6`.
The installed frozen runner's BLAKE3 is
`57042f2b8048afd51e1151749a9d01aa63b99e958bbd151e035c3a62816abadf`.
The exact compressed log is retained as `qualification-4e0f7ad0.log.gz`;
its BLAKE3 `74f281c28b0ad481c354a1cd4bd8f74d069ac8e3a946cee3a7823b21671b5810`
matches the remote compressed log. The remote uncompressed log BLAKE3 is
`b30a3ae9aa95a4dc76a6a8a6036ffb88aabdad5fedf75074b682d6dc84fc7ccb`.
These checks verify the source-authoring mechanism, not a provider or cold
fixed point. Task 1549 prepares the replacement inert cohort; isolated task
1550 covers the required CLI, contract and lifecycle legs without mutating the
frozen proof source. Real parity, release evidence and I3–I5 remain outstanding.

Task 1550 also completed successfully on an isolated extraction of the same
`4e0f7ad0…` archive. Its executed test legs were:

| Canonical leg | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| `release_cli` | 154 | 0 | 0 |
| `bootstrap_parity_cli` | 21 | 0 | 0 |
| `machine_schema_contracts` | 4 | 0 | 0 |
| library `source_built_fixed_point_trust_report::receipt_tests::` | 18 | 0 | 0 |
| `cargo_free_self_build::tests` | 59 | 0 | 0 |
| `source_toolchain_closure::tests` | 52 | 0 | 0 |
| `self_build::tests` | 172 | 0 | 0 |
| release-core `determinism::tests` | 22 | 0 | 0 |
| release-core `genuine_rebuild::tests` | 6 | 0 | 0 |

The machine-schema checker reported `PASS (24 contracted, 65 classified)`.
Pinned Nickel reported 1.17.0. Cairn validation and all three proposal/design/
tasks gates returned success with `valid: true`. The terminal Pueue result and
exact log are retained; compressed log BLAKE3
`02ccf20570cd9928818b358e60c1308fb7170083884356645c5963639851444e`
matches the remote copy, whose uncompressed BLAKE3 is
`da76fe841a885afd0fd24243dc08d09839899bc215ae699e3dcbc44bf1ac3d10`.
This completes current mechanism/CLI and structural qualification, including
V1's focused verification. It does not execute real bootstrap parity or verify
a successful release-evidence bundle; V3's proof-dependent evidence stays open.

## Replacement inert cohort rejected by graph admission

Leviathan task 1549 exited 0. Preparation binds refreshed input manifest
`2d3baa300694dcd1fa9cb0221a368dfca1795371b07ddbae0ed71df20577164d` and
input plan `9be4d9d54ab026ccd7cce90c12c9c826c029f3e9073f10f6255f87a9ef8895e3`
to output manifest
`4932432f6fddd92e0ecefcb96a4aff872e19af3d2e507f8d6112d1c0251a8e51`.
The real source-bundle verifier reported this exact output manifest `ready`,
with empty missing, stale, unsupported and untrusted record lists. The complete
preparation command, terminal result, report, input plan and verifier results
are retained in `2026-11-13-leviathan-4e0f7ad0/`.

| Added inert program record | Declared record BLAKE3 |
| --- | --- |
| `fixed-point-native-source-program` | `aba4f5acd5d8e6318aebe515d13b39b7734bdb69c31b1ed868cedf4bbc98872f` |
| `fixed-point-rust-source-program` | `fffa984ef1db0981dc0689bcf3a61109214570a01d951422fc6f15f10e346abd` |
| `source-built-mantle-programs` | `586da96b7f26cf0137743e5e1f0e07c879efcecde6b847165ae09dc96f37cedb` |
| `source-built-stagex-programs` | `0a211f5effe9e0eaa066467a166467a555f163324465ce5e1a31480d364d1b74` |

The raw prepared-profile JSON BLAKE3 is
`ffcc050da54bb5a05720e768be84e395759f6b0275e72566a86f3bce3189f3ec`;
the raw preparation-report BLAKE3 is
`00aa1dc381cec5251d8f8853a9c44e8548afa1ad6bba11ac199ec9b66b13cd1a`.
Neither byte checksum substitutes for the canonical output manifest identity.
The independent identity review accepted the exact inert profile bindings,
including raw hashes, source/plan linkage, unchanged initial output authority,
and all four program payload/fetch identities. All four record digests changed
across cohort directories because their fetch URLs are part of record identity;
the Rust program bytes also changed for canonical provider outputs. This is not
graph admission. The independent graph review rejected cold launch: both Mantle
recipes retain 740 input references each to nonexistent Rust-stage native-script
products. The LLVM helper's `text` and `assemble` return values still used a
hardcoded stage. The earlier focused regressions verified the corrected
translation/macro helpers but did not cover this nested helper.

Both complete reviews are retained. The cold-proof script was not executed.
The remaining repair binds these LLVM/native-script products to the actual
recipe stage and exercises whole native-script graph admission for Rust and
both Mantle stages, with static and captured environment/cwd inputs. No aliases,
source-pin changes or admission weakening are in scope. No provider, stage,
receipt, parity or release claim follows from the rejected inert cohort.

Current host/tool observations and a real minimal-root BusyBox/bwrap execution
are retained in `current-host-sandbox-preflight.log`. They bind the frozen
runner and exact sandbox tools and report sufficient output-filesystem space
for the selected 200 GB bound. They are prerequisites, not proof execution or
kernel-isolation evidence.

Task 1551 exercised the real shared native-script authoring API after the LLVM
product-constructor repair. It passed
`test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2962 filtered out; finished in 0.03s`.
That regression admits each owning Rust/Mantle stage with both static and
captured environment/cwd bindings through `validate_source_recipe`, then rejects
a cross-stage script-producer substitution. The exact two source-leaf hashes,
compiler identity, command and terminal result are retained in the rejected
cohort's evidence directory. This was a focused mechanism check in the mutable
canonical-check copy, not a new frozen-runner build or replacement cohort.
The current `4932432f…` profile remains rejected and unlaunched.

## Nested native-script corrected source freeze

Revision `4653ddef4edd196620f0a3337b2c7045c8cf4efd` contains the remaining
LLVM/native-script owning-stage repair, its executed graph-admission regression
and the reviewed-cohort rejection evidence. The raw archive BLAKE3 is
`75845ae712c685ca9a9aa2c2120c5f13934551845b65de9b329aec2868fae4a4`;
the transferred gzip BLAKE3 is
`0030d50e5c78a90d325e969eec8848174c06cacbae561f1a3768a905c8792c14`.
Leviathan task 1552 performs source-bound qualification/build. Task 1553 depends
on its success for fresh inert preparation and actual source verification;
isolated task 1554 runs the required CLI/contract/lifecycle legs. Commands and
frozen inputs are retained in `2026-11-13-leviathan-4653ddef/`. Submission is not
successful qualification, a replacement profile admission or a cold proof.

Task 1552 completed successfully. The captured execution summary is
`test result: ok. 256 passed; 0 failed; 27 ignored; 0 measured; 2680 filtered out; finished in 6.26s`.
The native-script, macro and canonical Rust-provider handoff regressions all
executed. Package formatting, strict root-package release Clippy and the locked
offline release build succeeded. Ignored protected smokes were not executed.
The normalized source-tree BLAKE3 is unchanged before/after the build:
`6f1ff45094ed34849af1434863ebb441234f4e67d216b0d9b9bd039fd428c1d7`.
The resulting frozen runner's BLAKE3 is
`b3ae319688efa1d1012caa794e49bfe7aeb0dd4cea3b909590f1d643d5734009`.
Retained log BLAKE3 values match Leviathan: compressed
`3c5c0395597bbc9c31545eebfbe5351798f78b49c29fa16bef7be36b8da63110`,
uncompressed `ebb0f3d6a89dec00a82f5401d433be1b815d9b30faaf62850b1a511461c79505`.
This is source-authoring mechanism and runner-provenance evidence, not actual
provider execution or a fixed point. Tasks 1553/1554 and replacement-cohort
admission remain separate obligations.

Task 1554 completed successfully on an isolated extraction of the exact
`4653ddef…` archive. All nine executed test legs have the same passed/failed/
ignored counts as the task-1550 table above; these are new current-source
executions, not reused older logs. The machine-schema checker passed, pinned
Nickel reported 1.17.0, and Cairn validation plus all three proposal/design/tasks
gates returned success with `valid: true`. The source-bound formatting/Clippy
leg remains the separate successful task 1552.

The exact retained canonical log's BLAKE3 values match the remote originals:
compressed `eca3edf13120cf88791e1b6e44250bf656b8ecd2d1054bb6ee2078c74efc36ef`,
uncompressed `3eb8d80a2dae7536cb938c4d7c181f008cfcfdeac1d16bea44bc726853a835ce`.
Current mechanism/CLI and structural gates do not replace inert-cohort graph
admission, a real cold proof, real bootstrap parity or release-bundle evidence.

## Current native-script corrected inert cohort

Task 1553 completed successfully. The actual source-bundle verifier reported
manifest `11498fc831f409ef59da7fa4bc8d78b43be04e00db0118dbfe210c1b7f8a0c9c`
`ready`, with empty missing, stale, unsupported and untrusted record lists.
Preparation binds refreshed input manifest
`a54619c4769b1b2e65f72c9e16776c536ba5b1a5109ac583c3289a74315da76a` and
input plan `5b585a63a3befc070b2af5886ccab1f79212a1ebf29a478a3f69cf9a4d7f26e7`.
The four declared program identities are retained in the exact preparation
report, alongside its inert-only non-claim, in `2026-11-13-leviathan-4653ddef/`.

Raw prepared-profile BLAKE3:
`49530bca341cafbc29fd8e1b4b4e9fae14c8ea952319a11c28005e223309a535`;
raw preparation-report BLAKE3:
`e9b9098ef6af3cc1d812b794f2fc97d2e041a8c84020162a65abeaabb09938a6`.
These are byte identities, not the canonical profile manifest. The independent
identity review accepted the exact inert payload/source bindings. The complete
graph review confirmed all 740 native-script references per Mantle recipe are
now stage-local, but rejected launch for a separate reachability boundary: only
22,126 of 24,101 steps are reachable from binary/smoke roots; 1,975 selected
topology steps are omitted. Semantic-product roots cover all but 12 PerlAsm
authentication steps, whose unused request-artifact inputs are pruned.

Both reviews are retained. The cold-proof script was never executed. The current
repair roots the selected semantic products, not arbitrary unused helpers, and
makes PerlAsm consume authenticated source bytes/members while preserving its
logical pathname and AWS's source-owned probe transformation. No validator,
guard, source pin or proof bound is weakened. No provider, stage, fixed-point,
parity or release acceptance follows from the rejected source-ready profile.

## Selected topology and authenticated PerlAsm repair

Mantle now roots its selected semantic Rust-unit/build-script products alongside
the binary and smoke observation, with deterministic deduplication. An unused
helper still fails strict reachability. PerlAsm now consumes an exact produced
file/member, preserving the logical source path separately. Ring's unchanged
request checks publish the checked script bytes rather than the binary request;
AWS consumes its produced transformed Perl tree, retaining the assembler-probe
cut and original-source dependency. ADR 0149 records this contract and ownership.

Task 1555 failed compiling a new regression fixture because its borrowed
used-input set overlapped mutable pruning; no tests executed. That fixture now
uses the same collect-unused-keys-before-removal pattern as `Stage::finish`.
The preserved failure is not a production admission relaxation.

Task 1556 passed
`test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 2961 filtered out; finished in 0.00s`.
These exercise selected semantic obligations, rejection of dummy helper roots,
authenticated file/tree-member dependencies after pruning and the earlier native
probe-state invariant. The command also compiled the actual ring/AWS helpers and
executed the production Perl wrapper with bounded host fixtures. Captured output
confirmed exact authenticated script bytes, separate logical path and both SHA
basenames; rejection of changed request cwd before publication; rejection of a
changed source pipe boundary before raw output; and preservation of AWS's actual
source-owned assembler-probe transformation.

Exact commands, source-leaf hashes, terminal results, runtime fixture inputs and
wrapper bytes are retained in `2026-11-13-leviathan-4653ddef/`. Owned temporary
fixture scripts and binaries were removed after execution. These are mechanism
and host-fixture observations only, not protected provider execution, an
original compiler, a bootstrap fixed point, parity or release eligibility. A new
frozen source, generated cohort and independent admission are still required.
