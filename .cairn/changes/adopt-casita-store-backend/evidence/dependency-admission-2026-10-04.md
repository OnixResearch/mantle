# Bounded dependency admission: 2026-10-04

Branch `work/adopt-casita-store-backend-20261004` from selection commit `d6f3ca7754d40ffb1f898f2ec0853a44d2e387d2`. Only the isolated Mantle clone was modified. This is a pin/generator admission check, **not** a clean-source vendor build, complete toolchain refresh, or passing dependency gate.

## Source observations

- `crates/crunch-store/Cargo.toml:16` already pins Casita to `90404fcb1cfb3d83f2233715448dfefe913f5fd1` with default features disabled and exactly `native`, `experimental`. `Cargo.lock` contains Casita 0.1.0, nine Turso packages from `dca55133caa690f90dcdd58d3c4329fb0703659c` at 0.8.0-pre.7, `astral-tokio-tar` 0.6.4, and exactly one `blake3` at 1.8.2. The existing Nix vendor derivation overrides the pinned checkout with the tracked Casita patch, and `scripts/vendor-deps.py` generates the ignored closure without replacing an existing directory.
- At the first bounded read, `bootstrap/rust-source-plan.ncl` and `bootstrap/rust-source-musl-host-plan.ncl` still selected **1.94.0** below Casita's 1.94.1 `rust-version`. Both checked-in final profiles have since been refreshed to 1.94.1 (see below); source-built compiler requalification remains unrun.

## Exercised checks

1. Before the guard change, a scratch copy of `Cargo.lock` with only `turso_core`'s source revision replaced by `deadbeef` passed `scripts/vendor-deps.py::check_pins()`: `BUG: tampered turso_core source admitted`. The real checkout lock was never modified. After the guard change, the same scratch mutation was rejected: `tampered Turso rejected: turso: Cargo.lock version or source drift`. The unchanged lock passed: `baseline accepted: 987 locked package identities`.
2. A scratch `vendor-deps/` with a `user-data` file made `scripts/vendor-deps.py generate` fail before Cargo ran: `vendor-deps exists; refusing to clobber user data (run check, or move it aside yourself)`. The file remained byte-identical (`preserve`). The real clone has no `vendor-deps/`; no generator run or offline Cargo resolution is claimed.
3. `nix eval --offline --no-write-lock-file --raw .#checks.x86_64-linux.casita-vendor-closure.drvPath` exited 0 and printed `/nix/store/jd10wgpk79p74drd2nhpzbylkvsf1jbs-vendor-cargo-deps.drv`. This evaluates the derivation, **not** the clean-source build or a compile. Neither the Nix closure nor the patched default dev shell was built in this run.
4. A bounded read of the two Rust source plans printed `final_version = "1.94.0" compatible: False` for each against the 1.94.1 floor, and the fetched profile included `rust-1.94.1-x86_64-unknown-linux-musl.tar.xz`.
5. Before the four source admissions,
   `nix shell --offline nixpkgs#cargo-deny nixpkgs#cargo --command cargo-deny --config deny.toml check`
   exited 9: `advisories FAILED, bans ok, licenses ok, sources FAILED`. Its six
   `source-not-allowed` reports name packages from four other git repositories
   (`onix-artifact`, `bounded-tree`, `durable-file-publication`,
   `transactional-reconciliation-core`), not Casita/Turso. The advisory
   output contains RUSTSEC-2024-0370 (`proc-macro-error 0.4.12`, reached via
   Casita through `genawaiter`), RUSTSEC-2026-0253 (`lru 0.16.4`, via Snix),
   RUSTSEC-2026-0285 (`rustls 0.23.37`, including Casita's `object_store`),
   and yanked `chacha20 0.10.0` and `spin 0.10.0`. No waivers or policy bypass
   were made. This tool/advisory DB result does not replace the 2026-09-30
   vendor-rail transcript or establish post-change categories.

T1.1 manifest/lock and the complete admission delta are recorded below; the new admission guard checks the complete Turso git package family and singular lock pins in both Nix and the generator. T1.2 remains open for its clean-source Nix proof and source compiler requalification. T1.3/T4.7 remain open because the audit failed. T1.4's declared contract was reviewed, but its serial checkbox remains open while T1.2/T1.3 are open; this run does not assert the entire end-to-end backend contract.

## Complete original Casita lock admission inventory

Parsed `da00f584^:Cargo.lock` and `da00f584:Cargo.lock` with `tomllib` and compared `(name, version, source)` package identities and full records. The checked-in `Cargo.lock` has the same records as `da00f584:Cargo.lock`: 889 original and 987 current package records; 99 added identities, one removed identity, and 50 previously present identities with modified dependency lists only. The lock format and other top-level keys did not change. This lists the entire admission commit's lock delta, not an assertion that each registry record is exclusively used by Casita. The exact dependency-list edits and registry checksums remain reviewable with `git diff da00f584^ da00f584 -- Cargo.lock`.

The ten new locked git identities are `casita@0.1.0` from `https://github.com/cachix/casita?rev=90404fcb1cfb3d83f2233715448dfefe913f5fd1#90404fcb1cfb3d83f2233715448dfefe913f5fd1`, and `turso@0.8.0-pre.7`, `turso_core@0.8.0-pre.7`, `turso_ext@0.8.0-pre.7`, `turso_macros@0.8.0-pre.7`, `turso_parser@0.8.0-pre.7`, `turso_sdk_kit@0.8.0-pre.7`, `turso_sdk_kit_macros@0.8.0-pre.7`, `turso_sync_engine@0.8.0-pre.7`, `turso_sync_sdk_kit@0.8.0-pre.7`, each from `https://github.com/cachix/turso.git?rev=dca55133caa690f90dcdd58d3c4329fb0703659c#dca55133caa690f90dcdd58d3c4329fb0703659c`. The pinned Casita checkout declares `rust-version = "1.94.1"`, Turso `=0.8.0-pre.7` at that git revision, and optional `astral-tokio-tar = "=0.6.4"`.

The 89 added registry identities (name@version) are:

```text
aead@0.5.2 aegis@0.9.8 aes@0.8.4 aes-gcm@0.10.3 allocator-api2@0.4.0 antithesis_sdk@0.2.9 archery@1.2.3 aristo@0.4.1 aristo-macros@0.4.1 assoc@0.1.3 astral-tokio-tar@0.6.4 bao-tree@0.16.1 bigdecimal@0.4.11 binary-merge@0.1.2 bindgen@0.69.5 bitvec@1.1.1 branches@0.4.6 bytemuck_derive@1.12.1 cexpr@0.6.0 cfg_block@0.1.1 cipher@0.4.4 clang-sys@1.9.1 const-oid@0.10.2 crc32c@0.6.8 ctr@0.9.2
env_filter@2.0.0 env_logger@0.11.11 fastbloom@0.14.1 fastcdc@5.0.0 fs4@1.1.0 funty@2.0.0 genawaiter@0.99.1 genawaiter-macro@0.99.1 genawaiter-proc-macro@0.99.1 ghash@0.5.1 home@0.5.12 icu_collator@2.2.1 icu_collator_data@2.2.0 icu_locale@2.2.0 icu_locale_data@2.2.0 imbl@7.0.2 imbl-sized-chunks@0.2.0 inout@0.1.4 inplace-vec-builder@0.1.1 intrusive-collections@0.9.7 io-uring@0.7.15 iroh-io@0.6.2 lazycell@1.3.0 libloading@0.8.9 memoffset@0.9.1 minimal-lexical@0.2.1
nix-archive@0.6.0 nom@7.1.3 num-bigint@0.4.8 opaque-debug@0.3.1 owo-colors@3.5.0 pack1@1.1.0 pastey@0.2.3 polyval@0.6.2 positioned-io@0.3.5 proc-macro-error@0.4.12 proc-macro-error-attr@0.4.12 proc-macro-hack@0.5.20+deprecated radium@0.7.0 rand_pcg@0.3.1 rand_xoshiro@0.7.0 range-collections@0.4.6 rapidhash@4.5.1 roaring@0.11.4 rustc-hash@1.1.0 rustc_version_runtime@0.3.0 safe_arch@0.7.4 self_cell@1.3.0 sha1_smol@1.0.1 shuttle@0.8.1
simdutf8@0.1.5 simsimd@6.5.16 softaes@0.1.7 symlink@0.1.0 syn-mid@0.5.4 tap@1.0.1 tracing-appender@0.2.5 uncased@0.9.10 universal-hash@0.5.1 utf16_iter@1.0.5 which@4.4.2 wide@0.7.33 write16@1.0.0 wyz@0.5.1
```

The only removed identity is `astral-tokio-tar@0.6.3` (registry), replaced by the added `astral-tokio-tar@0.6.4`; `blake3@1.8.2` is unchanged and unique in the current lock. The 50 previously present identities with changed dependency lists (not changed versions or checksums) are:

```text
bumpalo@3.20.2 bytemuck@1.25.0 colored@3.1.1 cranelift-codegen@0.116.1 crunch-nar@0.1.0 crunch-store@0.1.0 crypto-common@0.1.7 der@0.7.10 digest@0.10.7 digest@0.11.3 errno@0.3.14 fs-set-times@0.20.3 fuse-backend-rs@0.12.0 getrandom@0.4.2 hashbrown@0.15.5 hashbrown@0.16.1 icu_locale_core@2.2.0 icu_normalizer@2.2.0 icu_provider@2.2.0 io-extras@0.19.0 is-terminal@0.4.17 jiff@0.2.23 malachite-nz@0.9.2 miette@7.6.0 nickel-lang-vector@0.2.0
nix@0.24.3 nix-compat@0.1.0 potential_utf@0.1.5 quinn@0.11.9 quinn-proto@0.11.14 quinn-udp@0.5.14 regalloc2@0.11.2 rsa@0.9.10 rustix@0.38.44 rustix@1.1.4 rustls-platform-verifier@0.6.2 snix-castore@0.1.0 strum@0.26.3 tempfile@3.27.0 tinystr@0.8.3 tokio-stream@0.1.18 tokio-uring@0.4.0 tracy-client-sys@0.28.0 twox-hash@2.1.2 uuid@1.23.0 wide@1.7.0 winapi-util@0.1.11 winx@0.36.4 zerotrie@0.2.4 zerovec@0.11.6
```

An independent bounded Python assertion parsed the manifest and all three locks (`da00f584^`, `da00f584`, and the checked-in file), verified their complete record equality where expected, checked every listed identity against the calculated sets, verified exact source strings for all ten git additions and unique `astral-tokio-tar@0.6.4` and `blake3@1.8.2`, and printed: `PASS: exact Casita manifest pin; checked-in lock equals admission lock; all 99 additions (10 git, 89 registry), one removal, 50 modified records listed; astral and blake3 unique`. This is bounded lock bookkeeping, not a vendor closure proof.

The bounded `cairn gate tasks adopt-casita-store-backend --root .` check after marking T1.1 returned `verdict: PASS`, `issues: []`, and 9 done / 27 open tasks in advisory mode. It validates task structure, not vendor or implementation claims.

## Source-built Rust profile follow-up (same branch)

The Rust project distributor's pinned
[`rustc-1.94.1-src.tar.gz.sha256`](https://static.rust-lang.org/dist/rustc-1.94.1-src.tar.gz.sha256)
returned `4c142a625f12e3cdf716c68ae19f4f60d98ad1482627b08579b15838e95ad514  rustc-1.94.1-src.tar.gz`;
an HTTP HEAD on that archive returned status 200 and 651,725,417 bytes. This
independent official checksum, not a local guess or an unpacked tree hash, is
now in each typed Rust source plan. Both plans keep the 1.90.0 -> 1.91.1 ->
1.92.0 -> 1.93.1 bootstrap stages and replace the final source, stage ID,
source ID, URL, digest, and version together with Rust 1.94.1. The Rust
bootstrap patch-plan version admission now recognizes 1.94.1; the old 1.94.0
fixture remains as historical test input, not the checked-in final profile.

Using the already available Nickel 1.17.0 binary,
`nickel export --format json bootstrap/rust-source-plan.ncl` and the same
command for `bootstrap/rust-source-musl-host-plan.ncl` both exited 0. Reading
their evaluated JSON and checking the stage/source linkage printed:

```text
x86_64-unknown-linux-gnu final 1.94.1 sources 6 stages 5 sha256 4c142a625f12e3cdf716c68ae19f4f60d98ad1482627b08579b15838e95ad514
x86_64-unknown-linux-musl final 1.94.1 sources 6 stages 5 sha256 4c142a625f12e3cdf716c68ae19f4f60d98ad1482627b08579b15838e95ad514
```

That check asserted exactly one `rustc-final` stage, source ID
`rust-1.94.1`, stage ID `rust-1.94.1-final`, predecessor
`rust-1.93.1-stage1`, version 1.94.1, source URL and exact official SHA-256.
This is Nickel evaluation and shape/identity admission, **not** a successful
Rust source build or compiler bootstrap. The new Rust tests for the plan
deserialization/validator and patch planner were not compiled or run: the
required nightly dev shell has an uncached `rustc-dev-1.96.0-nightly` derivation
and unavailable configured remote builder (separately observed on the same
source flake). T1.2 still needs actual compiler requalification and a
clean-source Crane vendor build under the detached long-job workflow, which
the user has prohibited.

A targeted search in `bootstrap/`, `src/`, `tests/`, and `scripts/` found no production caller of the old `rust-1.94.0` source or `rust-1.94.0-final` stage IDs: remaining occurrences in `main.rs`, `rust_source_provider.rs`, and `source_toolchain_closure.rs` are inside `#[cfg(test)]` synthetic fixtures, and `rust_bootstrap_patch_plan.rs` retains historical fixture text. The checked-in plans and their new assertions use 1.94.1. No fixture was rewritten merely to erase historical input.

## Contract and destination-trust boundary review

For T1.4 specifically, `design.md:135-164` names the repository subdirectory, both root namespaces, both exact envelopes, payload-name digest, capability profile, and trust policy/set; `design.md:360-422` declares admission/read verification order and the fence/recovery record; `design.md:460-500` declares every blocker in the task catalog (and `casita-repair-final-nar-unsupported`). This is a complete *declared contract*, not a passing end-to-end adapter check. T1.4 stays open in the serial sequence behind the unproved T1.2 and failed T1.3 gates.

The proposed T1.4 layout and blockers are declared in the active design
(`design.md:124-172,461-501`) and ADR 0082. Source review found
`CasitaStore::open` validates `casita-trusted-public-keys` before opening the
repository at `<state-dir>/casita`, uses in-memory Snix blobs and temporary
directories, and `verify_signer` rereads the destination policy for each
PathInfo verification; `backend.rs` excludes repair, overlay composition,
unsigned admission, and Rust unit cache in the Casita profile while keeping
PathInfo-backed action-result outputs core. CLI and library selection both
reject overlays; the CLI denies unsigned flags before state access. In Casita
mode `bootstrap --fetch` loads the same durable default signer as `mantle build`
after backend identity preflight, while Snix keeps its per-run signer. This
review does not establish all T2.1–T2.3 runtime scenarios, so their task
checkboxes remain open. The castore-only Rust cache remains intentionally
unsupported until the separate parity gate is satisfied.

A bounded **existing prebuilt binary** CLI smoke (the binary was not built
from the current commit) used a fresh Casita state directory, a key in an
explicit external `CRUNCH_CONFIG_DIR`, and a `file://` fixed-output source.
With no destination policy, `--json build --no-substitute` exited 1 with
`casita-signer-untrusted`, left no state-dir `signing-key`, and created no
policy. After provisioning that public key into the destination policy, a
second build exited 0 with `built_total=1`, and a fresh `store info` succeeded.
After removing the policy, a fresh `store info` exited 3 with
`casita-signer-untrusted`; the external key remained byte-identical. This is
one production-binary behavioral probe for the trust boundary, not a run of
the newly added `tests/integration_build.rs` fixture nor proof that the current
source compiles. The fixture additionally checks the empty root listing after
rejection and unchanged physical content after revocation. The test is still
uncompiled because the checked-in nightly shell is unavailable as above.
An already cached standalone Rust 1.97.1 compiler parsed the three touched
Rust files (`src/rust_bootstrap_patch_plan.rs`,
`src/source_toolchain_closure.rs`, `tests/integration_build.rs`) using
`RUSTC_BOOTSTRAP=1 rustc --edition 2024 -Z unpretty=normal --crate-type lib`
with zero parse errors. This is **syntax only**: no type checking, linking,
test execution, or current-source binary build is claimed.

## No-waiver source-policy and advisory follow-up

`deny.toml` now adds only four previously unadmitted Git origin URLs:

- `ssh://git@github.com/OnixResearch/onix-artifact.git` at
  `c932138d880ddf4c2967f4c024b489b5c0022bf1` (two artifact-auth packages).
- `https://seed.radicle.garden/zqhtZvsteJhxCJE96dMAZSZ9y1PX.git` at
  `b0fd0103bc9eed2c1b6d852045959462d105d8f1` (two bounded-tree packages).
- `https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git` at
  `951c27f59003cea9bfdb40ed4d89653d50fada1f` (durable-file-publication).
- `https://seed.radicle.garden/z4Tky6zvC8w4Y6c4YBzNxVbq5n752.git` at
  `606489b5f40298181214bb76bc3457b607f225d9`
  (transactional-reconciliation-core).

Each has a separate exact-revision `flake.nix` source-input/Cargo
manifest/lock-source assertion (`flake.nix:162-300`), so `allow-git` does not
serve as a floating-revision authorization. A bounded Python positive/negative
fixture parsed each manifest and all six relevant lock entries, compared exact
`?rev=...#...` strings and the independent Nix guard strings, and rejected an
alien policy URL and changed revision in memory. Output ended
`PASS all four reviewed Git source policy fixtures`. This is static policy/pin
admission, not a run of Cargo Deny or Nix build.

The requested post-change
`nix shell --offline nixpkgs#cargo-deny nixpkgs#cargo --command cargo-deny --config deny.toml check`
scheduled four uncached derivations:

- `/nix/store/dp031zrwj47zprpjf99v6csvg5c01drn-source.drv`
- `/nix/store/mfkbl31bv2vg9iip181hzj60jlm597rs-cargo-deny-0.20.2-vendor-staging.drv`
- `/nix/store/cj722h87z9w2irp1s5z67mw3vk6x2zyx-cargo-deny-0.20.2-vendor.drv`
- `/nix/store/5p3ka6524czcrsvpmfh1qaa80m83ph4k-cargo-deny-0.20.2.drv`

Nix reported `failed to start SSH connection to 'aspen1.local'`. The job
was canceled immediately per the no-long-build instruction; no audit binary
ran and **there is no post-change exit code or measured advisory/source
category result**. The last completed pre-change Cargo Deny result remains
exit 9 with advisories/sources failed and bans/licenses okay. Four exact
source admissions are expected to remove those six old source-policy
findings, but that is unverified.

The locked dependency path is:

```text
casita@90404fcb -> bao-tree@0.16.1 -> genawaiter@0.99.1
  -> genawaiter-proc-macro@0.99.1 -> proc-macro-error@0.4.12
```

It is explicit in `Cargo.lock:944-980,570-585,3043-3072,6708-6719`; the
cached `genawaiter-proc-macro` manifest requires `proc-macro-error = "0.4"`
and the local RUSTSEC-2024-0370 advisory marks it unmaintained with
`patched = []`. Keeping the exact mandated Casita revision, rejecting
waivers/forks/vendor edits, and requiring current-source proof for a
dependency graph update leave no proven no-waiver fix. An approved reviewed
upstream pin/dependency replacement plus compiler validation, or a policy
exception, would be required to close that advisory; none was chosen here.
T1.2/T1.3/T4.7 remain open and no lock entry, Casita revision, or waiver changed.

## Detached Rust prerequisites and boundary checker integration

The user subsequently authorized detached pueue builds. Pueue task **254**
(`mantle-rust-1941-devshell-local-20261004`) began from HEAD
`36f199edbbb2fec58f67ab9a3c2e77525414c719` with
`nix develop --offline --no-write-lock-file --option builders '' --option max-jobs 2 --option cores 4`.
Its dedicated log is
`/home/brittonr/.cargo-target/mantle-rust-script-pin-20261004/runs/devshell-rust-1941-20261004/run.log`.
The last observed pueue status was **Running**; the log showed local builds
including `rustc-dev-1.96.0-nightly` and, incidentally, the old-pin Casita
checkout. This is a nightly shell prerequisite, **not** Rust 1.94.1 compiler
requalification or final-pin Casita vendor evidence. Local-only builders
avoid the previously observed `aspen1.local` SSH failure.

Independent pueue task **255**
(`mantle-rust-1941-source-archive-20261004`) completed successfully after
fetching the official Rust 1.94.1 source tarball; its log is
`/home/brittonr/.cargo-target/mantle-rust-script-pin-20261004/runs/rust-1941-source-archive-20261004/run.log`.
The captured `sha256sum` is
`4c142a625f12e3cdf716c68ae19f4f60d98ad1482627b08579b15838e95ad514`,
matching both checked-in source plans and the Rust distributor's checksum.
Task 255 authenticates one source archive; it did not build a compiler.

The privately proven capability checker change was integrated as `e79d911f`
from private commit `2eb576fe`. On integrated HEAD, standalone nightly
`cargo -Zscript` with the repository's missing `clang` linker overridden by
`cc` ran `tools/check_store_capability_boundary.rs --self-test`, producing
`self-test: ok`. Its `--root .` run scanned 558 files and reported
`raw_service_escape_count=0`, `writable_authority_escape_count=0`,
`handle_construction_escape_count=0`, `casita_type_escape_count=0`, and
`casita_session_escape_count=0`. T4.1 stays unchecked: it is marked serial
behind unfinished earlier tasks and must be rechecked on the final subject.

Upstream pin research found `cachix/casita` main at
`6711e0c8741347e9a6f1e78cc0ad2c9d2750dfa4` still declares native
`bao-tree = "0.16"` and adds Turso/Chroma dependency changes, so bumping to
main alone does not establish a no-waiver fix. The most recent upstream
`n0-computer/bao-tree` tag is `v0.16.1`; its default `validate` feature pulls
optional `genawaiter 0.99.1`. Casita's checked-in pinned source calls
`bao_tree::io::fsm::encode_ranges_validated`, which is not the same as the
`#[cfg(feature = "validate")]` range-validation API. No maintained upstream
revision replacing Genawaiter while preserving that verified-streaming
contract was identified. Disabling `validate` solely to clear Cargo Deny
would need upstream contract review and negative corruption fixtures; no
feature, pin, source patch, lock entry, or waiver was changed.

## Independent upstream review, 2026-10-05

The [latest Bao-tree tag `v0.16.1`](https://github.com/n0-computer/bao-tree/tags)
is commit `2be9abd144783455606424424c29bd3a57f926f8`, also the
current upstream `main` commit. Its
[`Cargo.toml`](https://github.com/n0-computer/bao-tree/blob/2be9abd144783455606424424c29bd3a57f926f8/Cargo.toml)
still declares optional `genawaiter = "0.99.1"`, `validate =
["dep:genawaiter"]`, and a default feature set containing `validate`.
[Bao-tree PR #42](https://github.com/n0-computer/bao-tree/pull/42)
introduced Genawaiter to stream complete data/outboard validation. The
current [Casita `main` manifest](https://github.com/cachix/casita/blob/main/crates/casita/Cargo.toml)
still admits optional `bao-tree = "0.16"` through `native`; the current
Casita release list contains no Casita dependency release. Thus neither
the latest Bao-tree release nor Casita `main` provides a reviewed,
validation-preserving replacement for this locked advisory path.

The missing upstream prerequisite is a Bao-tree maintainer-reviewed
replacement of the Genawaiter-based complete validation stream, preserving
its public validation API and error behavior, with tests rejecting corrupt
data and outboards, followed by a Bao-tree release and Casita maintainer
adoption with its verified streaming contract exercised. An explicit
streaming state machine is one feasible upstream patch direction; it is
not an approved downstream implementation. Only after that reviewed
adoption can Mantle assess the new Casita revision, update its exact pin,
lock and Nix/vendor admission together, and run unwaived `cargo deny
check` plus functional verification. Merely disabling Bao `validate`,
replacing the pinned vendored source, or adding an advisory waiver would
not satisfy the required contract. This read-only review did not change
the Mantle dependency graph, execute Cargo Deny, or close T1.2/T1.3/T4.7.

## Isolated no-waiver source-port admission, 2026-10-05

This follow-up is on separate linked worktree
`/home/brittonr/.cargo-target/mantle-advisory-source-ports-20261005`
from `e7d91c31abab268852c09097b581e13c7a47d24b`; it is not an
integration, archive, StageX, or release run. The read-only upstream review
above was accurate when recorded; this later Mantle-owned cutover supplies
new evidence, not a published upstream fix.

1. Exact published crates.io source archive SHA-256 and publisher revision:
   Nickel vector 0.2.0, MIT,
   `36f243832286908d8873add24a905d6732ffabd6cfb2bf74cb18d667e892e279`,
   `f09fce4517c853a9845db13aa60d2b73405c799a`; SecretSpec 0.17.0,
   Apache-2.0,
   `68498f9695bb3662c157b8fd4b4665a594f1157de022ff5b0f891af4c7ec75d2`,
   `a8794e46ec9664a0e1a3869cc3105d0853937e48`. The manifest and
   per-original/per-patched-file receipt is
   `third_party/advisory-source-ports.json`, SHA-256
   `ff3c1885a880ce2c260bfabbada13457c82c72b674dfd8031947e963434a157e`.
   The vector patch SHA-256 is
   `9199b87fb9cc197c1a9c7cd5c099b10cfe0cae0c1c1f276bdad9b69e88ed85c4`;
   SecretSpec patch SHA-256 is
   `7fc086ecdc98d0d3e2b32c3f2ad367c661476f58ae3d409dc35c5df1dce9c98a`.
   `python3 scripts/import-advisory-ports.py check` verified each authentic
   archive and replayed forward/reverse with `patch --fuzz=0`. An initial
   hand-assembled SecretSpec hunk failed strict replay and was replaced by
   the source-derived exact hunk; no fuzzy patch was accepted.
2. The root path-port lock selects vector 0.2.0 ->
   `imbl-sized-chunks 0.2.0`, and SecretSpec 0.17.0 without default or
   `rsa-generation` features. It contains no `bitmaps 3.2.1`,
   `imbl-sized-chunks 0.1.3`, or `rsa 0.9.10`. In contrast, isolated
   **standalone** SecretSpec with `--no-default-features --features
   rsa-generation` compiled RSA 0.9.10 and passed
   `generator::tests::test_generate_rsa_default` (1/1), proving optional
   RSA still works for other consumers. Locked offline `cargo metadata`
   for that standalone package additionally resolved its **default**
   feature to `rsa-generation` and an `rsa 0.9.10` dependency. The same
   isolated package with `--no-default-features --features sops` passed
   `generator::tests::test_disabled_rsa_generation_fails_closed` (1/1)
   and `provider::sops::tests::test_sops_single_file_get_json` (1/1).
   The published crate archive omits the repository test schema, so the
   scratch-only test fixture used its publisher-revision
   `schema/resolution-report.schema.json`; no tracked port file changed.
3. Before the port, `crunch-eval` `tests::eval_merge` passed against chunks
   0.1.3; after, the same test passed against compiled vector 0.2.0 from
   `third_party/` and chunks 0.2.0 (1/1). Actual patched `mantle` CLI
   compiled from the locked root, resolved freshly SOPS/age-encrypted
   `bootstrap` and `rotation` fixtures through the SecretSpec SOPS provider,
   and accepted both validated Ed25519 signing and keyed-BLAKE3 verifier
   keys. A `generate = true` change to the otherwise valid manifest
   exited 3 with `manifest-secret-write-or-cache-forbidden` before provider
   access. Secrets were not printed or checked in.
4. Actual Nix
   `checks.x86_64-linux.advisory-source-ports`,
   `checks.x86_64-linux.casita-vendor-closure`, and
   `checks.x86_64-linux.casita-crunch-store-check` built offline with
   `--option min-free 0 --option substituters '' --no-link`. The first
   caught a missing `patches/` source-filter entry before its correction;
   the final clean-store-source build passed. The default `nix develop`
   `scripts/vendor-deps.py dev-shell-check` passed without overriding Cargo
   source config. Repository-owned `scripts/vendor-deps.py refresh` using
   a **private copied** original registry cache/index/Git input home kept
   the prior ignored closure in a separate backup; `check` matched a new
   locked offline generation, 53,023 entries and 752 distinct external
   package names. No generated vendor source or checksum was hand-edited.
   `cargo check --locked --offline -p crunch-store --config
   .cargo/vendor-config.toml` compiled the generated closure in a fresh
   private Cargo target. Its first attempt reused a private target from
   another source map and failed when zstd-sys/bzip2-sys tried to overwrite
   their own existing mode-0444 copied headers; the fresh-target run
   exited 0 without touching generated vendor files or shared caches.
5. A private cargo-deny home containing a copied **read-only** crates.io
   registry index, offline advisory database, and immutable Crane source
   map ran from this isolated worktree:

   ```sh
   nix develop --offline --option min-free 0 --option substituters '' \
     --no-write-lock-file path:$PWD --command env \
     CARGO_HOME=/tmp/mantle-advisory-deny-home-20261005 CARGO_NET_OFFLINE=true \
     /nix/store/xqnwl8qppzhr3dq5nzw2ly6yx4lz9xyr-cargo-deny-0.19.0/bin/cargo-deny \
     --locked --offline check --config deny.toml --hide-inclusion-graph
   ```

   **Exit 0**: `advisories ok, bans ok, licenses ok, sources ok`, zero
   advisory/license/source errors or `index-failure` warnings. Remaining
   warnings: `cfg_block` and `wu-manber` missing license fields, duplicate
   crate versions, yanked `chacha20 0.10.0` and `spin 0.10.0`. Root
   `deny.toml` waivers/bypasses were unchanged. Casita remains
   `90404fcb1cfb3d83f2233715448dfefe913f5fd1` (Apache-2.0),
   Turso remains `dca55133caa690f90dcdd58d3c4329fb0703659c` (MIT),
   and `blake3` remains exactly `1.8.2`.

The initial offline Cargo lock update used a scratch `CARGO_HOME` with
symlinks to the shared registry/Git cache. That operation was not
instrumented to prove the shared cache was read-only; do **not** claim
retrospective non-mutation. All subsequent vendor-generation, test, and
indexed audit inputs used privately copied original caches, isolated
scratch fixtures, or immutable Nix source replacements. Only this
isolated worktree's tracked source/lock/docs were changed. T1.3's
configured audit and T4.7's no-waiver advisory gate now have scoped passing
evidence. T1.2's clean-checkout/source-built Rust 1.94.1 and self-build
source-bundle qualification remain open; this does not claim full CI,
StageX, upstream approval, or release eligibility. Future upstream bumps
must reauthenticate exact package archives, review/replay zero-fuzz
patches and repeat the runtime, vendor, and configured audit proofs.

## Live combined workspace admission, 2026-10-06

The exact authenticated Nickel vector and SecretSpec ports above were
applied to the primary live workspace without cherry-picking a stale
root manifest/lock or replacing concurrently admitted CC, readiness,
coordination, and Android dependencies. The saved pre-port live lock
contained 986 package identities; the combined lock has 978. Exactly
ten old registry identities were removed (including the two original
crates, `bitmaps 3.2.1`, `imbl-sized-chunks 0.1.3`, `rsa 0.9.10` and
their orphaned dependencies), and two unchanged-version path ports
were added. Twenty-one retained package identities had dependency
edge changes, but no other package identity/version was changed.
Complete `cargo metadata --locked --offline` resolved **890/890**
packages across **54** workspace members, versus the original live
**899/899**; CC driver, service-readiness, live-state, and coordination
members were all present. SecretSpec selected exactly `["sops"]`.
The combined `Cargo.lock` SHA-256 is
`c24dfe66e47ccc19ad432d5ccf1530574f092c1a33d540bfb371bf3be6bc26b7`.
Casita revision `90404fcb1cfb3d83f2233715448dfefe913f5fd1`, Turso
revision `dca55133caa690f90dcdd58d3c4329fb0703659c`, and unique
`blake3 1.8.2` are unchanged.

With that staged root graph, actual offline Nix
`path:$PWD#checks.x86_64-linux.advisory-source-ports`,
`path:$PWD#checks.x86_64-linux.casita-vendor-closure`, and
`path:$PWD#checks.x86_64-linux.casita-crunch-store-check` built
successfully offline using per-invocation `--option min-free 0
--option substituters '' --no-write-lock-file --no-link`; the store
check additionally specified `--option max-free 0`. The final
store check built its clean-source Crane dependency closure and ran
`cargo check --locked -p crunch-store` (934 derivations announced).
Default `nix develop --offline ... path:$PWD --command python3
scripts/vendor-deps.py dev-shell-check` resolved exactly the
immutable patched Casita source and both selected ports. The
repository-owned generator refreshed the ignored `vendor-deps/`
from a **privately copied** Cargo archive/index/Git home without
clobbering its prior checkout: `check` regenerated and compared
**53,023** entries and resolved **752** external package names
offline; a fresh-target `cargo check --locked --offline -p
crunch-store --config .cargo/vendor-config.toml` passed. The
authenticated importer's `check` verified both published revisions,
all 85 imported source files, and zero-fuzz patch replay.

Configured `/nix/store/xqnwl8qppzhr3dq5nzw2ly6yx4lz9xyr-cargo-deny-0.19.0/bin/cargo-deny`
(SHA-256 `9e0d43e46b809981aba11b41d069bb330a0ad09c6af851aa2c508b2299e6d53b`)
against the same lock and a copied private registry index/advisory
database ran `--locked --offline check --config deny.toml
--hide-inclusion-graph` with **exit 0** and
`advisories ok, bans ok, licenses ok, sources ok`. The four historical
`deny.toml` exceptions remain; no additional waiver, source exception,
or disabled advisory class was introduced. Existing warnings about
duplicate versions, missing upstream license fields, and the
already-yanked `chacha20`/`spin` entries remain visible.

Actual combined Nickel evaluator library tests passed **83/83** using
nightly `cargo 1.99.0-nightly (3efb1f477)` and
`rustc 1.99.0-nightly (dc3f85158)` from the named local toolchain.
The host's configured `clang` linker was unavailable, so the host
Nickel/SecretSpec runs used an explicitly available `cc` linker in
fresh owned targets; source and global linker settings were not
changed. A standalone published SecretSpec source copy was compared
byte-for-byte with all **72** live imported manifest/source/test files
and used its separately authenticated publisher-revision test schema:
actual SOPS provider tests **32/32**, disabled RSA generation
**1/1**, and optional RSA generation for other consumers **1/1**.
The root graph nevertheless contains no `rsa`, `bitmaps`, or chunks
0.1.3. The combined Mantle binary was compiled in the default Nix
devshell, whose immutable source map applies the required reviewed
Casita `nar.rs` patch; attempting a bare original-Git-Casita host
binary build instead failed at its known unpatched BLAKE3
`finalize().as_bytes()` call, so the host compile is **not**
misreported as a production-path success.

Actual `sops 3.13.3` was present at
`/nix/store/9vlk37b1fgklzwl5bl9xb2dk8sn947ih-sops-3.13.3/bin/sops`
(SHA-256 `ea54a71221a11a57642459f66677a0e83b76032ea8d8110ffb7613a048004498`).
The rebuilt Nix-devshell combined CLI at
`/home/brittonr/.cargo-target/mantle-ports-nix-combined-target-20261006/debug/mantle`
(SHA-256 `3bd60481307964e726e4371ac8db00a5c09afe0690c2d7180174afa58829a50c`)
ran the selected real age-encrypted bootstrap/rotation and bounded
timeout/oversized-output negatives:

```sh
nix develop --offline --option min-free 0 --option max-free 0 \
  --option substituters '' --no-write-lock-file path:$PWD --command \
  nix shell --offline --option min-free 0 --option max-free 0 \
    --option substituters '' nixpkgs#sops --command env \
    CARGO_NET_OFFLINE=true \
    CARGO_TARGET_DIR=/home/brittonr/.cargo-target/mantle-ports-nix-combined-target-20261006 \
    cargo test --locked --offline -p mantle --test remote_credentials_cli \
      sops -- --include-ignored --nocapture
```

**3/3 passed**; the real encrypted test is explicitly ignored when
`sops` is absent, never silently accepted as passing. A separate
freshly encrypted SOPS/age CLI smoke created a bootstrap ticket,
rotated to a distinct verifier identity, revoked the old ticket, and
found no bearer in outputs or durable state. With `generate = true`
and an intentionally missing provider file, the parent CLI returned
status 3 without changing ticket state; the direct isolated worker
also returned status 3 with the specific
`manifest-secret-write-or-cache-forbidden` marker **before** touching
that missing provider. The parent emits its generic
`remote-service-secret-worker-provider-failed` error, so only the
direct-worker marker is claimed.

That real smoke first reproduced a preexisting worker failure:
the Go `sops` binary could decrypt directly but could not reserve
its page-summary virtual memory when inherited worker `RLIMIT_AS`
was 512 MiB. Three bounded trials each gave failure at **512 MiB**
and **768 MiB**; **1,024 MiB** passed but left only **3,364 KiB**
sampled virtual headroom; **2,048 MiB** passed with **265,448 KiB**
sampled headroom. Unbounded SOPS decrypt sampled **1,757,908 KiB
VmSize** against only **23,152 KiB VmRSS**. Source now chooses the
still-bounded **2,048 MiB** virtual-address cap solely for an
explicit validated `sops://` remote-secret worker; the
`systemd-credential://` worker remains at **512 MiB**, and CPU
10 seconds, deadline 15 seconds, retained-output bounds,
provider/profile authority and process-group cleanup are unchanged.
This increases the worst-case SOPS worker memory/address-space
allowance rather than removing its limit; the owning
`docs/remote-credentials.md` records this tradeoff. The old-cap
failure is measurement evidence, **not** a permanent test assertion
against future Go/SOPS versions.

`PORTS_AUDIT_PASS` was sent separately from CC parity or Android
readiness. At that handoff, `df -B1 .` reported **41,087,401,984**
bytes available; StageX requires at least **171,798,691,840**
actual free bytes. No StageX run, threshold bypass, global GC,
archive, source-built Rust qualification, or final release readiness
is claimed. The parent alone owns the eventual combined-ready
signal after independent CC final parity.
