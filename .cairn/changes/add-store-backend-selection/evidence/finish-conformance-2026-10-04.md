# Backend-selection conformance rail: scoped results (2026-10-04)

This note records **executed targeted tests only**, not a T4.3 workspace gate
or unconditional acceptance of T3.1. Source commits `91cf5e5d` and
`eb4b6874` on published base `c5740ee6e220c41c16eaa2de988eaf6c489aea1b`
were combined with authored evidence commits `01105c15` and `33709d4c`.
Historical goldens are from *pre-selection* `7ec5177718a6950297e04eb4eb957a10b02e23ce` (see
`prechange-snix-golden-2026-10-04.{md,json}` and `finish-inventory-2026-10-04.md`).
The signer is the repository's explicit **TEST-ONLY/non-production**
`tests/store_archive_cli.rs:22-23` fixture; the alternate signer uses fixed
`[19_u8;32]` test bytes. Commands ran in `nix develop` with private targets
under `/home/brittonr/.cargo-target/` and `TMPDIR` under
`/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/tmp`,
`LANG=C LC_ALL=C TZ=UTC SOURCE_DATE_EPOCH=1600000000`, and
`CRUNCH_CONFIG_DIR`/`MANTLE_STORE_BACKEND` unset. No pueue, push, original
checkout mutation, or source bootstrap was used.

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
| GC candidates, root transition, stale rejection and accepted execution | Pinning the retained root keeps its referenced child while leaving exactly one unretained candidate. Pinning that candidate after the first plan makes `--execute --plan-id` fail with the declared `stale-gc-plan` under Snix or `gc-plan-stale` under Casita; exported content remains. Replanning reports zero candidates. Unpinning the candidate and dry-running again produces an accepted plan whose execution completes without failed operations, removes the candidate export and PathInfo, and retains the pinned root and referenced child for fresh `store info` and trusted `store verify`. |

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

## Complementary fixtures and work still to prove

The single rail above deliberately does **not** repeat same-path optional
fixtures merely to inflate a matrix. Existing source tests cover Snix-only
same-backend overlays and unsigned admission, Snix and Casita atomic batches,
Casita's 1,024/1,025 batch bound, non-overlay fail-closed profiles, stale GC
state, and read-only identity-less legacy markers. Existing root tests cover
Snix repair success and Casita repair dry-run/execute rejection before state
access (`tests/integration.rs:1296-1335`), as well as Casita rust-cache CLI
rejection (`tests/store_gc_cli.rs:700-742`). Source-peer additions cover a
synthetic no-Rust-unit-cache profile through a real command and both local
no-fallback directions. The other source fixtures must still run in the
**combined source + evidence tree** before their optional/bound branches count
toward T3.1/T3.3; source locations alone are not passing evidence. The
fixed-path prechange-vs-selected GC plan parity now passes, but all T4.3
workspace gates remain open. No general correctness, durability, GC safety,
or release eligibility claim follows from this rail.
