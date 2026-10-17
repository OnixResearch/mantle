# Android prebuilt-source admission receipt (2026-09-30)

## ADR correction and baseline

`adr/README.md:88` already assigned accepted ADR 0079 to `0079-spacewasm-stable-evidence-libtest-json.md`. The proposal's missing Android ADR 0079 filename was a collision, **not** an unused decision slot. ADR 0086 was reserved by the bootstrap-source-pins worker; this local change initially drafted proposed ADR 0087. Integration retained the already published canonical [ADR 0089](../../../../adr/0089-admit-prebuilt-android-toolchain-through-fixed-output-sources.md) for Android source-v1. Its six-field source-v1 BLAKE3 preimage is distinct from this receipt's reviewed prebuilt-record-v1 metadata identity; renumbering the historical reviewed digest is not an acceptance decision. The original receipt describes its own local source snapshot only. Do not overwrite or renumber the accepted SpaceWasm decision.

Proposed [ADR 0110](../../../../adr/0110-separate-reviewed-android-metadata-from-source-identity.md)
records the distinct reviewed metadata decision for integration review; its
Proposed status does not turn this historical receipt into current named-API
verification or accepted lifecycle evidence.

Before implementing, a scoped search of `lib/`, `bootstrap/`, `tests/`, and `.cairn/` for `temurin|commandlinetools-linux|build-tools;|platforms;android-|android.jar` found Android mentions in the Cairn proposal and dependent APK-proof proposals, but no existing `lib/` toolchain source surface. The new Android module is additive; it delegates fetch effects to `builtin:fetchurl` and uses the existing source-bundle override path.

## Direct upstream archive observations

These four URLs were fetched directly into `/home/brittonr/scratch/mantle-android-admission/`, without accepting a guessed Nixpkgs pin. `sha256sum` and `b3sum` ran on the downloaded archive bytes. ZIP integrity checks and JDK tar listing succeeded. BLAKE3 archive-byte digests below are **different from** the separately recorded metadata-record BLAKE3 identities in `lib/android.ncl`.

| Upstream URL | Exact package version | SHA-256 archive | BLAKE3 archive bytes | BLAKE3 canonical metadata |
|---|---|---|---|---|
| `https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.17%2B10/OpenJDK17U-jdk_x64_linux_hotspot_17.0.17_10.tar.gz` | Temurin 17.0.17+10 | `992f96e7995075ac7636bb1a8de52b0c61d71ed3137fafc979ab96b4ab78dd75` | `6f01a83b28361755ac5948d9c05572345bd1aa4deee23ef8f471102fec94a6b2` | `5891c7afb04cf3c783a9294b9033ed22092ce83206c850af8bab79a6d2b49d35` |
| `https://dl.google.com/android/repository/commandlinetools-linux-13114758_latest.zip` | `Pkg.Revision=19.0` (archive revision 13114758) | `7ec965280a073311c339e571cd5de778b9975026cfcbe79f2b1cdcb1e15317ee` | `05a7e51fbc2cac3ca3f6ac0d2749476b0151b510e3890b9c4e1ce7d3ab83b41d` | `15f4b4e15251dc2832ecaedfbee5925237b9a7f5d8ded9d3860d0704cada35b1` |
| `https://dl.google.com/android/repository/build-tools_r35_linux.zip` | `Pkg.Revision=35.0.0` | `bd3a4966912eb8b30ed0d00b0cda6b6543b949d5ffe00bea54c04c81e1561d88` | `044d83ad8acdb4db798f5e8af1140d93e2574bee6d1b072b3e6916cedeb6585a` | `5bc394f89b8d8f36d1d916873b34021bb142db17107460da180e07c4d0b67e7f` |
| `https://dl.google.com/android/repository/platform-35_r02.zip` | API 35, `Pkg.Revision=2`, archive r02; `android-35/android.jar` present | `0988cacad01b38a18a47bac14a0695f246bc76c1b06c0eeb8eb0dc825ab0c8e0` | `859e2409518fb2b0f4c2d0bacedb4b5547865c739030865195326166857f977a` | `d77b395b209a6e6b2becf69a924c19ff2580fb2c2c4113a4b38aa8bb09e126cf` |

Temurin's upstream Adoptium release asset API for `17.0.17+10` independently reports archive SHA-256 `992f96e...78dd75`. Google's official `repository2-3.xml` lists SHA-1 `0bb560a90a7a2cbd0dd8348224d518b638fe7949` for `platform-35_r02.zip` and `2cfaa0bbb2336e9ec18ed3ecea84fa2e2af607bc` for `build-tools_r35_linux.zip`; direct `sha1sum` matched both. The three Google packages' `source.properties` and the platform archive's `android.jar` location were inspected. No upstream SHA-256 is inferred from SHA-1; all manifest SHA-256 values are direct archive-byte computations.

## Focused checked gates

- `nix develop --offline --command nickel typecheck lib/lib.ncl && nix develop --offline --command nickel export -f json lib/android.ncl --field manifest`: exit 0, `schema=mantle-android-prebuilt-sources-v1`, four records (current API revision corrected to package version 19.0).
- `TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-android-admission/target nix develop --offline --command cargo test -p crunch-eval --lib stdlib::tests::`: 10 passed, 0 failed, 72 filtered, exit 0. Embedded nested file is materialized by `write_stdlib`.
- Local `/home/brittonr/git/cairn/result-cairn/bin/cairn gate proposal|design|tasks adopt-android-prebuilt-toolchain-sources --root .` (each separately): each `verdict=PASS`, `issues=[]`, exit 0. This is an advisory gate, not archive readiness.
- Local `cairn validate --root .` returned exit 0 but JSON `issues` included unrelated `thin-cli-composition-root/tasks.md:88 task I7 is done before predecessor I6`; therefore do **not** report global validation issue-free.
- Isolated detached `origin/main` worktree `/home/brittonr/scratch/mantle-android-isolated` overlaid only `lib/android.ncl`, `lib/android/sources.ncl`, `lib/lib.ncl`, `crates/crunch-eval/src/stdlib.rs`, and `tests/android_sources.rs` (not concurrent root changes). `TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-android-admission/isolated-target nix develop --offline --command cargo test -p mantle --test android_sources -- --nocapture`: `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`, exit 0. Included a real file:// source bundle export/import/pin/preflight/build after deleting the original payload; the selected output file was byte-equal to `android offline fixture bytes`. An unmatched file:// URL failed before fetch with `ready_class=missing`, one `missing_records` entry, `next_actions[0].blocker_class=missing-source-state`, and empty `network_required_records`. Forced embedded child `mantle eval` with `CRUNCH_FORCE_EMBEDDED_STDLIB=1` resolved `android.ncl` and nested `android/sources.ncl`. SHA-256/BLAKE3 metadata drift, unsafe archive paths, missing/placeholder fields, duplicate and overcount sources were rejected in the same suite.
- In the same isolated overlay, `TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-android-admission/isolated-target nix develop --offline --command cargo fmt --check -p mantle -p crunch-eval`: exit 0, no output.
- In the same isolated overlay, `TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-android-admission/isolated-target nix develop --offline --command cargo clippy -p crunch-eval --lib --no-deps -- -D warnings && ... cargo clippy -p mantle --test android_sources --no-deps -- -D warnings`: both `Finished dev profile` (1m34s, 2m13s), exit 0. This scoped result does not establish root-wide Clippy after other concurrent changes.
- After a fresh current-tree root `cargo check -p mantle --bin mantle --locked --offline` passed, shared checkout `TMPDIR=/home/brittonr/scratch CARGO_TARGET_DIR=/home/brittonr/scratch/mantle-android-admission/integrated-target NIX_CONFIG='eval-cache = false' nix develop --offline --no-write-lock-file -c cargo test -p mantle --test android_sources --locked --offline -- --nocapture`: `7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`, exit 0. This remains source-contract/offline-fixture proof, not reviewed JDK/SDK tool execution.
- Following Android and Cargo-unit example catalog/README additions, focused shared checkout `cargo test --locked --offline -p mantle --test examples_inventory`: `17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`, exit 0. An earlier attempt failed before tests because a previously copied vendored header in the private scratch target was mode 0444; making only that generated target writable fixed the pre-test environment error.

## Reviewed original-HTTPS physical consumer (build-tools only)

- With `S=/home/brittonr/scratch/mantle-android-admission/android-consumer-imtxna` and the existing private compiled `mantle` binary at `/home/brittonr/scratch/mantle-android-admission/integrated-target/debug/mantle`, the scratch root `S/reviewed-build-tools.ncl` is `android.fetch_source (android.reviewed_source "android-build-tools")`. `mantle --json --state-dir "$S/state" source bundle export --build-root "$S/reviewed-build-tools.ncl" --import-path lib --fetch-missing --to "$S/reviewed-build-tools-bundle.json"` fetched the manifest's **original HTTPS URL** `https://dl.google.com/android/repository/build-tools_r35_linux.zip`, not a `file://` alias. The one `fixed-url` bundle record has archive-byte SHA-256 `bd3a4966912eb8b30ed0d00b0cda6b6543b949d5ffe00bea54c04c81e1561d88`, 61,958,799 payload bytes, record identity `fixed-url-699050417b5f7f5c2b7d5faba42bcd81eafaa9fe85a2ed71678136e2302a3ec3`, and content BLAKE3 `51545a19a79e96170c8f483c412efcbb5f10965a28154e6ef9d1528e94adc9aa`.
- `source bundle import --from "$S/reviewed-build-tools-bundle.json" --pin` imported one pinned record (manifest BLAKE3 `8484eda9d1f78b5bf3e4c0ede93678a1a08f149171637b5f154e279a1ffb1dd6`); `source bundle verify --from ... --imported` and `source bundle preflight --build-root "$S/reviewed-build-tools.ncl" --import-path lib` both reported `ready`, no missing/stale/untrusted records. The consumer root preflight likewise reported `ready`, no network-required or unpinned records. The offline build used `--state-dir "$S/state" --store "$S/store" build "$S/reviewed-build-tools.ncl" --import-path lib --offline-source-preflight --no-substitute`: one built, zero failed. Its physical fetched output `$S/store/9rsbd7ajs1y9bn4a4rhvqya8hqx2zpc5-android-android-build-tools-35.0.0` has SHA-256 exactly `bd3a4966...e1561d88` and `cmp` was byte-identical to a separately downloaded official upstream archive.
- `S/reviewed-consumer.ncl` calls hard-coded `android.bind_tool "android-build-tools" (android.reviewed_source "android-build-tools")` on a native sandbox consumer. Evaluated input count is one `builtin:fetchurl` with the original HTTPS URL; bound environment names component `android-build-tools`, SHA-256 `bd3a4966...e1561d88`, and metadata BLAKE3 `5bc394f89b8d8f36d1d916873b34021bb142db17107460da180e07c4d0b67e7f`. The builder checks those values, locates the declared fetched input under `$NIX_STORE`, and copies its physical archive bytes with sandbox `/bin/busybox cat` into `$out`. With `SNIX_BUILD_BWRAP` set to the pinned Nix-shell bubblewrap executable, `mantle --json --state-dir "$S/state" --store "$S/store" build "$S/reviewed-consumer.ncl" --import-path lib --offline-source-preflight --no-substitute` built one, failed zero, and denied ordinary consumer network access. The FOD was reused from signed local action-result state (`trust_basis=["crunch-britton-desktop-1"]`, 61,958,912 NAR bytes reused). Consumer physical output `$S/store/4v17ajkzrgcjpfvs0jl45438khfr9wrh-reviewed-build-tools-physical-consumer` is 61,958,799 bytes, SHA-256 `bd3a4966...e1561d88`, and `cmp` was byte-identical to the fetched FOD file.
- `store info` reports fetched PathInfo `ca=Flat(Sha256 pin)`, NAR SHA-256 `9d68905675654706e6717a2a278f1f67871fb17b150034cdb537c25b8d3f3b46`, NAR size 61,958,912, one local `crunch-britton-desktop-1` signature. Consumer PathInfo reports `ca=Nar(Sha256)` with the same NAR digest and size, and one local signature. `store verify --signing-key "$S/config/signing-key"` on each exact path fragment passed `trusted_signatures=1/1`, `0 mismatches`; these are **local Mantle signatures**, not upstream code provenance. The first consumer attempt could not start because the bare host PATH lacked `bwrap`; explicitly selecting the available pinned Nix-shell bubblewrap repaired that environment prerequisite. No Android SDK executable was run; this proves fetched bytes were physically consumed by the reviewed binder, not APK correctness or all-four-component toolchain readiness.

- After the reviewed original-HTTPS physical consumer proof, local `cairn gate proposal`, `gate design`, and `gate tasks` for `adopt-android-prebuilt-toolchain-sources --root .` each returned `verdict=PASS`, `issues=[]`, exit 0; tasks reported `task_done=19`, `task_todo=0`. These advisory gates do not substitute for an APK runtime or archive decision.

The demonstrated trust boundary is source-record admission, offline fixture replay, and one original-HTTPS reviewed Android build-tools archive physically copied by an identity-bound native consumer with verified local PathInfo/NAR. No prebuilt JDK or Android SDK executable was run, no four-component signed APK was built, no upstream binary provenance or bootstrap-chain provenance is established, and this Cairn is not archived. APK-proof and release acceptance remain separate.

## Current named-API decision check (2026-10-10)

This section is additional evidence over merged Mantle parent
`05422d87ac1452f45dc9bae602a0248e3f21fc45`, not a rewrite of the
2026-09-30 snapshot above. The immediate outcome is acceptance of the
*reviewed metadata/finite source-v1 bridge* in ADR 0110; the maintained
capability is identity-bound, offline-replayable source admission for Android
tool consumers. Mantle Android source-admission maintainers own the reviewed
alias table and its drift fixtures. ADR 0089 remains a separate proposed
decision for the published source-v1 implementation. Any cohort change
requires checking both preimages, the pin equivalence and the byte path again.

- Isolated clean-parent worktree, locked Mantle Nix dev shell, private Cargo
  target: `cargo test -p mantle --test android_sources --locked --offline --
  --test-threads 1` reported `test result: ok. 9 passed; 0 failed`;
  `cargo test -p crunch-eval --lib stdlib::tests:: --locked --offline --
  --test-threads 1` reported `test result: ok. 11 passed; 0 failed`.
  Among the named cases: `reviewed_binding_declares_fetch_and_rejects_metadata_drift`,
  `reviewed_tool_binding_uses_one_validated_source_v1_fetch_for_each_alias`,
  `published_source_and_reviewed_metadata_cannot_cross_identity_domains`,
  and the offline unmatched-source rejection. Full captured log:
  `android-reviewed-adr0110-focused-fixtures-20261010.log`
  (BLAKE3 `b9f259604bf34066f4d346df55f9141d008ef5fd626e58e7fdc77cbf3279bc6b`).
- The locked `nix build .#default` packaged `mantle` from the same source
  cohort. `mantle eval --import-path lib` of an external
  `android.reviewed.bind_tool "android-build-tools"` native consumer had
  **one** fixed-output `builtin:fetchurl` input for the original
  `https://dl.google.com/android/repository/build-tools_r35_linux.zip`,
  pinned to published SHA-256 SRI
  `sha256-vTpJZpEuuLMO0NALDNprZUO5SdX/4AvqVMBMgeFWHYg=`. Its reviewed
  metadata identity was
  `5bc394f89b8d8f36d1d916873b34021bb142db17107460da180e07c4d0b67e7f`,
  not the source-v1 identity.
- `source bundle export --build-root reviewed-consumer.ncl --import-path lib
  --fetch-missing` returned `ready`, one `fixed-url` record and 61,958,799
  bytes. `source bundle import --pin` imported one record (manifest BLAKE3
  `cc8d22a2b17e86da6733316b2815ed1e530fe27588f29f6c01778ec26c887ba0`);
  `source bundle verify --imported` and exact-consumer `source bundle
  preflight` returned `ready` without missing or stale records. The export
  did **not** establish a new HTTPS transfer: available source bytes may
  have been reused. Logs:
  `android-reviewed-adr0110-original-https-source-export-20261010.log`
  (BLAKE3 `8ce63218ce4d5d55e4ce6b879249875085d01f0e99c47aae42d8b04bfbede259`)
  and `android-reviewed-adr0110-current-bridge-physical-consumer-20261010.log`
  (BLAKE3 `6aabb16ff104d20e365e941a5255e6657d53180a024a3dd12978a62929c45a7f`).
- `build reviewed-consumer.ncl --offline-source-preflight --no-substitute`
  used a pinned static BusyBox shell in bwrap to check the bound environment,
  then copied **only** the declared source-v1 `$NIX_STORE` archive into
  `$out`; `built_total=1`, `cached_total=0`, `failed_total=0`,
  `fod_mismatches=[]`. A separate `build canonical-build-tools.ncl
  --offline-source-preflight --no-substitute` selected that fixed-output
  input: `cached_total=1`, `failed_total=0`, `fod_mismatches=[]` after import.
  Its log is `android-reviewed-adr0110-canonical-source-physical-20261010.log`
  (BLAKE3 `728d99b3288b3abb6c11e42bc0ecd6f5cc65493be97ff48aaf260e8b27636b41`).
- Physical source
  `2zy2i572yas9gfpdz7ax1kjs09paal7v-android-build-tools-35.0.0` and
  physical consumer
  `41h22yl3dvgwcq2kyqbyzfi9nhmlxizv-reviewed-build-tools-source-v1-consumer`
  under `/home/brittonr/mantle-cairn-finish/proofs/android-reviewed-adr0110-20261010/store/`
  each had 61,958,799 bytes, `sha256sum`
  `bd3a4966912eb8b30ed0d00b0cda6b6543b949d5ffe00bea54c04c81e1561d88`,
  `nix hash file --type blake3 --base16`
  `044d83ad8acdb4db798f5e8af1140d93e2574bee6d1b072b3e6916cedeb6585a`,
  and `cmp` returned 0. `mantle store info` reported matching NAR SHA-256
  `9d68905675654706e6717a2a278f1f67871fb17b150034cdb537c25b8d3f3b46`
  and NAR size 61,958,912. `store verify --signing-key` for **each**
  exact path returned `trusted_signatures=1/1`, `0 mismatches`. Log:
  `android-reviewed-adr0110-local-pathinfo-verify-20261010.log`
  (BLAKE3 `7cb83dbce7b351d4c8601d6eb20686323c0d277505aa307f36429a55d0d7ac8a`).

The source and consumer PathInfo signatures are local Mantle store assertions,
not vendor signatures. A raw-file output's optional AST-grep evidence reader
reported `ast-grep-evidence-read-error`; the JSON build result and the
independent byte comparison above do not rely on that parser. No SDK or JDK
executable ran, no APK was signed, and this archive check proves neither
upstream provenance nor release eligibility. Repeatable positive and negative
routes live in `tests/android_sources.rs` and the embedded stdlib tests; the
external physical proof inputs and outputs remain in the isolated proof path
above rather than being added to the checkout.

### Current Cairn merge review

The accepted `.cairn/specs/android-adapter/spec.md` already contained all
three requirement IDs. The initial `cairn sync` preflight correctly rejected
two duplicate `ADDED` headings. The change delta now uses `MODIFIED` for all
three, retaining the existing source-v1 normative admission and binding
sentences and adding the distinct reviewed SHA-256/BLAKE3 metadata and finite
bridge constraints. The non-claim requirement was identical to the accepted
one. A new `cairn sync` dry run reported `blocked=false`, two modifications
`applied`, and the identical non-claim block `already_applied`; the archive
dry run reported `blocked=false`. These are **previews**, not executed sync
or archive receipts. The separate advisory `cairn readiness` report has
`completion=unavailable` without a supplied completion observation; it
does not authorize mutation.

Using the pinned Cairn executable from Mantle's flake cohort,
`cairn validate --root .` returned `valid=true`, empty global/change/spec
issues. `cairn gate proposal|design|tasks
adopt-android-prebuilt-toolchain-sources --root .` each returned `PASS`
with no issues; the tasks gate counted `19 done`, `0 todo`. Exact external
transcripts (BLAKE3) are:

| Check | Log under `/home/brittonr/mantle-cairn-finish/logs/` | BLAKE3 |
|---|---|---|
| Global validate | `android-reviewed-adr0110-cairn-validate-reconciled-20261010.log` | `c7c7db25b8c35bc41a4f503bffe29235d15f9e48d037bd4fd49c251fd98e59ab` |
| Proposal gate | `android-reviewed-adr0110-cairn-gate-proposal-reconciled-20261010.log` | `473413a7d03b8837c730e6f537645d7259356a4a569c618608261baeae32f9aa` |
| Design gate | `android-reviewed-adr0110-cairn-gate-design-reconciled-20261010.log` | `5d86598f452b1515b88c25b3b94672ae87c4e6c7347a7e6d8fba35b80c662046` |
| Tasks gate | `android-reviewed-adr0110-cairn-gate-tasks-reconciled-20261010.log` | `8f18f730d19b72c047aa69c3dc83a77ea825d3fad22f6ee97b0bf049611c1b63` |
| Sync preview | `android-reviewed-adr0110-cairn-sync-preview-20261010.log` | `dddf8c9f17968e7aa308f44ee31c32d20fe93ebdf86c54c8f7f4885019923513` |
| Archive preview | `android-reviewed-adr0110-cairn-archive-preview-20261010.log` | `2228ca60295f9b8776e9684a1ae2feecec02a29f7ef33b1cf4ca7c8ecdc4ed2f` |

### Merged-tree Mantle check before lifecycle cutover

On the isolated Android acceptance worktree, `nix flake check -L
--keep-going --no-write-lock-file` with only locked-source local-mirror
overrides and Nix store-space options exited 0 (detached task 794):
`running 25 flake checks...`, `all checks passed!`. Its first-party nextest
check reported `6262 tests run: 6262 passed, 150 skipped`; the named
`android_sources` positive/drift and `android_sources_offline` actual
fixed-output digest-drift rejections passed. This check ran on x86_64-linux;
the incompatible aarch64/darwin systems were not checked. Captured log:
`/home/brittonr/mantle-cairn-finish/logs/android-reviewed-adr0110-full-flake-corrected-20261010.log`,
BLAKE3 `ef74b093bde22d92acd219110c69f8af57adc247bfdf5a0311f0f6f8bdf6e848`.
The flake source filter excludes this *active* Cairn change delta and its
evidence; the separately captured pinned Cairn validation/gates and sync
preview above cover that surface. Archive-state checks require a fresh gate
after Cairn moves the change into the filtered archive.

### Executed Cairn cutover and postarchive validation

The implementation/evidence/ADR commit
`796cca191` preceded lifecycle mutation. `cairn sync
adopt-android-prebuilt-toolchain-sources --root . --execute` returned
`mutated=true`, `blocked=false`, and accepted Android spec hash
`fbff39d3ecf84a6f53ba2ff5ee8b8448acebc773948dda48ff48952b79918896`.
The source-v1 admission and binding requirements were modified with the
additional reviewed constraints; the prebuilt non-claim block was already
applied. Captured execution:
`android-reviewed-adr0110-cairn-sync-executed-20261010.log`
(BLAKE3 `ccf1512019e5c0265354b1f1beaa066941fea93376cf7f3cbce491418a7006f1`).

`cairn archive adopt-android-prebuilt-toolchain-sources --root . --execute`
returned `mutated=true`, `blocked=false` and moved the active change to
`.cairn/archive/1970-01-01-adopt-android-prebuilt-toolchain-sources/`
(archive receipt
`android-reviewed-adr0110-cairn-archive-executed-20261010.log`,
BLAKE3 `467b909e1dd9bfb669e51321d016dc7ccddde442837025306f58b98b82db0fee`).
The `1970-01-01` prefix is Cairn's chosen directory name for this change,
**not** the date of the 2026-10-10 proof. Live ADR and guide evidence links
were moved to that path. The downstream `add-apk-build-adapter` metadata still
names this predecessor; its readiness no longer reports
`change_dependency.unsatisfied_dependency`. It continues to report
`completion=unavailable` without a supplied completion observation. This
cutover does not archive the downstream change or prove real SDK execution.

Full captured postarchive `cairn validate --root .` output (exit 0) is
`evidence/cairn-postarchive-validation-2026-10-10.json`, BLAKE3
`44c5e8ea7035467970462dad87e144f27b6cff38b9833557ff57155293eb0f6b`.
Its `receipt_hash` is
`0ad2efb1f41c7f84b793f2ce0d8635c75cfeacd9f8ab62e53ce5f29a25cea9d2`;
it reports `valid=true`, `issues=[]`, `change_issues=[]`, and `spec_issues=[]`.
This is postarchive Cairn *structural* validation, not a product or release
claim. Recheck the filtered archive and updated guide with Mantle's full
flake gate after this transcript is in the source tree.
