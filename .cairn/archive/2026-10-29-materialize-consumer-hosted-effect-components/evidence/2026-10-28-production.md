# Hosted-effects producer verification, 2026-10-28

## Scope and immutable inputs

Published Octet: `abb6082062251944a40093a274df3d35331fa168`.
Full catalog: `09f1fbb7ce4d9da085b3f1325dbd5e8ae1c17bde2755caf79c1d2850e67b116a`.
Closed effects profile: `a3ad131b214390c9e0ab515e0a63698067dcdd15f2fd4abbc1fd2d120726292e`.
Portable and Molten baseline identities remain unchanged.

## Exact production observations

| Product source | Source closure BLAKE3 | Bundle identity | Final artifact BLAKE3 |
|---|---|---|---|
| Mantle neutral actor fixture | `97a132b98b163204a08309365c4580e2503420132254e9f06c1b7f1977ec361a` | `e9f156e1e10e85ac01ff32944dc284c8e550f0e82614a7ca891730d09a3aeeaf` | `db83d1d091b1dc4dfe92a2dd0a9c9b300fcef8a94a6fdb39d6855f374b49b657` (6123 bytes) |
| Molten-owned extension callback fixture | `544bf1071f42c33bc5b9ac7ce4aa2bbc87f14801086931ee692913de40cbee64` | `8f6eb7da03079e9c08bb5028259127f80602856f98688dcfe8f4ab1a12a68282` | `e7492c6e50b201cff0d836955aa7ce13ce97fed9e7d884e95fe91d0a4e904e2a` (19907 bytes) |

Both unchanged real CLI bundles passed report verification and complete Structural+Bytes verification, including directory-aware source closure remeasurement and copying. Both export the typed `consumer-host-required` policy for `molten:component-effects` / `actor-effects` and exactly `effect-request`. The exact versioned WIT is a byte-verified Molten-owned input.

Cohort: `703be19cf0bab382e10a65790c39540a45c5ab89262070ef9036c1ee35f5f306`.
Realized toolchain root: `/nix/store/rxkdd20n7c510z6n0jd4s4f15zf75lfb-mantle-wasm-component-toolchain-v1`.
Producer binary (BLAKE3 and locator): `b01ed12af74376724af035945b50e723b6c5325abde45c07696c3a8a2ac8546b  /home/brittonr/.cache/mantle-octet-profiles-20261028/final-quality-target/debug/mantle`.
Immutable retention of that observed producing binary: `/nix/store/mz2b9n74j316dfvc6hhi6k5g19vsx8qy-mantle-hosted-effects-observed-producer`; its independently remeasured BLAKE3 equals the producing-binary observation.
Exact versioned WIT BLAKE3: `9478f7ddd9380bcbe898f2101d615674067e0f167ddc6f0fc345553751b5eaa2`.
Runtime profile: `046921e828525584ec883c92b141e224b05d88dfab519af725c2d6645000cdab`.

## Standalone regression

`cli-regression.log` records three passing real pinned-toolchain CLI regressions: the standalone pipeline with rehashable component evidence, target-specific Wasmtime AOT without portable promotion, and fail-closed identity/interface/composition/environment/runtime drift. The production test requires execution receipts for all eight tool roles, including Wasmtime, and rejects tampered emitted component evidence.

`post-style-cli-regression.log` repeats all three after the final source corrections, now using genuine legacy standalone request payloads that omit runtime_validation. A separate final-code real hosted CLI smoke passed complete Structural+Bytes/source-copy verification and retained the same exact neutral source, WIT, final artifact, toolchain cohort, and runtime-profile/host-requirement identities without replacing either handed-off bundle.

## Commands and retained failures

Every recorded command, environment override, exact exit status and complete captured output is retained alongside this document. `retained-evidence.json` measures the retained files.

The initial focused test run passed 18 adapter tests but failed one core regression: its new host-requirement receipt reused another fixture receipt identity. Distinct fixture identities corrected the test without weakening production validation. The initial ordinary quality run caught three obsolete diagnostic-wording arguments after the assertion helper had migrated to behavioral denials; those arguments were removed, not re-pinned. The next workspace run reached seven existing Unix-domain socket tests whose generated socket paths exceeded SUN_LEN under the long TMPDIR. A short private TMPDIR corrected the harness, without editing unrelated source or filtering tests. The first real preparation exposed a noncanonical default fixture path; resolving the workspace ancestor corrected that producer example. The first real CLI bundle correctly stopped on empty-lock byte drift: the source locks were then regenerated through the actual pinned wkg owner, rather than rewriting any produced bundle or weakening drift checks. Rejected profile transcription and observer error-adaptation attempts are also retained. Every exact failed command/output remains alongside its successful successor.

Independent closed-profile denials were preserved while the genuine source/compiler cohort was completed: the neutral source now declares size optimization, LTO, and disabled sign-extension in checked Cargo inputs. The closed checker catalog/profile and baseline identities were not relaxed. The first callback attempt exposed an ambient env::memcmp import; its product owner supplied a real guest-local implementation before the fresh callback source was admitted. No generated binding, Wasm, bundle, export, or sidecar bytes were hand-edited to manufacture admission.

The ordinary first-party workspace quality rail passed. The following full Nix rail found the new empty-lock collection used an implicit foreign-type serde default, rejected by the unchanged Tiger Style policy. The source now uses the repository convention of a named explicit empty constructor, without a lint suppression. The existing empty-closure regression also exercises omitted-package deserialization; fresh core tests, actual-lock API smoke, scoped Tiger Style, and the complete Nix rail cover that final source correction. Original producing binary and genuine bundle bytes remain the measured historical observations above. The checker CLI does not implement --help; its pinned parser supplied the supported scoped check syntax after that failed help attempt was retained.

The next full Nix pass progressed beyond that corrected core and identified six adapter style violations: a foreign-type runtime-policy default, non-predicate boolean names and an overly compound guard, and a six-parameter copy helper. Explicit standalone construction, positive predicate grouping, and deriving the canonical output inside the five-parameter helper corrected the source. The helper now preserves only the needed plan identity rather than cloning the complete plan. No lint level, profile bound, validation stage, or runtime requirement was weakened; all failed Nix attempts are retained beside the final scoped and full checks.

| Required command record | Exit status |
|---|---|
| `checker-lock-update.json` | 0 |
| `toolchain.json` | 0 |
| `toolchain-checks.json` | 0 |
| `focused-core-consumer-corrected.json` | 0 |
| `consumer-no-std.json` | 0 |
| `changed-package-format.json` | 0 |
| `canonical-fixture-format.json` | 0 |
| `neutral-checked-lock-owner-regenerate.json` | 0 |
| `neutral-checked-lock-owner-identity.json` | 0 |
| `producer-binary-identity.json` | 0 |
| `producer-binary-version.json` | 0 |
| `mantle-build.json` | 0 |
| `cli-regression.json` | 0 |
| `ordinary-quality-short-tmp.json` | 0 |
| `post-style-native-nix-checks.json` | 0 |
| `explicit-lock-default-core-tests.json` | 0 |
| `explicit-lock-default-format.json` | 0 |
| `explicit-lock-default-api-smoke.json` | 0 |
| `explicit-lock-default-tigerstyle.json` | 0 |
| `durable-live-receipt-export.json` | 0 |
| `durable-live-receipt-hash.json` | 0 |
| `neutral-prepare-closed.json` | 0 |
| `neutral-build-closed.json` | 0 |
| `neutral-verify.json` | 0 |
| `callback-prepare-local.json` | 0 |
| `callback-build-local.json` | 0 |
| `callback-verify.json` | 0 |
| `production-proposal-gate.json` | 0 |
| `production-design-gate.json` | 0 |
| `production-tasks-gate.json` | 0 |
| `production-review-completeness.json` | 0 |
| `production-native-strict-validate.json` | 0 |
| `post-default-review-completeness.json` | 0 |
| `post-default-native-strict-validate.json` | 0 |
| `observed-producer-immutable-store.json` | 0 |
| `observed-producer-immutable-identity.json` | 0 |
| `post-style-format-corrected.json` | 0 |
| `post-style-focused-crates.json` | 0 |
| `post-style-consumer-no-std.json` | 0 |
| `post-style-cli-regression.json` | 0 |
| `post-style-binary-identity.json` | 0 |
| `post-style-neutral-cli-smoke.json` | 0 |
| `post-style-neutral-verify.json` | 0 |
| `post-style-tigerstyle.json` | 0 |
| `post-style-review-completeness.json` | 0 |
| `post-style-native-strict-validate.json` | 0 |
| `neutral-replay-checker-collect.json` | 0 |
| `neutral-replay-checker-verify.json` | 0 |
| `callback-replay-checker-collect.json` | 0 |
| `callback-replay-checker-verify.json` | 0 |

## Claim boundary

This evidence proves the recorded scoped production and static/byte verification observations for these exact source, WIT, checker, toolchain, policy, and artifact inputs. A not-run consumer-host requirement is not a producer runtime smoke or authority receipt. No downstream actor/system-extension execution, general correctness, release eligibility, compiler correctness, sandbox effectiveness, or deployment safety is claimed here. Separately owned actual consumer observations must be recorded before lifecycle archival or consumer activation.
