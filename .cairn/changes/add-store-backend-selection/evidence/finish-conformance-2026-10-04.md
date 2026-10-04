# Backend-selection conformance rail: scoped results (2026-10-04)

This note records executed targeted tests and scoped quality checks, not
unconditional acceptance of T3.1, a full-workspace gate, or Casita release
eligibility. Source commits `91cf5e5d` and `eb4b6874` on published base
`c5740ee6e220c41c16eaa2de988eaf6c489aea1b` were combined with
authored evidence commits `01105c15` and `33709d4c`; later integration
commits and their distinct results are identified below. Historical goldens
are from *pre-selection* `7ec5177718a6950297e04eb4eb957a10b02e23ce`
(see `prechange-snix-golden-2026-10-04.{md,json}` and
`finish-inventory-2026-10-04.md`). The signer is the repository's explicit
**TEST-ONLY/non-production** `tests/store_archive_cli.rs:22-23` fixture;
the alternate signer uses fixed `[19_u8;32]` test bytes. Historical and
first combined golden comparisons ran in `nix develop` with private targets
under `/home/brittonr/.cargo-target/`, `TMPDIR` under
`/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/tmp`,
`LANG=C LC_ALL=C TZ=UTC SOURCE_DATE_EPOCH=1600000000`, and
`CRUNCH_CONFIG_DIR`/`MANTLE_STORE_BACKEND` unset. Later commands state
their own environment and target. No pueue, push, original checkout
mutation, or source bootstrap was used.

## One real backend-parameterized core rail

Executed command:

```sh
env CARGO_TARGET_DIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/selected-golden-target \
  TMPDIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/tmp \
  nix develop -c sh -c 'env -u CRUNCH_CONFIG_DIR -u MANTLE_STORE_BACKEND LANG=C LC_ALL=C TZ=UTC SOURCE_DATE_EPOCH=1600000000 cargo test --test store_archive_cli admitted_backends_share_signed_core_gc_identity_and_profile_conformance_rail -- --exact --nocapture'
```

Observed: `1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out`.
`tests/store_archive_cli.rs::admitted_backends_share_signed_core_gc_identity_and_profile_conformance_rail`
executes the **same fixture loop** for `snix` and `casita`, not a test-name
inventory. Its consumer-visible operations per backend are:

| Required property | Same-fixture observed behavior |
|---|---|
| Signed output admission, local lookup and fresh handle reopen | Three signed file PathInfos admitted; reopened original signed PathInfo equals the seeded record. |
| Closure resolution, physical export, NAR and verification | Referenced child appears in strict closure; all three exported content bytes match their originals; `store verify` reports `trusted_signatures=1/1` for each. Both backends yield the same signed retained PathInfo and NAR SHA-256 `42a7f16a040111ab52d03cf78d7178101b4746c983a12551f07b305181f7024e`. |
| PathInfo-backed `ActionResultPort` reuse | Fresh handle probes the retained output and reports its original PathInfo plus matching reused NAR bytes under **both** profiles, including `casita` without `rust-unit-cache`. |
| Store archive export/import, second signer | Exports the retained root and its referenced child, imports into a fresh state of the same selected backend with an explicit signer policy, observes unchanged PathInfo facts; real `store sign` with a separately provisioned test key appends a second signature, and a **new `store verify` child process** with both public keys cryptographically checks `trusted_signatures=2/2`. |
| Identity and mixed-backend negative | Fresh `mantle-store-state-v2` records the selected backend. A real CLI read with the other backend fails `store-backend-mismatch` and every recorded state-file byte remains identical. |
| Profile-dependent behavior | Snix reports unbounded `max_root_changes` and supports `rust-unit-cache` and overlays; Casita reports 1,024 and no Rust unit cache or overlays. Casita `--base-store` fails with `casita-overlay-unsupported`, with all recorded state-file bytes unchanged. This checks the declared bound, **not** the 1,024/1,025 admission behavior. |
| GC candidates, root transition, stale rejection and accepted execution | A deterministic legacy retained root (`created_unix_s: 100`) keeps its referenced child while leaving exactly one unretained candidate. Pinning that candidate after the first plan makes `--execute --plan-id` fail with the declared `stale-gc-plan` under Snix or `gc-plan-stale` under Casita; exported content remains. Replanning reports zero candidates. Unpinning the candidate and dry-running again produces an accepted plan whose execution completes without failed operations, removes the candidate export and PathInfo, and retains the rooted output and referenced child for fresh `store info` and trusted `store verify`. |

Exact successful test stdout:

```text
BACKEND_CORE_RAIL snix {"nar_sha256":"42a7f16a040111ab52d03cf78d7178101b4746c983a12551f07b305181f7024e","profile_max_root_changes":null,"retained_store_path":"0000000000068rbfd5hp8rbj5mn6jqbj-rail-retained","signed_count_after_store_sign":2,"stale_plan_blocker":"stale-gc-plan"}
BACKEND_CORE_RAIL casita {"nar_sha256":"42a7f16a040111ab52d03cf78d7178101b4746c983a12551f07b305181f7024e","profile_max_root_changes":1024,"retained_store_path":"0000000000068rbfd5hp8rbj5mn6jqbj-rail-retained","signed_count_after_store_sign":2,"stale_plan_blocker":"gc-plan-stale"}
```

## Other exercised root fixtures

- `tests/store_archive_cli.rs::independent_snix_stores_agree_on_signed_bytes_only_with_the_same_fixture_key`: **1 passed**. Two independent default/explicit Snix states with the same explicitly provisioned key have byte-identical signed PathInfo; a third with another explicitly provisioned key has identical unsigned PathInfo and NAR, different signed bytes, and each state verifies its own signer with `trusted_signatures=1/1`.
- `tests/transcript_cli.rs::transcript_real_child_reopens_selected_casita_and_rejects_dropped_selection_without_mutation`: **1 passed** after the direct-child amendment. The actual Mantle child opened Casita with an outer transcript selection. A direct real child `mantle --state-dir <Casita> --store <...> store list` with **no** `--store-backend`, as well as one with an explicit conflicting child selector, failed `store-backend-mismatch` and preserved every recorded state-file byte. This does not depend on a fake binary or argv echo.
- The first `tests/store_archive_cli.rs::default_and_explicit_snix_preserve_prechange_signed_and_gc_golden_facts`
  run **failed before its explicit-Snix iteration**: signed bytes, NAR hashes,
  store paths, info, and roots matched, but the two blob-index GC observations
  appeared in the opposite order. The historical golden JSON and original
  failed selected-stdout artifact remain unchanged. Controlled same-root
  historical and selected reruns proved the ordering nondeterministic within
  either revision, not a changed candidate or backend-selection difference.

### Combined exact-root CLI, child, and GC results

At combined HEAD `33709d4c`, with the canonicalization in `eb4b6874`, the
actual Mantle child was exercised by the scoped archive CLI test binary at
the **same absolute fixture root and test signer** as the historical golden:

```sh
test ! -e /home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/fixture-state-compare &&
env -u CRUNCH_CONFIG_DIR -u MANTLE_STORE_BACKEND LANG=C LC_ALL=C TZ=UTC SOURCE_DATE_EPOCH=1600000000 \
  CARGO_TARGET_DIR=/home/brittonr/.cargo-target/mantle-rust-script-pin-20261004/gc-ordered-target \
  TMPDIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/tmp \
  MANTLE_BASELINE_FIXTURE_ROOT=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/fixture-state-compare \
  nix develop --offline --no-write-lock-file --command cargo test --test store_archive_cli -- --nocapture
```

Observed: **13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out**.
The strict default- and explicit-Snix comparison both passed the historical
signed PathInfo, NAR, info, roots, complete GC observation/report, and exact
plan-ID checks. Under this same root, both selected plan IDs equal the
prechange `a8cf..., cf82...` golden
`b3:787fc14bbf1d8438cbd52cc1be068d68f62967b5634b5a495c938e24565d586c`.
The same run exercised the real Snix and Casita core rails; the two
`BACKEND_CORE_RAIL` stdout records above were reproduced, including verified
second signatures and fresh accepted plan-bound GC execution after stale
rejection. Neither the golden nor the comparator was relaxed.

The combined `transcript_real_child_reopens_selected_casita_and_rejects_dropped_selection_without_mutation`
test also passed (**1 passed; 0 failed; 11 filtered out**) with the same
isolated target and fixed environment, but without the golden fixture-root
variable. A direct Mantle child missing `--store-backend` failed without
mutating the recorded Casita state, as did an explicitly wrong child.

The final combined `crunch-store` regression
`gc::tests::reversed_blob_enumeration_has_the_same_reclaim_observations_and_plan_id`
passed (**1 passed; 0 failed; 401 filtered out**): explicitly reversed
same-file path sequences use the production canonicalizer and yield identical
reclaim observations and plan IDs without relying on filesystem enumeration.
`gc::tests::blob_creation_order_does_not_change_plan_but_changed_blob_facts_stale_it`
also passed (**1 passed; 0 failed; 401 filtered out**): real dry-runs over
oppositely created files agree, while a changed file size changes the plan ID
and rejects the old accepted ID before deleting anything.

The combined `crunch-store`
`capability::tests::source_slice_batch_publishes_both_backends_without_partial_conflicts`
passed (**1 passed; 0 failed; 401 filtered out**); a test-only N=2 profile
accepted two published, verifiable signed entries under both Snix and Casita,
then rejected N+1 with byte-identical backend state. This does not claim all
remaining T3.3 profile fixtures were executed.

### Historical 7ec same-rail Snix comparator: canonical facts, not every old ID

The comparator consumes **only the preserved first** 7ec three-path rail
capture for signed/NAR and other shared core facts. All executed historical
reruns and their GC orders are documented in
`prechange-snix-golden-2026-10-04.md`; none replaced that first input.
The selected rail was run at the **same absolute physical root**:

```sh
test ! -e /home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/fixture-rail-compare &&
env CARGO_TARGET_DIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/selected-golden-target \
  TMPDIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/tmp \
  MANTLE_RAIL_FIXTURE_ROOT=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/fixture-rail-compare \
  nix develop -c sh -c 'env -u CRUNCH_CONFIG_DIR -u MANTLE_STORE_BACKEND LANG=C LC_ALL=C TZ=UTC SOURCE_DATE_EPOCH=1600000000 cargo test --test store_archive_cli admitted_backends_share_signed_core_gc_identity_and_profile_conformance_rail -- --exact --nocapture'
```

Observed: **1 passed; 0 failed; 12 filtered out**. The three Snix
`rail-*` paths matched 7ec's exact store paths, NAR SHA-256 and sizes,
test-key serialized **signed** PathInfo bytes and physical exported bytes.
Fresh-handle signed reopen, strict retained/child closure, PathInfo-backed
action reuse, archive-list paths and imported signed facts, the independent
second signature verified 2/2, GC retained/candidate and stale-plan
transition, and accepted execution all matched the corresponding 7ec
fixture's **consumer facts**. The same test then ran the Casita core and
declared profile checks; it did not compare Casita-specific profile facts to
the historical Snix-only build.

The full Snix GC report matched after one justified comparison transform:
only the prechange unsorted `blob-index` and `blob-chunk` observation paths are
ordered as the selected production `crates/crunch-store/src/gc.rs`
`canonicalize_dead_blob_paths` does **before hashing**. The selected report
must already be in this canonical order; path, bytes, blockers, candidate,
retention and all other report fields remain compared. For this three-path
rail, numeric plan IDs are displayed **only at the historical physical root**
as two observed results; no exact 7ec numerical identity parity is claimed.
Exact old-vs-selected plan-ID equality is claimed **only** by the existing
two-path `baseline-keep`/`baseline-candidate` strict test at its identical
physical root: both default and explicit Snix passed the original golden
`b3:787fc14bbf1d8438cbd52cc1be068d68f62967b5634b5a495c938e24565d586c`.
Actual first-three-path capture comparison stdout:

```text
PRECHANGE_SNIX_RAIL_GC_PLAN planned old_first="b3:36eb6141fa768f053c08258dc684365f28e908643a2e51bffcb63a0b7cf31491" selected_canonical="b3:c73dcda6e8949135b7d49298cd219c3845e8eb6e18653d60b8cd5bb9c8b90e95"
PRECHANGE_SNIX_RAIL_GC_PLAN fresh old_first="b3:36eb6141fa768f053c08258dc684365f28e908643a2e51bffcb63a0b7cf31491" selected_canonical="b3:c73dcda6e8949135b7d49298cd219c3845e8eb6e18653d60b8cd5bb9c8b90e95"
BACKEND_CORE_RAIL snix {"nar_sha256":"42a7f16a040111ab52d03cf78d7178101b4746c983a12551f07b305181f7024e","profile_max_root_changes":null,"retained_store_path":"0000000000068rbfd5hp8rbj5mn6jqbj-rail-retained","signed_count_after_store_sign":2,"stale_plan_blocker":"stale-gc-plan"}
BACKEND_CORE_RAIL casita {"nar_sha256":"42a7f16a040111ab52d03cf78d7178101b4746c983a12551f07b305181f7024e","profile_max_root_changes":1024,"retained_store_path":"0000000000068rbfd5hp8rbj5mn6jqbj-rail-retained","signed_count_after_store_sign":2,"stale_plan_blocker":"gc-plan-stale"}
```

Prechange 7ec plan IDs genuinely vary with `read_dir` order; the archived
first three-path capture had `blob-index` order `4df6, e6a6, d1f4`
whereas selected Snix deterministically reports `4df6, d1f4, e6a6`.
The selected canonicalized output matches this **valid old-source
observation order** as a candidate-and-reclaim *fact* comparison, not as
observed numerical equality; `[INFERENCE]` the old unsorted enumeration
could also have yielded this ordered sequence, but none of our recorded
7ec three-path runs established that plan ID. The original first rail
artifact, T1.1 golden and red selected-observation artifact are unchanged.
This bounded pre-existing non-repeatability is an explicit supplemental
three-path non-claim, not a favorable-retry golden. The original T1.1
two-path fixed-root capture has exact old-vs-selected numerical plan-ID
equality; the first preserved three-path capture has matching signed
bytes and canonicalized consumer facts but **different** old and selected
raw IDs. Spec `specs/store-backends/spec.md:193-197` also says **every
positive fixture MUST reproduce its golden**. The supplemental
three-path fixture was separately captured, not the original T1.1
golden; if that literal clause includes this independently captured
positive fixture's numeric plan ID, T3.1 cannot currently pass. No
later 7ec traversal may replace the first preserved capture to
manufacture a match. T3.1 remains unchecked pending explicit scope
interpretation even after optional fixtures are exercised.

The corrected source-only capability checker pass below likewise does
not finish the remaining T4.3 quality gates.

The same focused test was rerun without
`MANTLE_RAIL_FIXTURE_ROOT` (`env -u MANTLE_RAIL_FIXTURE_ROOT` in the command
above, retaining the same `nix develop`, isolated target, signer and fixed
environment): **1 passed; 0 failed; 12 filtered out**. It compared all
portable canonical GC facts with paths relative to the fixture roots but did
**not** claim a numeric execution-ID comparison across different absolute
paths; both backends again printed the signed two-signer core records above.

## Complementary fixtures and work still to prove

At this first integrated run, the backend-parameterized rail exercised core operations and
some declared-profile rejections. It does **not** yet invoke all
Snix-declared overlay/atomic-batch/unsigned/Rust-cache positives,
Casita's real 1,024/1,025 batch bound, or every unsupported-feature
negative **inside that same rail**. Standalone source tests cover many
of these behaviors, and those distinct tests are worth exercising,
but their source locations or standalone pass results cannot silently
fulfil the spec's `one backend-parameterized conformance rail` condition.
Existing focused fixtures include Snix overlay composition, atomic
batch N=2/N+1 under a test-only profile, unsigned admission through
Nario v2, Snix Rust cache reuse, Casita's production 1,024/1,025
boundary, disabled-overlay preflight, Casita repair CLI/library
rejections and Snix repair success, a test-only disabled-Rust-cache
real command, and PathInfo ActionResult reuse under the real
no-Rust-cache Casita profile. Their exact executions and the
then-remaining synthetic-profile gap are recorded below.

The original two-path fixed-root historical GC plan-ID comparison
passes; the supplemental three-path comparison makes no exact raw
numeric ID claim. Other T4.3 workspace gates remain open. No
general correctness, durability, GC safety, or release eligibility
claim follows from this rail.

## Source-only quality observation (not a completed selection gate)

The source integration owner reported `cargo test -p crunch-store --lib` after
the pure open-decision split at source-only commit `32c59def`: **403 passed**.
The same owner's pinned-Nix isolated-target run of
`cargo -q -Zscript tools/check_store_capability_boundary.rs --root .` exited
**1** with `files_scanned=558`, three reported findings:
`src/bootstrap.rs:825` (`handle-construction`),
`src/store_cmd.rs:220` and `:448` (two `raw-service-escape`
`.pathinfo_service()` usages). The first source-only result is preserved
as the genuinely red observation, not rewritten as a successful gate.
The source owner subsequently moved sign/verify into named `StoreAdmin`
operations, made obsolete raw query functions crate-private, and removed
their public re-exports. The only exact checker owner added is
`src/bootstrap.rs`, an actual CLI raw-seed fetch composition root:
preflight backend identity, acquire mutation guard,
open the selected `StoreHandle`, and immediately split it into
`PipelineStoreParts`; this owner entry does **not** waive either raw-service
or writable-authority checks. With those source changes, the owner ran
`cargo -q -Zscript tools/check_store_capability_boundary.rs --root .`
again and reported **exit 0**:

```text
files_scanned=558 raw_service_escape_count=0 writable_authority_escape_count=0 handle_construction_escape_count=0
```

The later pass corrects the first failure but is only the source-owner
capability checker observation, not the completed combined-tree T4.3
quality suite, T4.4 archive/sync gate, or Casita release acceptance.

## Integrated fixture additions: red assertion, correction, and scoped observations

The separate combined worktree started at integrated source `94751b40`.
Commit `97bd9ef6` adds byte-snapshot negatives in `tests/store_gc_cli.rs`,
positive legacy/identity-less signed Snix reopen in `tests/store_archive_cli.rs`,
an actual post-import unsigned PathInfo lookup, and explicit Snix selection
for the real repair dry-run/execute CLI fixture. Source-owner constructor
commit `9dcaeca1` was cherry-picked as `c07450c1` into that worktree;
subsequent source fixes and final post-change gates are recorded below.
These additions do not modify either preserved 7ec golden JSON.

The first **14-test** archive run on the initial integrated source produced
**13 passed, 1 failed**. Both old two-path T1.1 fixed-root plan-ID comparisons,
the first-capture three-path Snix/Casita core rail, and the new signed
legacy/identity-less reopen passed. The one failure was the newly added Nario
assertion querying `store info` by `/nix/store/<name>`: CLI `store info`
filters by the stored PathInfo's relative `<name>` and reported
`no PathInfo matching '/nix/store/j3wdfhfzn69xrn6lkk7sm210yx8fp0k7-payload.txt'`.
A throwaway real CLI import/list/info reproduction showed imported
`imported_count: 1`, a persisted relative store path, then **exit 3** for
the full-path query and **exit 0** for its `<name>` query with
`"signatures": []`. The fixture now queries `<name>` and asserts the
persisted unsigned signature list is empty; the focused corrected
`cargo test --test store_archive_cli
nario_v2_cli_lists_imports_and_skips_pinned_producer_fixture -- --exact
--nocapture` passed **1/1**. The first red run is not counted as T4.3 proof.

After the fixture commit and first source constructor change,
`cargo test --test store_gc_cli -- --nocapture` passed **17/17** on the
isolated combined worktree. Its real CLI matrix rejects an identity-less
`casita/` repository with selected `snix` and `casita`, alone and beside
a real Snix database; a foreign file or Snix database under identity-less
`casita` also fails `store-backend-mismatch`. Each rejected case compares
complete nested state and output paths and file bytes before/after.
Wrong-backend sign/build/overlay and environment-selected-backend negatives
preserve the same invariants. `nix develop -c rustfmt --edition 2024 --check
tests/store_archive_cli.rs tests/store_gc_cli.rs tests/integration.rs` passed.
The combined `cargo -q -Zscript tools/check_store_capability_boundary.rs
--root .` exited **0** with:

```text
files_scanned=558
raw_service_escape_count=0
writable_authority_escape_count=0
handle_construction_escape_count=0
```

An initial Rust cache command with `--exact` reported **0 tests**, because
the selected adoption test is under a `tests::` module. It proves nothing;
the corrected substring-filtered actual daemon test is reported below
only after it runs. These are interim observations, not a completed
post-source-change T4.3 suite.

## Durable-open constructor and test-only injected-service boundary

The source owner found a real public constructor-default gap after the first
combined source: `StoreHandle::from_services_with_store_dir` implicitly
selected Snix for prebuilt services. Source commit `9dcaeca1`, cherry-picked
as `c07450c1`, removed `from_services`, requires an explicit backend in the
remaining injected constructor, rejects injected Casita before state access,
and migrated the active build/store/root integration callsites. A further
source audit found that even explicitly selected Snix could load a persisted
Casita state's CA mappings and metadata cache through this helper without
the normal identity preflight. Follow-up source commit `5ec3b25d`, cherry-picked
as `b701841e`, invokes the existing pure `StoreConfig` identity preflight
**before** these loads. A test seeds a real recorded Casita v2 repository,
valid foreign CA mapping and advisory metadata file, then rejects injected
Snix with exact `store-backend-mismatch: requested snix, state declares
casita` and a full byte-identical state snapshot and absent synthetic output.

The injected `StoreHandleServices` and constructor are now available only
under `cfg(test)` or an explicit `crunch-store/test-support` feature. Only
root `mantle` and `crunch-build` **dev-dependencies** enable that feature.
The test-only helper preflights before it reads shell metadata, but its
caller supplies already-built services and it intentionally does **not**
bind an identity or establish the durable backend itself. Do not cite that
synthetic seam as a durable open. Production default builds expose
`StoreHandle::open(StoreConfig)` with a required backend: its pure decision
precedes identity bind, directory creation and selected service open.
On the source-owner isolated checkout after the follow-up, production-default
`cargo check -p crunch-store --lib --no-default-features` passed; focused
wrong-backend helper regression **1/1**, `cargo test -p crunch-build --lib`
**690/690**, real `tests/integration_build.rs` local-file fetch
**1/1**, and package rustfmt passed. The final **combined worktree** package
and CLI gate results are recorded separately below.

The corrected full fixed-root archive CLI test ran **14/14** after the
test assertion fix, but its Cargo command started before the final source
cherry-pick; it is not counted as a post-final-source quality gate. Likewise,
the first complete `tests/integration.rs` run passed **77/77**, including
the Casita repair rejection and explicit/default Snix repair roundtrip, but
its build began before the `test-support` Cargo feature was declared and
printed three `unexpected_cfgs` warnings at the new guard sites. The
definitive archive and integration suites were restarted against the
committed final source and manifest; their exact outcomes are below.

## Reachable combined task receipt before final package gates

On the combined `b701841e` source and `97bd9ef6` fixture commits, the
repository-wide structural `cairn validate --root . --policy
/home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json`
returned `"valid": true`, `"issues": []`, `"change_issues": []`;
`cairn gate tasks add-store-backend-selection --root . --policy <same>`
returned `"valid": true`, `"verdict": "PASS"`, `"task_done": 13`,
`"task_todo": 6`. The selected sibling policy's SHA-256 is
`1501c8c5a387098987d9feef3869e155b47748ae973785ae8f7aee3cfd92bbe9`.
There is **no global Cairn validation blocker** in this run. Structural
PASS with open tasks is not T4.4 sync/archive eligibility. T4.3 remains
open until post-final-source full scoped suites and strict Clippy complete.

## Omitted Rust backend: compile-time negative, not a runtime blocker

A separate scratch Rust caller uses `StoreConfig { state_dir, output_dir,
remote_cache_urls: Vec::new(), base_state_dirs: Vec::new(),
fallback_mode: StoreFallbackMode::Practical, store_dir: "/nix/store".to_owned()
}` and omits only `backend`. With the existing selected `snix` Mantle CLI,
`store list` first seeded a real state (`directories.redb`, `pathinfo.redb`,
`store-identity.json`), all captured as full file bytes. Against the
compiler-compatible `libcrunch_store-60859064e630e08d.rlib`, the pinned
toolchain's `nix develop -c rustc --edition=2024 --crate-name
store_config_missing_backend <scratch>.rs --extern
crunch_store=<rlib> -L dependency=<selection-target>/debug/deps
--emit=metadata -o <outside-state>.rmeta` exited **1** with exactly:

```text
error[E0063]: missing field `backend` in initializer of `StoreConfig`
 --> .../store-config-missing-backend.rs:5:5
  |
5 |     StoreConfig {
  |     ^^^^^^^^^^^ missing `backend`
```

No metadata output was created, and the entire seeded file-byte snapshot
before and after compilation was **equal**. The initially selected older
rlib fingerprint produced only unrelated `E0463` and was discarded; it is
not this fixture's outcome. The stable missing-field class is **Rust
`E0063`**; there is no impossible runtime `StoreConfig`-without-backend
blocker to fabricate. The other T3.2 runtime negatives retain their own
stable blockers and byte-equivalence checks.

## Five launcher forwarding contracts and bounded observed subset

The local remote-worker launcher in `src/main.rs:5388-5440` constructs
the default `remote serve --binding stdio-once` child with the selected
`--store-backend` argument. `src/bootstrap_validate.rs:258-294` forwards
`request.ctx.store_backend` to its real build child.
`src/source_built_fixed_point_shell.rs:1875-1925` adds the selected
backend to the actual native build-child `Command` after it has constructed
a proof staging directory. `src/transcript_cmd.rs:497-513` adds the scratch
backend to child arguments unless the transcript explicitly provided it.
The CLI Rust cache route `src/main.rs:4194-4211` passes the selected
backend into the wrapper daemon; the wrapper's `DaemonOptions.backend`
requires it. Bounded real bootstrap/transcript/daemon child fixtures
were exercised in their focused suites; **neither a local remote-worker
build nor a source-built fixed-point proof was launched**. T2.3's checked
claim covers selected-identifier forwarding in all five source
contracts, not a fabricated runtime proof of those two unbounded builds.

## Broader Mantle bin nonclaim outside the specified focused T4.3 suite

One broad `cargo test -p mantle --bin mantle -- --nocapture` execution
finished with **2482 passed; 2 failed; 68 ignored** (2552 tests;
`artifact://28708`). The failing names were
`external_batch_dispatch::tests::slurm_adapter_submits_observes_reconciles_and_cancels_fake_cli`
and
`protected_exec_seccomp::linux::tests::seccomp_supervisor_reads_deep_descendant_exec_path`;
the latter nested test reported
`Supervisor("adopted StageX descendants did not exit within 30000 ms")`.
The unchanged broad suite was **not rerun**. Its output independently
lists successful `store_cmd` tests, backend-selection parser and test-only
no-Rust-cache negative, and Rust cache CLI parser tests. This is **not**
a claim that the broad binary suite passed; T4.3 names focused Mantle
store suites, so its status also depends on those distinct focused
results, not on disguising the two broad-suite failures.

## Post-constructor focused backend negatives and store-core suite

The post-constructor/feature `cargo test --test store_gc_cli -- --nocapture`
passed **17/17** (the Nix evaluator emitted only an ignored busy-cache
warning). It actually ran identity-less Casita marker negatives under
both selected backends with/without Snix files, foreign identity-less
content, unknown identifier, ambient selection, wrong-backend sign,
build and overlay, Casita Rust-cache daemon and plan preflights, signed
Casita plan-bound GC, and Snix overlay store info. Rejected filesystem
trees were compared byte-for-byte by the focused fixtures. The same
`cargo test -p crunch-store --lib -- --nocapture` passed **405/405**,
including `nario_casita_accepts_configured_1024_root_boundary` and
`nario_casita_rejects_1025_new_roots_before_any_publication`.

The first strict Clippy attempt against touched package targets passed
`-- -D warnings` to vendored path dependencies too, and exited **101**
before establishing first-party lint success: `fuse-backend-rs` alone
raised **36** pre-existing `clippy::io_other_error` errors
(`artifact://28720`). `cargo clippy --help` confirms `--no-deps` is the
supported way to lint only selected first-party packages without
linting their dependencies. That precise scoped follow-up and final
rustfmt/whitespace checks are not claimed here until executed.

The post-constructor `cargo check -p crunch-store --lib
--no-default-features` passed against the isolated selection target.
This is a production-default compile proof, not an optional-profile
behavior fixture.

`cargo test --test integration_build
fetchurl_downloads_and_verifies_hash -- --exact --nocapture`
passed **1/1**, exercising real local-file realization on the combined
post-constructor source. The broader untouched build matrix is not
claimed by this targeted integration result.

## Signed Casita Nario 1,024/1,025 durable-state boundary

Test-only source commit `3780360b`, cherry-picked as `8c0944ad`,
strengthens the **existing signed** Nario v2 1,025-path fixture in
`crates/crunch-store/src/nario.rs`. It snapshots every relative
directory and the complete bytes of every file in the destination
tree immediately before import, after the real `casita-batch-limit`
rejection, and after the failed import handle closes. Each snapshot is
equal; on fresh reopen all 1,025 PathInfos and physical output paths
are absent. Its focused command passed **1/1; 404 filtered**, as did
the separately executed signed 1,024-path import and fresh-reopen
positive **1/1; 404 filtered**. The observed unchanged-production
1,025 rejection is **green**; there was no failing-before durable
mutation and no speculative production Nario rewrite.

This proves no *durable* state bytes or output paths change on signed
Nario over-bound rejection. Casita stages earlier NAR bytes in
session-memory blob/temporary directory services before encountering
the 1,025th record; the fixture does **not** claim zero transient
memory effects, and the archive's declared 1 TiB aggregate limit can
still imply substantial transient resource pressure. The separate
backend-parameterized rail additionally uses real signed
`put_batch_atomic` at 1,024/1,025 to exercise the selected backend
root-commit bound; that lower-level operation is **not** substituted
for the signed Nario importer regression.

## Real dropped-child selection and T3.2 completion

The post-constructor selected-child command
`cargo test --test transcript_cli
transcript_real_child_reopens_selected_casita_and_rejects_dropped_selection_without_mutation
-- --exact --nocapture` passed **1/1; 11 filtered**. It launches a real
transcript child under selected Casita, observes its recorded identity,
then launches a child omitting backend selection and observes exact
`store-backend-mismatch` with a complete directory-and-file-byte tree
unchanged. A transcript explicitly selecting the wrong backend also
gets the expected mismatch without mutation. In combination with the
**17/17** backend-selection/identity/ambient CLI negative matrix above
and the separate real `StoreConfig` Rust `E0063` compiler negative
against byte-identical seeded state, T3.2's runtime/compile-time
negative fixtures are checked. No runtime blocker is attributed to
an unconstructible Rust config.

## Selected build/store CLI integration suite

`cargo test --test integration` completed **77/77 passed** on the
combined selection checkout, including Casita repair rejection before
state creation, selected Snix `store repair-final-nar` execution and
fresh-process archive export, signed store metadata and verification,
bootstrap seed creation/import, and the store pull HTTP closure round
trip. Some builder and shell cases printed `skipping build test:
bwrap or /nix/store not available` / `skipping: bwrap or /nix/store not
available` and returned success through their test functions; the
77/77 result is **not** proof those unavailable sandbox build paths
executed.

## First expanded optional-rail compile attempt

The first expanded `cargo test --test store_archive_cli -- --nocapture`
attempt failed **before running any test** with Rust `E0599`: the new
Rust-cache rail helper tried to clone `StoreConfig`, which intentionally
does not implement `Clone`. This did **not** create the historical fixed
fixture root. The helper now constructs a fresh equivalent typed
`StoreConfig` with a local closure for the first open, reopen, and
unsupported-profile preflight. The failed compile is not counted as
optional-profile or T4.3 proof; subsequent executions must determine
whether the corrected fixture actually passes.

## Expanded rail: environment quota failure and pinned retry

The corrected one-test optional rail compiled, exercised the Snix
core and new optional branches successfully, then **failed on the
Casita 1,024-root positive import at root 226**, reporting
`staging Casita ...: Disk quota exceeded (os error 122)`. Its run had
not pinned `TMPDIR`; Casita creates transaction scratch with
`tempfile::tempdir()`, so this process used `/tmp`. The immediately
observed `df -h` reported `/tmp` on `datapool/tmp` **258G/258G used,
25M available (100%)**, whereas the selected cargo-target filesystem
had **306G available**. This is an environment resource failure, not
evidence of a source correctness defect or a passing Casita bound.
No unrelated `/tmp` or shared cargo state was deleted. A corrected
exact-root conformance run was launched with
`TMPDIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/tmp`,
the original absolute historical
`MANTLE_RAIL_FIXTURE_ROOT=.../fixture-rail-compare`, and
`CRUNCH_CONFIG_DIR`, `MANTLE_STORE_BACKEND` unset plus `LANG=C`,
`LC_ALL=C`, `TZ=UTC`, `SOURCE_DATE_EPOCH=1600000000`; its result is
recorded only after completion.

## One disabled Rust-cache profile, real PathInfo action reuse

On combined HEAD with source commit `fb7d28d4`, the corrected
`cargo test -p mantle --bin mantle
test_only_profile_without_rust_cache_blocks_cache_and_reuses_pathinfo_action_result
-- --nocapture` passed **1/1; 2551 filtered**. An earlier invocation
with `--exact` reported **0 tests** because the test is nested under
`tests::`; that invocation is not proof. In the *same* test-only Snix
profile with `rust_unit_cache: false`, the real `rust-cache serve`
entry point rejects before creating state, output, or receipts. The
test then publishes an actual signed PathInfo-backed ActionResult,
reopens the selected store, discovers the persistent local record,
verifies its detached signature, probes the output with
`reused_nar_bytes` equal to the measured NAR and
`transferred_nar_bytes == 0`, checks physical export, and rejects
the Rust-cache command again without any receipt directory. This is
not inferred from the separate real-Casita profile fixture.

## Corrected fixed-root optional and bounded profile rail

The corrected command reused the original *absolute historical* Snix
rail root only after verifying it was absent:

```sh
test ! -e /home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/fixture-rail-compare &&
env CARGO_TARGET_DIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/selection-target \
  TMPDIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/tmp \
  MANTLE_RAIL_FIXTURE_ROOT=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/fixture-rail-compare \
  nix develop -c sh -c 'env -u CRUNCH_CONFIG_DIR -u MANTLE_STORE_BACKEND LANG=C LC_ALL=C TZ=UTC SOURCE_DATE_EPOCH=1600000000 cargo test --test store_archive_cli admitted_backends_share_signed_core_gc_identity_and_profile_conformance_rail -- --exact --nocapture'
```

Observed **1 passed; 0 failed; 13 filtered**, with both
`BACKEND_CORE_RAIL snix` and `BACKEND_CORE_RAIL casita` after all
optional operations. Snix's new branches composed **two actual
read-only bases** and resolved distinct signed paths in each while
comparing base-file bytes, admitted and reopened a signed atomic
batch, actually imported an unsigned Nario path with explicit
`--trust-unsigned`, and published then restored a real Rust-unit
cache artifact after fresh reopening without including mutable
receipt material. Casita rejected overlay/unsigned/Rust-cache
preflights with stable blockers before opening bases or mutating
state, rejected a signed atomic 1,025-root batch with
`casita-batch-limit` and byte-identical state, then admitted exactly
1,024 roots and resolved representative PathInfos after reopening.
The independent **signed Nario importer** 1,024/1,025 durable-tree
regressions remain separately executed; the direct batch fixture
does not replace them.

The first preserved three-path 7ec GC execution IDs remain
`b3:36eb6141fa768f053c08258dc684365f28e908643a2e51bffcb63a0b7cf31491`
versus selected
`b3:c73dcda6e8949135b7d49298cd219c3845e8eb6e18653d60b8cd5bb9c8b90e95`.
No recapture or golden rewrite occurred. The original two-path
fixed-root execution ID matched exactly, and the shared signed
PathInfo/NAR/consumer facts matched. T3.3 is checked with the
combined same-profile ActionResult fixture above; T3.1 remains
unchecked under spec 193–197's literal every-positive-fixture golden
clause pending explicit scope interpretation. A full 14-case archive
suite under this pinned environment is recorded separately when it
finishes; one selected test is not itself T4.3's whole suite.

## Focused Mantle bin selectors, bounded partial receipt

An initial three-selector command was cut off by its **3,600-second
job deadline during compilation of the third selector**. Its first
two actual executions passed:
`cargo test -p mantle --bin mantle store_backend -- --nocapture`
**1/1; 2,551 filtered**, and
`cargo test -p mantle --bin mantle store_cmd -- --nocapture`
**6/6; 2,546 filtered**. There was no third-test result; timeout is
not a green Rust-cache selector. Only the unfinished
`cargo test -p mantle --bin mantle rust_cache -- --nocapture` selector
was launched again with pinned quota-safe `TMPDIR`, without rerunning
either already-passed selector. Its outcome is recorded after it
executes.

## Definitive full selected archive CLI suite

The full `cargo test --test store_archive_cli -- --nocapture`
completed **14/14 passed** on the combined source and test-only peer
commits, using the original absolute
`MANTLE_RAIL_FIXTURE_ROOT=.../fixture-rail-compare`, pinned quota-safe
`TMPDIR`, and unchanged baseline locale/time/signer environment
shown in the corrected command above. It included both default and
explicit Snix portable historical signed/GC facts, independently
verified same-key versus distinct-key Snix signatures, signed Nario
and native archive CLI admissions/rejections, Casita trust and unsigned
preflights, state-byte identity/migration, and the new **actual**
Snix/Casita optional-profile/bound one-rail fixture. Its
`PRECHANGE_SNIX_RAIL_GC_PLAN` lines still display the preserved raw
three-path numerical mismatch; 14/14 is not a claim that spec
193–197's literal every-positive-golden clause was resolved.

## Final post-cherry store-core and first-party non-Clippy gate receipts

With both test-only source cherries present, the exact scoped command

```sh
env CARGO_TARGET_DIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/selection-target \
  TMPDIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/tmp \
  nix develop -c cargo test -p crunch-store --lib -- --nocapture
```

passed **405/405** (`artifact://28785`). The output expressly
includes both signed Nario
`nario_casita_rejects_1025_new_roots_before_any_publication` and
`nario_casita_accepts_configured_1024_root_boundary` as passing,
Casita direct repair-library rejections, and GC over more roots than
one atomic mutation. This final pass supersedes the earlier
pre-test-only-cherry 405/405 receipt for the store package.

The first package-scoped `cargo fmt --check --package crunch-store
--package crunch-build --package crunch-rustc-wrapper --package mantle`
returned **1** solely for five layout/import-order differences in the
new `tests/store_archive_cli.rs` optional fixture. The exact
rustfmt-only changes were committed as `ca34d1c0` on top of the
behavior-tested fixture commit `2c6c13ab`; no assertion or source
logic changed. The same package-scoped check on `ca34d1c0`, with
`CARGO_INCREMENTAL=0`, pinned `TMPDIR`, and
`nix develop --offline --no-write-lock-file`, then returned **0**.
The first-party capability boundary checker returned **0** over
**558 files** with raw-service, writable-authority, and
handle-construction escape counts all **0**; `git diff --check`
returned **0**. Those receipts are not substituted for the
still-running strict first-party Clippy check.

On `2c6c13ab` under the explicitly pinned sibling Cairn policy
`/home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json`
(SHA-256
`1501c8c5a387098987d9feef3869e155b47748ae973785ae8f7aee3cfd92bbe9`),
`nix run --offline --no-write-lock-file
path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
--policy <absolute>` returned **0**, `"issues":[]` and
`"change_issues":[]`. Proposal, design, and tasks gates for
`add-store-backend-selection` under the same policy each returned
**0**, `"valid":true`, `"verdict":"PASS"`, `"issues":[]`; tasks
reported **16 done/3 open** (T3.1, T4.3, T4.4), not archive
readiness. A nonfatal upstream `git.onix.computer` HTTP 530 warning
in some invocations does not replace these observed explicit-policy
gate results. The layout-only follow-up did not modify Cairn
change files. At this stage, T4.3 still awaited focused runtime and
strict first-party lint receipts, recorded below.

## Strict touched-first-party Clippy after vendored-path red

The source quality owner executed the exact scoped command:

```sh
env -u CRUNCH_CONFIG_DIR -u MANTLE_STORE_BACKEND \
  LANG=C LC_ALL=C TZ=UTC SOURCE_DATE_EPOCH=1600000000 \
  CARGO_INCREMENTAL=0 \
  CARGO_TARGET_DIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/t43-first-party-target \
  TMPDIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/tmp \
  nix develop --offline --no-write-lock-file --command \
  cargo clippy --no-deps -p crunch-store -p crunch-build \
  -p crunch-rustc-wrapper -p mantle --lib --bins --tests -- -D warnings
```

Observed **exit 0**, finished after 4m12s, no touched-first-party
warnings (`artifact://28787`). It began at committed `2c6c13ab`,
and formatting-only `ca34d1c0` landed while dependencies compiled;
the precise source text rustc read was not independently captured,
but that follow-up changed only five rustfmt layouts/import grouping
in the tested archive fixture, not behavior. The scoped package
rustfmt check passed on final `ca34d1c0`. The earlier **36 vendored
`fuse-backend-rs` `clippy::io_other_error` errors** under the broader
command without `--no-deps` remain an explicit red nonclaim; the
broader Mantle bin's two Slurm/seccomp failures are likewise not
concealed or rerun. T4.3 is not checked until every remaining
focused runtime result is observed.

The source owner then cherry-picked the tested fixture and layout
commits onto the published source branch as `9c444755` and
`61bd4465` respectively. The **same exact strict first-party command**
above ran again on final published source HEAD `61bd4465` and exited
**0**, `Finished dev ... in 2m 32s`, with no first-party warnings.
This final-source run eliminates the prior detached run's uncertain
layout-only rustc-source timing. On the same published HEAD, the
four-package rustfmt, 558-file zero-escape checker,
`git diff --check`, pinned-policy Cairn validate and
proposal/design/tasks gates all passed, the latter still reporting
**16 done/3 open** *before* the final evidence-only T4.3 task decision.

## Final original two-path exact historical root recheck

After both source/test-only cherries and the layout-only fixture
commit, this targeted original T1.1 comparison passed **1/1; 13
filtered**:

```sh
env CARGO_TARGET_DIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/selection-target \
  TMPDIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/tmp \
  MANTLE_BASELINE_FIXTURE_ROOT=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/fixture-state-compare \
  nix develop -c sh -c 'env -u CRUNCH_CONFIG_DIR -u MANTLE_STORE_BACKEND LANG=C LC_ALL=C TZ=UTC SOURCE_DATE_EPOCH=1600000000 cargo test --test store_archive_cli default_and_explicit_snix_preserve_prechange_signed_and_gc_golden_facts -- --exact --nocapture'
```

Both default and explicit `snix` iterations compare signed
PathInfo, NAR and store info/roots facts to the unchanged pre-change
golden. Because the physical root equals that recorded in the
original two-path T1.1 golden, the fixture additionally asserts
**exact numeric plan-ID parity**:
`b3:787fc14bbf1d8438cbd52cc1be068d68f62967b5634b5a495c938e24565d586c`.
It does not change the separate supplemental three-path first-7ec
raw-ID mismatch or authorize T3.1/T4.4 closure.

## Completed focused Mantle bin selector and T4.3 decision

The previously unexecuted third focused selector completed on
combined source:

```sh
env CARGO_TARGET_DIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/selection-target \
  TMPDIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/tmp \
  nix develop -c cargo test -p mantle --bin mantle rust_cache -- --nocapture
```

**4/4 passed; 2,548 filtered**, including the test-only profile
without Rust cache with signed PathInfo-backed ActionResult reuse, the
Rust-cache daemon's global backend/state binding and policy/receipt
CLI gates, and shared-cache trust requirement. The earlier focused
Mantle bin `store_backend` **1/1** and `store_cmd` **6/6** remained
passed; the initial compilation timeout had no third test execution.
Together with final post-cherry store-core **405/405**, archive CLI
**14/14**, GC CLI **17/17**, store integration **77/77** (the printed
sandbox-body skips are nonclaims), the exact original fixed-root
two-path result above and the strict touched-first-party quality gates
on published source `61bd4465`, the specified **focused T4.3 gate is
checked**. This is neither a blanket Mantle bin pass (two unrelated
Slurm/seccomp failures on the earlier broad run) nor a broad
vendor-inclusive Clippy pass (36 existing dependency errors), and
does not establish T3.1 or T4.4.

After checking only T4.3 and updating ADR 0082 while retaining its
`Proposed` status, the final pinned-policy task gate:

```sh
env TMPDIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/tmp \
  nix run --offline --no-write-lock-file path:/home/brittonr/git/OnixResearch/cairn#cairn \
  -- gate tasks add-store-backend-selection --root . \
  --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
```

returned exit **0**, `"valid":true`, `"verdict":"PASS"`,
`"issues":[]`, `"task_done":17`, `"task_todo":2`, for 19 task
checkboxes (receipt hash
`adc446e071cdbe5d8fc261cacfd5ace59bdadd6052eee752c0f4ec4682b76aa0`).
Only T3.1 and T4.4 remain unchecked; the Cairn gate explicitly
does not establish acceptance, archive, evidence truth, or release.

The same explicit-policy `validate --root . --policy <absolute>`
after the checkbox and ADR edits returned exit **0**, `"valid":true`,
`"issues":[]`, `"change_issues":[]` (receipt hash
`6f27893b2b17e1e64f76cb3d89f179b5ab444aefcc4afcc2bc59d49d322ea9fe`).
