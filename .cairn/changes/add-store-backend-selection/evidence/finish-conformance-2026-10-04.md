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
three-path non-claim, not a favorable-retry golden. The conformance
specification at `specs/store-backends/spec.md:181-189` requires parity
with goldens recorded **before** selection: the original T1.1 two-path
fixed-root capture has exact old-vs-selected plan-ID equality, whereas
the separately captured additional three-path rail has matching signed
and canonicalized consumer facts but **different** old and selected raw
plan IDs. Its numerical disagreement alone does not prevent T3.1
acceptance, and no later 7ec traversal may replace the first capture
to obtain a favorable numerical match. T3.1 remains open because the
parameterized rail has not yet run every declared optional capability
and bound fixture and each undeclared capability's fail-closed fixture.
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

The one backend-parameterized rail above exercises core operations and
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
no-Rust-cache Casita profile. Their exact executions and remaining
synthetic-profile gap are recorded below rather than inferred here.
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
