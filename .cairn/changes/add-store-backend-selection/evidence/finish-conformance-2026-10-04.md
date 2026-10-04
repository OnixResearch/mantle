# Backend-selection conformance rail: scoped results (2026-10-04)

This note records **executed targeted tests only**, not a T4.3 workspace gate
or unconditional acceptance of T3.1. Code and historical fixture are from
published base `c5740ee6e220c41c16eaa2de988eaf6c489aea1b` plus the root
fixture additions in the isolated evidence branch. Historical goldens are
from *pre-selection* `7ec5177718a6950297e04eb4eb957a10b02e23ce` (see
`prechange-snix-golden-2026-10-04.{md,json}` and `finish-inventory-2026-10-04.md`).
The signer is the repository's explicit **TEST-ONLY/non-production**
`tests/store_archive_cli.rs:22-23` fixture; the alternate signer uses fixed
`[19_u8;32]` test bytes. Commands ran in `nix develop` with private target
and `TMPDIR` under `/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/`,
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
| Store archive export/import, second signer | Exports the retained root and its referenced child, imports into a fresh state of the same selected backend with an explicit signer policy, observes unchanged PathInfo facts; real `store sign` with a separately provisioned test key appends a second signature, visible in fresh `store info`. |
| Identity and mixed-backend negative | Fresh `mantle-store-state-v2` records the selected backend. A real CLI read with the other backend fails `store-backend-mismatch` and every recorded state-file byte remains identical. |
| Profile-dependent behavior | Snix reports unbounded `max_root_changes` and supports `rust-unit-cache` and overlays; Casita reports 1,024 and no Rust unit cache or overlays. Casita `--base-store` fails with `casita-overlay-unsupported`, with all recorded state-file bytes unchanged. This checks the declared bound, **not** the 1,024/1,025 admission behavior. |
| GC candidates, root transition and stale execution | Pinning the retained root keeps its referenced child while leaving exactly one unretained candidate. Pinning that candidate after the first plan makes `--execute --plan-id` fail with the declared `stale-gc-plan` under Snix or `gc-plan-stale` under Casita; exported content remains. Replanning reports zero candidates. |

Exact successful test stdout:

```text
BACKEND_CORE_RAIL snix {"nar_sha256":"42a7f16a040111ab52d03cf78d7178101b4746c983a12551f07b305181f7024e","profile_max_root_changes":null,"retained_store_path":"0000000000068rbfd5hp8rbj5mn6jqbj-rail-retained","signed_count_after_store_sign":2,"stale_plan_blocker":"stale-gc-plan"}
BACKEND_CORE_RAIL casita {"nar_sha256":"42a7f16a040111ab52d03cf78d7178101b4746c983a12551f07b305181f7024e","profile_max_root_changes":1024,"retained_store_path":"0000000000068rbfd5hp8rbj5mn6jqbj-rail-retained","signed_count_after_store_sign":2,"stale_plan_blocker":"gc-plan-stale"}
```

## Other exercised root fixtures

- `tests/store_archive_cli.rs::independent_snix_stores_agree_on_signed_bytes_only_with_the_same_fixture_key`: **1 passed**. Two independent default/explicit Snix states with the same explicitly provisioned key have byte-identical signed PathInfo; a third with another explicitly provisioned key has identical unsigned PathInfo and NAR, different signed bytes, and each state verifies its own signer with `trusted_signatures=1/1`.
- `tests/transcript_cli.rs::transcript_real_child_reopens_selected_casita_and_rejects_dropped_selection_without_mutation`: **1 passed**. The actual Mantle child opened Casita with an outer transcript selection; an explicit conflicting child selector failed `store-backend-mismatch` and every state-file byte was unchanged. This does not depend on a fake binary or argv echo.
- `tests/store_archive_cli.rs::default_and_explicit_snix_preserve_prechange_signed_and_gc_golden_facts`: **failed before the explicit iteration** at the strict historical GC comparison. Same-key signed PathInfo, paths, NAR, `store info`, and roots matched first; first default-Snix GC observations had the same content but two blob-index observations arrived in the opposite order, producing a distinct plan identity. Both exact payloads and the plan-ID distinction are recorded in `prechange-snix-golden-2026-10-04.md`, the historical golden JSON, and `selected-snix-gc-observed-2026-10-04.json`. The test remains strict; neither plan ID nor order was normalized away to claim success. Fresh-state repeatability investigation is pending in the source branch.

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
no-fallback directions. Those separate fixtures must be run in the **combined
source + evidence tree** before counting the optional/bound branches toward
T3.1/T3.3; source locations alone are not passing evidence. The fixed-path
prechange-vs-selected GC plan parity and all T4.3 gates also remain open as
recorded above. No general correctness, durability, GC safety, sandboxing,
or release eligibility claim follows from this rail.
