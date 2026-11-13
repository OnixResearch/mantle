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
