# Historical Snix signed golden capture (2026-10-04)

This is an executed pre-selection fixture from commit
`7ec5177718a6950297e04eb4eb957a10b02e23ce`, checked out separately at
`/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/prechange-snix`.
The later published starting point for the selection branch was
`c5740ee6e220c41c16eaa2de988eaf6c489aea1b`; it is **not** the source of these
historical goldens. The historical `Cargo.lock` SHA-256 is
`b7f8bb33e1470665596a52e7b5b768824e2fb47989ae257784efca70adddf762`.

The reproducible capture harness is `evidence/prechange-snix-capture.rs`:
copy it into `tests/backend_baseline.rs` in the historical checkout. It uses
`tests/store_archive_cli.rs:22-23`'s existing `TEST_KEYPAIR`, an explicitly
provisioned **TEST-ONLY/non-production** Ed25519 signer `archive-cli-1`.
The public key is
`archive-cli-1:yKUSiqP9yaMSduDmGtw8U9iVVd/Coyv9csB1rjHtiRM=`; its
SHA-256 digest is
`324e9b62d066a2f63c8c0af21af910abfb75b6febb7fc7fff30091c869177721`.
No operator key was read or copied. Both fixture states explicitly receive
that test signing key before any CLI open. The fixture pins the logical store
prefix `/mantle/store`, two file NAR contents `kept NAR\n` and
`candidate NAR\n`, and a legacy root with `created_unix_s: 100`.

One executed focused command (separate Cargo target; no pueue or source bootstrap):

```sh
env CARGO_TARGET_DIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/baseline-target \
  TMPDIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/tmp \
  MANTLE_BASELINE_FIXTURE_ROOT=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/fixture-state-compare \
  nix develop -c sh -c 'env -u CRUNCH_CONFIG_DIR -u MANTLE_STORE_BACKEND LANG=C LC_ALL=C TZ=UTC SOURCE_DATE_EPOCH=1600000000 cargo test --test backend_baseline capture_prechange_snix_golden -- --exact --nocapture'
```

Observed: `test capture_prechange_snix_golden ... ok`; `1 passed; 0 failed;
0 ignored; 0 measured; 6 filtered out`, after a 5.43-second warm targeted
build. An earlier first cold build in a **different** isolated target ran and
passed the same historical test in 7m 40s, but used `tempfile::tempdir()`
with a random physical fixture path; it is not used for the fixed-path GC
identity comparison. A separate earlier attempt to reuse the current branch's
isolated target failed building `aws-lc-sys` with `PermissionDenied` on its
generated headers; the clean historical-only target recovered without
altering shared artifacts.

The **exact CLI stdout**, store paths, NAR SHA-256, serialized signed PathInfo
bytes (hex of `serde_json::to_vec(&PathInfo)`, including signatures), and
literal fresh `store-identity.json` bytes (hex) are retained in
`evidence/prechange-snix-golden-2026-10-04.json`. The `*_stdout` fields hold
original bytes as JSON strings, preserving newlines. The captured JSON's
SHA-256 is
`81d74a733f93f2521330907f2b3c06f968bb603a8e39436435899bfda963a46d`.
Selected historical results include:

- Keep: `0000000000070rb5dcnnavk9dijp6qb2-baseline-keep`, NAR SHA-256
  `0b9a1594d448b3eed7bfd85b6dc1110e4f3e4695fe027241ef90a99eba409bbb`.
- Candidate: `0006ax31ciln8vk1ccnnavk9dijp6qb2-baseline-candidate`, NAR SHA-256
  `33cbc97e0f392db6b3528967569d5399244e30a7cb76a3d110e54517896003ef`.
- GC `plan_id`: `b3:787fc14bbf1d8438cbd52cc1be068d68f62967b5634b5a495c938e24565d586c`;
  the full report and `retention_plan_id` are in the JSON. A fresh identity
  is `mantle-store-state-v1`, without a backend field; the post-selection
  identity is intentionally v2 with `backend: snix`.

**Portability boundary:** `store_gc_dry_run_stdout.reclaim_observations[].path`
and its execution plan ID depend on the physical fixture root. Exact GC plan
identity equality must be checked against a selected-backend fixture run at
the identical absolute root. Portable full-suite runs compare observation
paths *relative* to their fixture roots and all other GC facts, including the
retention plan identity. Neither normalized equality nor a passing build
alone proves the full T3/T4 proposal gates.

## First selected-Snix comparison: failure retained, not accepted

A first `c5740ee6e220c41c16eaa2de988eaf6c489aea1b` default-`snix` run
under that exact fixed fixture root, key, and environment matched the historical
signed PathInfo bytes, NAR hashes, store paths, `store info` facts after removing
the intentionally added backend/profile fields, `store roots`, and the fresh
identity's unchanged prefix and trust policy. The new identity correctly
changed schema to `mantle-store-state-v2` with `backend: snix`.
The run failed **before** the explicit-`snix` iteration on GC report equality;
it is not a passing T3.1 comparison. Exact direct-CLI selected GC stdout from
the failed fixture state is retained in `selected-snix-gc-observed-2026-10-04.json`.

The prechange `plan_id` was
`b3:787fc14bbf1d8438cbd52cc1be068d68f62967b5634b5a495c938e24565d586c`;
the selected `plan_id` was
`b3:e948819705335e9e9d9392b4db933266fcd98108f2bb0d3f1ef2ac8ea002403d`.
Both reports had the same
`retention_plan_id: b3:e6d147ef5cf689b0dd984dd77f478370f833f9ac5041f46fb4e109f741804709`
and the same observed paths, bytes, candidate, roots, and explanations.
The differing JSON field was the order of two `blob-index` entries in
`reclaim_observations`: the historical report listed `a8cf...` then `cf82...`;
the selected report listed `cf82...` then `a8cf...`. Both code versions
enumerate blob-directory entries via filesystem `read_dir`; the selected
`crates/crunch-store/src/gc.rs:1242-1267` hashes the observations *in that
order* into the execution plan ID. No operator documentation reviewed here
declares the display order itself semantic, but it **is** operationally bound
into the accepted plan ID; the comparison must not discard or sort it just
to make the test pass. A controlled source-branch rerun of the **prechange
7ec** fixture at the same key, environment, and state produced the selected
`b3:e948819705335e9e9d9392b4db933266fcd98108f2bb0d3f1ef2ac8ea002403d`
identity with `cf82...` before `a8cf...`; repeated selected runs also varied.
This is genuine pre-existing filesystem enumeration nondeterminism, not proof
that backend selection changed GC candidates. The combined source commit
`eb4b6874` canonicalizes each dead-blob category before observation hashing;
its focused explicit-order and changed-fact stale-plan regressions both passed.
The unchanged strict test then passed for **default and explicit Snix** in the
combined source/evidence tree (`13 passed; 0 failed` in `store_archive_cli`),
under the same fixture root, signing key, and environment. Both selected
dry-run plan identities equal the original `a8cf..., cf82...` historical golden
`b3:787fc14bbf1d8438cbd52cc1be068d68f62967b5634b5a495c938e24565d586c`;
the original golden JSON and the failing selected-observation artifact remain
unchanged. An old in-flight accepted plan ID based on the alternative unsorted
observation order can safely reject as stale; operators must re-run dry-run
and accept its current plan ID.

## Supplemental historical rail fixture

The original `prechange-snix-golden-2026-10-04.json` above is **unchanged**.
The added `capture_prechange_snix_rail_golden` in the same historical harness
records *the same `rail-*` fixture as the selected-backend conformance rail*:
three file NARs, a retained output referencing a child, a separate unretained
candidate, the same explicitly provisioned test-only signer, archive export
and fresh-state import, second independent signer with real verification,
fresh builder-store closure and `ActionResultPort` reuse, and a plan-bound GC
transition. It seeds a legacy logical retained root with
`created_unix_s: 100`, so root timestamps do not perturb retention identity;
candidate pin/unpin and stale-plan rejection remain actual CLI operations.
The historical checkout uses a temporary `tests/backend_baseline.rs` that
includes this harness; it is **not** a production source change.

Executed historical capture at revision `7ec51777` with a fresh exact root:

```sh
test ! -e /home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/fixture-rail-compare &&
env CARGO_TARGET_DIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/baseline-target \
  TMPDIR=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/tmp \
  MANTLE_RAIL_FIXTURE_ROOT=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/fixture-rail-compare \
  MANTLE_RAIL_GOLDEN_OUTPUT=/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/source/.cairn/changes/add-store-backend-selection/evidence/prechange-snix-rail-golden-2026-10-04.json \
  nix develop -c sh -c 'env -u CRUNCH_CONFIG_DIR -u MANTLE_STORE_BACKEND LANG=C LC_ALL=C TZ=UTC SOURCE_DATE_EPOCH=1600000000 cargo test --test backend_baseline capture_prechange_snix_rail_golden -- --exact --nocapture'
```

Observed `capture_prechange_snix_rail_golden ... ok` (**1 passed, 0 failed,
7 filtered**). The new, separately keyed
`prechange-snix-rail-golden-2026-10-04.json` captures the exact serialized
signed PathInfo bytes for all three rail paths, their NAR SHA-256 and size,
physical exported bytes, reopen/closure/reuse, archive-import PathInfo facts,
the two-signer result, the complete dry-run and fresh GC reports, the
candidate pin/unpin transition, and accepted execution facts. The **first**
executed supplemental capture is retained byte-for-byte as
`prechange-snix-rail-unsorted-observed-2026-10-04.json` (SHA-256
`e0810f74d24b727ef6e3128bfc574607a18f57c50c5f387225966eb5d5ba768c`);
it reported index order `4df6, e6a6, d1f4` and plan ID
`b3:36eb6141fa768f053c08258dc684365f28e908643a2e51bffcb63a0b7cf31491`.
The supplemental `prechange-snix-rail-golden-2026-10-04.json` is another
**actual** old-source capture, not a rewritten report; its SHA-256 is
`4b65f9f0cc4856236a9c0775e866011a0cd46487ff8bf10ec6de1c2849d5eab6`,
its index order is `e6a6, 4df6, d1f4`, and its old execution ID is
`b3:f8c01c8b584203afe4f05b68b5ac1b0c4ac3d09a03292d1f52990b16022ed4c9`.
Neither this rail artifact nor the original T1.1 golden is silently replaced.
`selected-snix-gc-observed-2026-10-04.json` still retains the original red
selected-Snix observation from the separate two-path baseline.

The initial capture and **every executed rerun** are listed below. Each
historical `capture_prechange_snix_rail_golden` invocation passed; its complete
JSON was inspected before disposable trial outputs were removed. `R` is the
fixed `/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/fixture-rail-compare`
root; `A` is that path with `-alt` appended. The order codes are the **actual
ordered `blob-index` reclaim observations**, abbreviated by digest prefix:
`I=4df6,e6a6,d1f4`; `II=e6a6,4df6,d1f4`;
`III=d1f4,4df6,e6a6`; `IV=d1f4,e6a6,4df6`.
The old `plan_id` codes for root `R` are
`P1=b3:36eb6141fa768f053c08258dc684365f28e908643a2e51bffcb63a0b7cf31491`,
`P2=b3:f8c01c8b584203afe4f05b68b5ac1b0c4ac3d09a03292d1f52990b16022ed4c9`,
`P3=b3:ddf49d71ea9630171bf3ee39b56beddd78aaa0fc72f0bceca2647859eb63c966`,
and `P4=b3:aa1ed5bc08fddb7d82e9e27df64bb12fab92dda2d3f2aea84d67e6df1b8c7001`.
For root `A`, `Q1=b3:e8dd885ae93dabf39209fa20d360bbbe68294bb51b6eac3513ebcff53e1cc516`,
`Q2=b3:2de0e57585a23ce71d2d710d2c8b126db3319129c48dd429f079a7e2b5a9f8b5`,
and `Q3=b3:d1b900a19c06f6da243793036a021e5550577c00068ec4896ceba2b740cbd0ef`.

| Historical capture in execution order | Root | Index order | Dry-run `plan_id` |
|---|---|---|---|
| First, retained `unsorted-observed` artifact | R | I | P1 |
| Rerun before admission-order adjustment | R | I | P1 |
| Rerun retained as supplemental golden | R | II | P2 |
| Trial 1 | R | III | P3 |
| Trial 2 | R | III | P3 |
| Trial 3 | R | I | P1 |
| Trial 4 | R | I | P1 |
| Trial 5 | R | IV | P4 |
| Trial 6 | R | III | P3 |
| Trial 7 | R | II | P2 |
| Trial 8 | R | I | P1 |
| Trial 9 | R | II | P2 |
| Trial 10 | R | IV | P4 |
| Trial 11 | R | III | P3 |
| Trial 12 | R | IV | P4 |
| Trial 13 | R | IV | P4 |
| Trial 14 | R | I | P1 |
| Trial 15 | R | I | P1 |
| Trial 16 | R | II | P2 |
| Trial 17 | R | I | P1 |
| Trial 18 | A | I | Q1 |
| Trial 19 | A | IV | Q2 |
| Trial 20 | A | III | Q3 |
| Trial 21, already running when recaptures were halted | R | II | P2 |

Some fixture seeding and admission orders were varied symmetrically in the
historical harness and selected rail while investigating the old
`read_dir` traversal. The final fixture restores the **first capture's**
`child, candidate, retained` constructor/admission order. No retries remain
in the comparator: a permanent test consumes the retained original
supplemental golden. Prechange 7ec plan IDs are **nondeterministic** because
blob directory traversal order entered the hash. The selected source fix
`eb4b6874` sorts dead `blob-index` and `blob-chunk` paths separately before
hashing; the historical comparison therefore canonicalizes **only those
two old-source category runs**, preserving all paths, category boundaries,
bytes, blockers, candidate and retention facts. It demands selected output
already be in canonical order. At an identical physical root it also checks
an exact numeric plan ID if that old capture's order was already canonical;
otherwise it records both real IDs as distinct, rather than claiming all
prechange runs had the selected canonical ID. A different physical root
never licenses a numeric execution-ID comparison.
