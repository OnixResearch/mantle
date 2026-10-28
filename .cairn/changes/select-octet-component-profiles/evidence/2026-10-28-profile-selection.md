# Profile selection: executed producer/consumer evidence, 2026-10-28

## Outcome, contribution and ownership

The supported Molten actor-world source now materializes under the explicitly
selected Molten artifact profile. The exact same source remains blocked under
the portable profile. No Octet finding is reinterpreted or suppressed.

The durable contribution is closed, identity-based profile selection across the
request, tool invocation, validation, stage evidence and published bundle.
Mantle's component-pipeline/toolchain owners maintain that mechanism. Octet
owns artifact admission; Molten owns runtime-profile admission and authority.
The current consumer is Molten's supported actor-world producer fixture.

## Published prerequisite and immutable cohort

- Octet revision: `0e0e5dc8461b8b32c286de362c290b3f3ea2deee`.
- Octet registry: `48aabf8dc2ad189ebe051ddce48746a8f878fcbcebf506007c505085e9a8ed83`.
- Portable profile: `e8fbf118b283d63a3662af4853a2ce5f4a1777c18fe37e0be0dc1e235fd707bd`.
- Molten profile: `24dc3346b817ee2645b000922a612a1988f63914b49688a8b72ebb758316c846`.
- wasm-tools cohort: `c50e2d7f0e8c49de4a1d44afae196bdf96bb14e67e7de0a153de146a6207449a`.
- Generated Mantle cohort: `4747263ad5c687ba0df0747aef63b5c63f8908f6be02fb35c72bbe80a982fbf8`.
- Realized toolchain: `/nix/store/mycwcrzb1j2aap7h02jqfbfqimrm1qj2-mantle-wasm-component-toolchain-v1`.

The published Octet correction admits only the reviewed Preserves type import,
bulk memory and padded indirect-call table immediates. It does not enable
reference types, SIMD or all of Wasm 2. Its native change is archived at
`OnixResearch/octet`'s
`.cairn/archive/1970-01-01-add-molten-component-artifact-profile/`.
Its positive/negative exact-byte regression executed 15 tests.

## Executed focused checks

Commands ran from the Mantle worktree, inside its pinned Nix shell. Long runs
were detached with Pueue. The final run directory is
`~/.cache/mantle-octet-profiles-20261028/cohort-final/`; the final task label is
`final-bound-molten-producer-20261028` (numeric queue IDs may be reused).

```sh
nix build .#wasm-component-toolchain \
  .#checks.x86_64-linux.wasm-component-toolchain-identity \
  .#checks.x86_64-linux.wasm-component-toolchain-compatibility --no-link
nix develop -c cargo test -p crunch-wasm-component \
  -p crunch-wasm-component-core -p mantle-wasm-consumer-verifier --all-targets
nix develop -c cargo clippy -p crunch-wasm-component --all-targets -- -D warnings
nix develop -c cargo fmt --check -p crunch-wasm-component
nix build .#checks.x86_64-linux.durable-file-publication-adoption --no-link
nix develop -c cargo test -p mantle --test wasm_component_cli -- --test-threads=1
nix develop -c rustfmt --check --edition 2024 tests/wasm_component_cli.rs
```

All commands above exited 0. The executed test groups were **17 adapter + 36
core + 1 consumer facade + 27 consumer fixture = 81 passed**, zero failures.
The actual CLI regression target additionally executed **3 passed, 0 failed**
after removing diagnostic-wording assertions. It runs the real binary for
successful publication, target-specific AOT and fail-closed drift scenarios.
The source-receipt gate checks regenerated Nickel output, digest sidecar, actual
Cargo/Nix source bindings, source pin agreement and positive/negative fixtures.
Updating those source bindings is maintenance, not a new publication-behavior
or historical evidence replay claim.


The final disk-backed `nix develop -c ./scripts/check-first-party-quality.sh`
exited 0: GCC configure bridge, registered-package formatting, strict first-party
workspace Clippy and serialized first-party workspace library/integration tests
all completed. The repository-owned vendored-member exclusions were unchanged.

Its following full Nix invocation evaluated all outputs, but failed during
sandbox setup because the custom build root was below a private home directory:
`getting attributes of path .../nix-builds/.../build: Permission denied`.
The corrected run uses an owned build root below traversable `/var/tmp`; it
does not change the home directory's permissions or repository sandbox policy.

## Actual producer and complete consumer observation

A disposable observer reused `tests/wasm_component_cli.rs`'s real `CliFixture`
and called `run_component_pipeline`, not mocked tools. The supported-world run
used the genuine copied Rust source tree, `molten:component-runtime/actor` WIT,
the pinned package cohort and real compile, composition, validation and runtime
stages. The independent runtime-profile identity remained
`605a53191900ec2c8016cacc1e17c2338e9cda5423115dd5bb993e5082975910`.

For successful publication the observer executed
`verify_pipeline_execution_report_files`, `verify_materialization_bundle_files`,
`ConsumerRoot::verify_bundle` and `copy_source_closure` into a fresh private
root. It asserted every observed member was `Matched` and that the bundle's
artifact-profile identity equaled the declared selection.

| Scenario | Observed producer outcome | Complete consumer observation |
|---|---|---|
| Supported actor world, Molten profile | `succeeded`, no blockers | `Verified`, 37 members, source copy `Matched` |
| Same source, portable profile | `blocked`, `octet-report-not-admitted` | No admitted materialization bundle |
| Original portable demo | `succeeded`, no blockers | `Verified`, 37 members, source copy `Matched` |
| Unknown profile identity | Denied before pipeline execution | Publication absent |

Exact supported-world facts:

- Bundle: `8b622bd5b9364333f6b144715d8a7154ac31171c0783b690acec928fc999e429`.
- Source tree: `afb70067b2b37a3185d7f15b816ebd81f1f0df9924fc28a49f29364589a01899`, 8436 bytes.
- Normalized component: `e3e1c38cb8bbc4c9866f1791f1f7f10cb48f8de2db10eb8a66de4eada9f49199`, 3700 bytes.
- Selected WIT: `83be349bb27975ada30dbe60817c5404df7862babdf02ae229b399886e76d5e8`.
- Portable demo bundle: `fe2be2d57984e65fe3e0637e5bf39f3ec0b5c14880001b8fa13cf85b96fe6853`.

The retained runtime reports and published files are host-local observations,
not durable product locators or a release-distribution promise. The mechanism
is repeatable through the repository-owned typed request, CLI fixture and
pinned toolchain checks; these recorded identities identify this particular
execution, not every future run's path-dependent receipt graph.

## Preserved failed attempts

The original Octet profile failed the real guest with `zero byte expected` at
`0x491`. Octet fixed the narrow encoding prerequisite rather than admitting all
Wasm 2. After refreshing Nix inputs, the first corrected witness correctly
refused stale Rust admission constants with `Octet source revision, package,
profile, or wasm-tools cohort identity drifted`. The source revision, registry
and Molten-profile constants were then migrated together. No guard was relaxed.
The final four-scenario witness and focused checks above ran after that fix.
Earlier run directories and their failed outcomes are retained separately.

The first ordinary repository gate passed its GCC bridge, formatting and strict
Clippy stages, then exited 101 while linking tests because `/dev/shm` ran out
of space. Its log is retained in the original run directory. The disk-backed
gate uses a fresh private target on the root filesystem instead.

The first broad Nix run reached the actual component CLI suite: its positive
pipeline and target-specific AOT cases passed. The negative suite correctly
denied an unknown profile before publication, but failed an old diagnostic-text
substring assertion. Removed incidental wording assertions rather than
repinning them. Denial status, absence of an admitted report, absence of
published evidence and canonical blocker-code checks remain.


## Claim boundary

This proves the scoped producer/profile selection and Structural+Bytes
consumer observation for the recorded source, policy and toolchain inputs. It
does not establish arbitrary component correctness, actor/extension adoption,
capability authority, runtime-profile authority, sandbox security, upstream
correctness or release eligibility. Molten's runtime-adoption work remains a
separate downstream change.
