# Focused Rust-plan validation

Task-ID: mantle.rust_package_planning.hexagonal_core
Covers: native Rust-plan core, application-owned ports, admitted adapter effects, compatibility, and bounded Cargo-free execution.

## Environment and first coherent binary

Validation used the checkout at `/home/brittonr/git/OnixResearch/mantle`, `TMPDIR=/home/brittonr/scratch`, the checked-in lockfile, and `nix develop --offline --no-write-lock-file`. The first post-cutover root `cargo check -p mantle --bin mantle --locked --offline` and `cargo build -p mantle --bin mantle --locked --offline` exited 0. The actual built executable was:

```text
/home/brittonr/scratch/rust-plan-integrated-target/current-tree-stable/debug/mantle
SHA256 9bff7a785ac1e74bff7966cf95e8f1501daed8ce39095e16c407b14949409eb0
size 1551831776 bytes; mtime 2026-10-01 00:45:02 -0400
```

The build preceded the live concurrent-socket proof, so this fingerprint identifies the binary used for the following runtime observations. A later shared-source edit is **not** covered by this fingerprint.

## Pure-core and application tests

Command inside the offline Nix shell:

```text
cargo test -p mantle-rust-plan-core -p mantle-rust-plan-app --locked --offline -- --nocapture
```

Exit 0. Exact test result lines from the completed run (`artifact://8343`):

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

The 10 application and 24 core fixtures include admitted port observations, rejected malformed and wrong observations, bounded planning, and package/feature/topology cases. The two application and eleven core rustdoc tests include compiled negative authority examples. `cargo clippy -p mantle-rust-plan-core -p mantle-rust-plan-app --all-targets --locked --offline -- -D warnings` exited 0; `cargo check -p mantle-rust-plan-core --target wasm32-unknown-unknown --locked --offline` exited 0. A wasm target check is a compilation boundary, not a claim that WebAssembly was run.

The eleven compiled `compile_fail` examples in `crates/mantle-rust-plan-core/src/lib.rs:13-90` reject filesystem access, process invocation, ambient environment lookup, Cargo oracle capture, rustc invocation, store mutation, cache restoration, host path operations, async-runtime driving, CLI dispatch, and receipt rendering from core values. Two compiled application-port examples in `crates/mantle-rust-plan-app/src/ports.rs` reject replacing `AdapterError` with a CLI `RunError` on workspace and cache ports. These examples execute through rustdoc's compiler; a source-text scanner is not counted as a core authority boundary.

The checked-in `crates/mantle-rust-plan-core/tests/plan_fixtures.rs` supplies durable compatibility goldens: `accepted_compatibility_claims_distinguish_oracle_and_bounded_surface` fixes the Cargo-oracle, bounded Cargo-free, and blocked classes and ordered surface IDs; `typed_receipt_preimage_hashes_exact_field_order_without_new_domain` compares the receipt hash against the literal `{"unit_id":"unit","receipt_hash":""}` preimage. These assertions ran in the 24/24 core result above; they complement, rather than replace, the detached byte-for-byte CLI golden comparisons below.

The actual bounded Cargo oracle adapter was exercised with `cargo test -p mantle --bin mantle rust_plan::tests::cargo_oracle_ --locked --offline -- --test-threads=1 --nocapture`: exit 0, 2 passed. Real `/bin/sh` child fixtures covered bounded stdout and stderr, child kill/reap after exceeding the pipe bounds, and a successful bounded JSON read. The production adapter caps Cargo stdout at 64 MiB and stderr at 1 MiB while reading the pipes, not after unbounded capture.

## Detached accepted/rejected JSON compatibility

The first binary was invoked directly with the Nix development shell toolchain `PATH`; each invocation exited 0 with **zero stderr bytes**. Full stdout was compared byte-for-byte against detached pre-cutover `.before.json` receipts, not merely parsed JSON fields:

| Fixture | First-binary stdout bytes | SHA-256 of exact stdout | Historical receipt byte equality |
| --- | ---: | --- | --- |
| Cargo-oracle accepted local-path workspace | 19827 | `3c9685761bcecdbb5cff28f15edc96e3309c979a518ca4bf7a3f5c74f1c173da` | yes |
| Cargo-forbidden accepted local-path workspace | 18812 | `20c3a5632753981ac6c0b425eb3f5717bf0237729344c8fcd42a434878389900` | yes |
| Cargo-forbidden unsupported bench surface | 8662 | `62052224715b7a8f59306eaf908bc1aa16afb767a8e37ad9862b8214c1f6b139` | yes |
| Cargo-forbidden unknown feature rejection | 19610 | `5b6a16e8cbf20b273cdbf05d5b26724f48461c5b698ad5f073503402c44447f3` | yes |
| Cargo-forbidden cold two-unit rustc execution | 23634 | `beeaac4c50e3ce54ebab0d70aff100d177bec5d4ce00092297cd9d2f123884f4` | yes |

The cold execution used the same explicit `/home/brittonr/scratch/rust-plan-fixtures/exec-parity` output root as the historical golden; its previous scratch outputs were moved aside and restored, and new compiler outputs were archived separately under `exec-parity.first-binary-cold-2026-10-01`. Both real units recorded `rebuilt-explicit-unit`, completed successfully, and emitted the exact historical receipt bytes. The prior cached-output state was preserved, not deleted. The deliberately failing Cargo shim marker remained absent.

## Cargo-forbidden bounded compiler proof

Executed against that fingerprinted binary:

```text
scripts/prove-cargo-free-rust-plan.sh --full --bundle-dir /home/brittonr/scratch/rust-plan-first-proof-2026-10-01 --mantle-bin /home/brittonr/scratch/rust-plan-integrated-target/current-tree-stable/debug/mantle
cargo-free proof OK: /home/brittonr/scratch/rust-plan-first-proof-2026-10-01
```

Exit 0. Generated `meta.json` records `"status": "success"`, `"smoke_stdout": "42"`, and `"cargo_forbidden_marker_absent": true`; `status.txt` contains `0`, `smoke-stdout.txt` contains `42`, and declared library and executable digests are in `output-digests.json`. This proves the bounded fixture's real rustc execution and resulting program output, not full Cargo compatibility, bootstrap correctness, or compiler correctness.

A separate first-binary real no-Cargo build-script fixture with no package `description` also completed both custom-build and lib units; the script supplied `mantle_build_script`, `BUILD_VALUE=env-ok`, and `generated.txt`, with no forbidden Cargo shim invocation. Its explicitly requested local cache **rejected** two action candidates because the Nix absolute C linker and the nested `src/lib.rs` manifest-root path were not both classified; both compilers executed. This is successful fail-closed compiler fallback, **not** a cache hit. The proposal explicitly prohibits changing linker or cache decisions; the post-first regression checks the same fail-closed behavior and a second real build-script metadata run, but has not yet been executed against a coherent second-stage source snapshot.

## Pre-final focused-root diagnostic

The first isolated-target `cargo test -p mantle --bin mantle rust_plan::tests:: --locked --offline -- --test-threads=1 --nocapture` completed with the exact result:

```text
test result: FAILED. 202 passed; 2 failed; 0 ignored; 0 measured; 2434 filtered out; finished in 4.51s
```

The two failing legacy fixtures manually cleared a planned effect's declared outputs or called an isolated target while it had a selected host producer. New selected-effect preflight rejected them earlier with typed `incomplete-unit-effect` and `isolated-unit-has-selected-producer`, respectively. These fixtures had asserted obsolete later adapter errors (`missing-declared-output`, `missing-host-artifact`) and were removed rather than re-pinned to the new error words. At this pre-final boundary no passing-after focused-root result was available; the committed-tree V4 result below supersedes that gap.

## Provisional second-source type check

The one user-authorized shared-root command ran against a separate warm target, **not** the pinned first executable:

```text
TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/rust-plan-private-root-target NIX_CONFIG='eval-cache = false' nix develop --offline --no-write-lock-file -c cargo check -p mantle --bin mantle --locked --offline
Finished `dev` profile [unoptimized + debuginfo] target(s) in 5m 23s
```

Exit 0 (`artifact://9689`); the Mantle binary target emitted 13 dead-code warnings from other owner files and **no new type errors**. Retention's shared `crates/crunch-store/src/roots.rs` unique per-interest selection and test changed during this check, so this run is **provisional, not proof of a final coherent source snapshot**. It did not link or replace the SHA-256-pinned first executable. The changed-source focused gate and final V4/V5 tests must follow the post-backend source leases.

## First coherent pre-watch-CLI root type check

After the Watch Worker and pipeline authors released their source, the private-target worktree check ran with `TMPDIR=/home/brittonr/scratch`, `CARGO_TARGET_DIR=/home/brittonr/scratch/rust-plan-private-root-target`, and `NIX_CONFIG='eval-cache = false'`:

```text
nix develop --offline --no-write-lock-file -c cargo check -p mantle --bin mantle --locked --offline
warning: `mantle` (bin "mantle") generated 13 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 15s
```

Exit 0; no Rust source type errors. Three earlier attempts stopped in vendored `bzip2-sys`, `zstd-sys`, and `nickel-lang-parser` build scripts because already-generated files in this **private scratch target** were read-only and their `fs::copy` calls could not overwrite them. Restoring write permission to this private target's build artifacts resolved those failures without changing repository or vendor sources. This check preceded the guarded Watch CLI source changes and does not certify the final committed binary or complete V4/V5.

## Committed-tree V4 focused acceptance (2026-10-10)

The source under test was `710add98b8090cca1ac9617c36df94d2766295d0` in the isolated Leviathan worktree. The Nix development shell used `--no-write-lock-file` and exact-revision local Git overrides for `cairn`, `cairn/artifact`, `tigerstyle`, `artifactAuthSource`, `octet`, `octet/valence`, and `octet/cairn`; the local source NAR hashes matched the locked source hashes. The private upstream archives could not be fetched directly. The Octet pin was imported from the existing workstation Octet checkout into Leviathan's input mirror without changing the Mantle lockfile or Octet source.

With `TMPDIR=/home/brittonr/mantle-cairn-finish/targets`, an isolated `CARGO_TARGET_DIR=/home/brittonr/mantle-cairn-finish/targets/remote-core-style-20261008`, and the checked-in toolchain, these commands exited 0 in that shell:

```text
cargo test -p mantle-rust-plan-core -p mantle-rust-plan-app --locked --offline -- --test-threads=1
cargo test -p mantle --bin mantle rust_plan::tests:: --locked --offline -- --test-threads=1
cargo test -p mantle --test rust_plan_cli --locked --offline -- --test-threads=1
```

Exact `test result:` lines, in execution order (application library, application fixtures, core library, core fixtures, application and core compiled negative authority examples, root adapter, and real CLI integration):

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
test result: ok. 202 passed; 0 failed; 0 ignored; 0 measured; 2431 filtered out; finished in 3.02s
test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 180.75s
```

The retained full transcript is `/home/brittonr/mantle-cairn-finish/logs/rust-plan-v4-focused-octet-valence-cairn-20261010.log` (BLAKE3 `d5331b216fbd103349e1eaeea5327612e4163240adaf02e0e919112a40a7b807`). The 59 CLI tests exercise real accepted and rejected `rust-plan` processes and bounded compiler execution. This is focused V4 acceptance, not full Cargo compatibility or a self-build proof; V5 is checked separately below.

## Committed-tree V5 bounded gates (2026-10-10)

The source under test remained `710add98b8090cca1ac9617c36df94d2766295d0`. The same exact-NAR-mirrored developer shell used for V4 ran these commands with `--locked --offline` and exited 0:

```text
cargo check -p mantle-rust-plan-core --target wasm32-unknown-unknown
cargo clippy -p mantle-rust-plan-core -p mantle-rust-plan-app --all-targets -- -D warnings
cargo clippy -p mantle --bin mantle --no-deps -- -D warnings
```

The wasm core check finished in 8.80s; focused core/application Clippy finished in 2.83s; strict first-party root adapter Clippy finished in 1m 48s. An initial root Clippy command without `--no-deps` failed on 36 existing `vendor/fuse-backend-rs` lints, so only the failed first-party root leg was rerun with the repository's vendored-dependency exclusion. This is not a strict Clippy claim about vendored packages. The retained transcripts are `/home/brittonr/mantle-cairn-finish/logs/rust-plan-v5-wasm-clippy-cargo-free-20261010.log` (BLAKE3 `6eb39fafc15b553f9d3128a7d98120244be11c6dcc4dea620c9fbb04333834f0`) and `/home/brittonr/mantle-cairn-finish/logs/rust-plan-v5-root-firstparty-clippy-cargo-free-20261010.log` (BLAKE3 `a43ba91961f21b09c6dcf2873b62af77beb27030e40e061f923e74b990791f46`).

The corrected leg then executed `scripts/prove-cargo-free-rust-plan.sh --full --bundle-dir /home/brittonr/mantle-cairn-finish/proofs/rust-plan-v5-710add98b-20261010 --mantle-bin /home/brittonr/mantle-cairn-finish/targets/remote-core-style-20261008/debug/mantle`. Its actual output was `cargo-free proof OK: /home/brittonr/mantle-cairn-finish/proofs/rust-plan-v5-710add98b-20261010`; the CLI binary BLAKE3 was `deef9f8a48e94663f6fd57f19e6713136158c1f559d3b1763de65050472c86a6`. The bundle's `meta.json` has schema `mantle-cargo-free-rust-plan-proof-v2`, `status = success`, `smoke_stdout = 42`, and `cargo_forbidden_marker_absent = true`; `status.txt` contains `0`, actual executable stdout contains `42`, and the receipt reports `topology_execution.execution_status = success` with three successful real compiler units (two libraries and one binary). Receipt BLAKE3: `541516b4986e9d90803b6ba1e99a3a8389b00009870f8ff65b1dd7d57a705eed`; metadata BLAKE3: `9032986d33e312e177e7a4a4ebfbaf8bfc718647d3028d74c8b5d86f2baed308`. This fixture does not establish Cargo compatibility, fixed-point self-hosting, bootstrap provenance, release reproducibility, or network fetches.

The full `nix flake check -L --keep-going --no-write-lock-file` using the four exact locked-source local Git mirrors completed `all checks passed!` on x86_64-linux: the root nextest summary was `6262 tests run: 6262 passed, 150 skipped`. Incompatible `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` were omitted. Full transcript: `/home/brittonr/mantle-cairn-finish/logs/rust-plan-v5-full-flake-verified-inputs-20261010.log`, BLAKE3 `299c9a5f705917ea5544aa18e4483f134731cf813d55df7c546670fbc8cf1237`.

At the V5 run, Mantle's then-pinned Cairn revision `695124d459574ba7aeba6097310d237f393c243c` rejected the checked-in `outcome_machine` admission, as documented in `cairn-policy/PIN-SKEW-2026-09-11.md`; it could not validate that policy. The repo-prescribed live-checkout Cairn CLI was used from Leviathan's `/home/brittonr/mantle-cairn-finish/cairn` at `007ba5193dd8fec9067a1c0e0f8ef487807af872`, with the exact locked artifact source mirrored locally. Direct private Git SSH was unavailable; neither the Mantle lockfile nor the Cairn policy export was changed in that V5 run.

After V4 and V5 were marked done in `tasks.md` and the evidence above was written, `cairn validate --root .` reported `valid: true`. `cairn gate proposal`, `cairn gate design`, and `cairn gate tasks` for `separate-rust-plan-hexagon` each reported `verdict: PASS` and `valid: true`; the tasks gate returned no ready tasks. The final retained gate transcript is `/home/brittonr/mantle-cairn-finish/logs/rust-plan-v5-current-cairn-final-post-evidence-20261010.log` (BLAKE3 `a367776cd4e23ae8388721fce65a7e3c1b1e11ad11d6ed4fb001d20556aa130d`). These are validation and readiness checks, not a Cairn archive, self-build proof, release, or publication.

## Published Cairn pin acceptance (2026-10-10)

Mantle's `flake.nix` and generated root `flake.lock` now pin published Cairn
revision `007ba5193dd8fec9067a1c0e0f8ef487807af872` with NAR hash
`sha256-Kgx8JgsC/+26ETbL3lRaxXAYfzv2cmtG/i9OywPOz0c=`, verified both
through GitHub and the local mirror. The dev shell put the Nix-packaged
`/nix/store/1z6bhr2cl43z4kpsvjkxy9b47nqr9pkg-cairn-0.1.0/bin/cairn`
on `PATH`. Running that pinned CLI on the repository reported `valid: true`
for `validate --root .` and `valid: true`, `verdict: PASS` for proposal,
design, and tasks gates for this change; the tasks gate returned no ready
tasks. The same seven exact-revision input mirrors were necessary on
Leviathan because direct Git input fetching was unavailable.

The retained pinned-tool transcript is
`/home/brittonr/mantle-cairn-finish/logs/mantle-cairn-pin-007ba-cli-gates-corrected-20261010.log`
(BLAKE3 `d615f47e2a866f7aa5974054cbf500d573526e0f6a1ba08eb51f0762c35e0552`).
This supersedes the pin-skew workaround for new Mantle checks; it does not
retroactively change the V4/V5 source or establish archive, fixed-point,
release, or publication readiness.

## Final pinned-input Nix acceptance (2026-10-10)

The Cairn pin changes the root `flake.nix` and `flake.lock` digests consumed by
the durable-file-publication adoption receipt. Pinned Nickel 1.17.0 regenerated
the JSON from its typed `.ncl`; the independent validator and `.blake3`
sidecar now bind the current Nix file hashes. The first broad gate exposed a
separate root-package `cargo test` failure in the remote coordinator lock
test under parallel libtest scheduling; the same source test passed in both
isolated nextest binaries. The Nix `crunch` package test phase now uses
`--test-threads 1`, matching Mantle's existing first-party quality rail
without changing that test's internal two-thread concurrency or the
fail-closed mutation guard. This does not establish a root cause for the
earlier transient lock-busy result.

The focused adoption check and serialized root package check passed; the root
binary unit-test legs each recorded `2,565 passed; 0 failed; 68 ignored`.
Their retained transcript is
`/home/brittonr/mantle-cairn-finish/logs/mantle-cairn-pin-007ba-focused-adoption-serial-package-20261010.log`
(BLAKE3 `e5e03505c3a20360c98b94be620046acf579284d6173b5a01eaee95ab1fbce50`).
On the final documentation-inclusive source snapshot,
`nix flake check -L --keep-going --no-write-lock-file` with exact-NAR local
source mirrors completed `all checks passed!` on x86_64-linux; its nextest
summary was `6262 tests run: 6262 passed, 150 skipped`. The final transcript
is `/home/brittonr/mantle-cairn-finish/logs/mantle-cairn-pin-007ba-post-docs-full-flake-20261010.log`
(BLAKE3 `b0f362f8949005d02478599dd9d0a6a1b60a19a2c41be3fb822e3b774e97f5a7`).
The gate omitted incompatible aarch64-darwin, aarch64-linux, and
x86_64-darwin systems. It does not archive the change, repair the separate
outdated function-address release-intake fixtures, or prove self-build,
release eligibility, or deployment.

## Native Cairn sync and archive after publication (2026-10-10)

With the verified implementation and Cairn pin committed on published Mantle
`main` at `f17816217575d659390913af9df7b6b68d8c618e`, the Nix-packaged
Cairn binary named above executed the `separate-rust-plan-hexagon` spec sync.
The sync reported `blocked: false`, `mutated: true`, and added the four
`rust_package_planning` requirement IDs for the hexagonal core,
application-owned ports, explicit unit effects, and compatibility to
`.cairn/specs/rust-package-planning/spec.md`; its expected accepted-spec hash
was `6cef3707548ab39e2c680e2ce595034d8175a949274947ad2da5629db69e0c0c`.
The retained sync receipt is
`/home/brittonr/mantle-cairn-finish/logs/mantle-rust-plan-cairn-native-prearchive-sync-20261010.log`
(BLAKE3 `ea17f0669a61c38e2f7dd8ace4b6061bceca636ffabd71a4ba2a4b2812206f49`).

On the synced main tree, Cairn validation reported `valid: true`; proposal,
design, and tasks gates each returned `valid: true`, `verdict: PASS`, with no
ready tasks. The archive dry-run was unblocked with no reasons. Full gate and
dry-run output is retained at
`/home/brittonr/mantle-cairn-finish/logs/mantle-rust-plan-cairn-native-post-sync-prearchive-gates-20261010.log`
(BLAKE3 `6e3a9b9df245ce2ee745d17200f261d85bb558ad9642d857ba049a1c2fc98f52`).
The dated native archive then reported `blocked: false`, `mutated: true` and
moved the change to `.cairn/archive/2026-10-10-separate-rust-plan-hexagon/`;
post-archive validation returned `valid: true`. Its retained transcript is
`/home/brittonr/mantle-cairn-finish/logs/mantle-rust-plan-cairn-native-archive-postvalidate-20261010.log`
(BLAKE3 `68c9fabb65217dbaf1442eb739164741e5d3c8de76d3809a628b07933a5814bc`).
This lifecycle close is not a source-built fixed-point proof, release
eligibility, a passing function-address intake, or deployment.

With the archived change and accepted spec staged as Nix source inputs, the
post-archive x86_64-linux `nix flake check -L --keep-going --no-write-lock-file`
completed `all checks passed!`; nextest reported
`6262 tests run: 6262 passed, 150 skipped`. Transcript:
`/home/brittonr/mantle-cairn-finish/logs/mantle-rust-plan-postarchive-full-flake-20261010.log`
(BLAKE3 `0b82e4b88aa731623c4bbe0882277cbc43121238ae13e7405a8d2270b2fcadf6`).
This gate omitted aarch64-darwin, aarch64-linux, and x86_64-darwin.
