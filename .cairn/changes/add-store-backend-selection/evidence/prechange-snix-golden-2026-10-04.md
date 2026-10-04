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
that backend selection changed GC candidates. The source branch is adding
per-category deterministic ordering *before* hashing, with a scoped
invariant; a previously issued plan may then reject and require a new dry-run.
The original historical golden and the strict selected parity test stay
unchanged until a combined-tree run actually passes.
