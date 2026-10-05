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
