# Dependency audit policy

## Entry point

Run dependency audit from repo root with the checked-in policy:

```sh
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo-deny check --config deny.toml
```

Root policy lives in `deny.toml`. Unconfigured cargo-deny output is not Mantle audit evidence.
Validate audit transcripts before citing them:

```sh
cargo -Zscript scripts/check-dependency-audit-evidence.rs <evidence.md>
```

The policy governs four audit classes:

- vulnerabilities
- unsoundness
- unmaintained crates
- yanked crates

License and source checks also run through the same entry point.

## Classification buckets

Current findings are classified into these buckets:

- **first-party actionable** — this repo controls the manifest, lockfile, or vendored workspace crate and should fix the finding before adding a waiver
- **transitive but tractable** — this repo does not own the direct manifest edge, but a lockfile update or compatible dependency bump from this repo can remove the finding
- **vendored or upstream-blocked** — the finding sits behind vendored code or an upstream crate choice that is not realistically fixable here without carrying a larger fork
- **dev-only or non-runtime** — the finding appears only in dev-dependencies, optional features not enabled in the shipped path, or other non-runtime exposure

## Triage snapshot (2026-07-03)

Historical transcript: `openspec/changes/classify-dependency-audit-findings/evidence/cargo-deny-final.txt`

Transcript: Cairn evidence for `resolve-dependency-audit-waivers` on 2026-07-03.
The newer [Casita admission run](#casita-admission-run-2026-09-30) reports open findings that this table does not list.

| Finding | Crate(s) | Current classification | Current action |
|---|---|---|---|
| RUSTSEC-2026-0112 / RUSTSEC-2026-0113 / RUSTSEC-2026-0145 | `astral-tokio-tar` | fixed | bumped `vendor/snix-castore` to `astral-tokio-tar 0.6.3` |
| RUSTSEC-2026-0104 | `rustls-webpki` | fixed | bumped lockfile to `rustls-webpki 0.103.13` |
| RUSTSEC-2026-0194 / RUSTSEC-2026-0195 | `quick-xml` | upstream-blocked accepted waiver | latest `object_store 0.14.0` still caps `quick-xml` below the fixed 0.41 line |
| RUSTSEC-2026-0173 | `proc-macro-error2` | upstream-blocked accepted waiver | path is `oci-spec` -> `getset`; latest `oci-spec 0.10.0` still depends on `getset` |
| RUSTSEC-2023-0089 | `atomic-polyfill` | fixed | disabled default `postcard` features in vendored snix crates so `heapless 0.7` / `atomic-polyfill` are no longer in `Cargo.lock` |
| RUSTSEC-2024-0436 | `paste` | upstream-blocked accepted waiver | current path runs through `nickel-lang-core` |
| RUSTSEC-2023-0056 | `vm-memory` | resolved/no longer encountered | stale waiver removed after current audit reported no matching advisory criteria |
| RUSTSEC-2024-0002 | `vmm-sys-util` | resolved/no longer encountered | stale waiver removed after current audit reported no matching advisory criteria |
| RUSTSEC-2026-0204 | `crossbeam-epoch` | fixed | generated a targeted lock update from `0.9.18` to the compatible fixed `0.9.20`; no waiver added |
| license policy | `winx 0.36.4` | fixed policy gap | admitted only its declared `Apache-2.0 WITH LLVM-exception` SPDX expression; crate license checks and confidence remain enabled |
| source policy | `nickel-export-core` | fixed policy gap | admitted only `https://github.com/OnixResearch/nickel-export`; Cargo/Nix/spec checks still enforce revision `257fafc1c746f1faf156207043a4c826bfb16d49` |

## Casita admission run (2026-09-30)

This run belongs to the Cairn change `adopt-casita-store-backend`. Its results come from the vendor rail report; no transcript file is checked in. The run failed, so it is not passing audit evidence.

Admitted sources and lock updates:

- `casita` 0.1.0 from `https://github.com/cachix/casita` at revision `90404fcb1cfb3d83f2233715448dfefe913f5fd1`, with default features disabled and exactly `native` and `experimental`
- `turso`, `turso_core`, `turso_ext`, `turso_macros`, `turso_parser`, `turso_sdk_kit`, `turso_sdk_kit_macros`, `turso_sync_engine`, and `turso_sync_sdk_kit`, each locked at 0.8.0-pre.7 from `https://github.com/cachix/turso.git` revision `dca55133caa690f90dcdd58d3c4329fb0703659c` via `casita`
- `deny.toml` admits both repositories individually; the `casitaSourceAdmitted` assertion in `flake.nix` checks the Casita revision, features, lock source, and `Apache-2.0` license, and `scripts/vendor-deps.py` checks the Casita manifest pin and all nine locked Turso identities plus the `casita`, `astral-tokio-tar`, and `blake3` entries
- `blake3` stays at `1.8.2`; `astral-tokio-tar` moves from `0.6.3` to `0.6.4`
- `patches/casita-blake3-finalize.patch` changes one hunk in `crates/casita/src/nar.rs` for this revision only: method resolution in Mantle's graph selects `sha2::Digest::finalize` there, so the patch calls the BLAKE3 method explicitly; the Nix vendor closure and `scripts/vendor-deps.py` both apply it
- [ADR 0082](../adr/0082-select-store-backends-explicitly-and-admit-casita.md) lists the experimental Casita API that Mantle uses; a new Casita revision or feature set needs its own reviewed change that updates ADR 0082 and this record and reruns the checks below

The [complete admission lock inventory](../.cairn/changes/adopt-casita-store-backend/evidence/dependency-admission-2026-10-04.md#complete-original-casita-lock-admission-inventory) compares the pre-admission lock to the unchanged current lock: ten git and 89 registry identities added (including `astral-tokio-tar 0.6.4`), `astral-tokio-tar 0.6.3` removed, and dependency lists modified on 50 previously present identities. It records every new locked git identity and every changed lock-record identity; none is a passing clean-source closure or audit claim.

Commands:

```sh
nix build --no-link --print-out-paths .#checks.x86_64-linux.casita-vendor-closure
nix develop --offline --no-write-lock-file --command python3 scripts/vendor-deps.py generate
nix develop --offline --no-write-lock-file --command python3 scripts/vendor-deps.py check
nix develop --offline --no-write-lock-file --command \
  /nix/store/xqnwl8qppzhr3dq5nzw2ly6yx4lz9xyr-cargo-deny-0.19.0/bin/cargo-deny check
```

Results:

- the Nix check built `/nix/store/skq3d11jy1b67by802kq0d5mmkdhzkmi-vendor-cargo-deps` from clean source
- `generate` wrote `vendor-deps/` without replacing existing data; `check` regenerated the closure, matched every file (53,491 entries), and resolved 766 external package names from locked offline metadata
- the patched `nar.rs` bytes are identical in the Nix closure and in `vendor-deps/`
- `cargo-deny` exited 9: advisories and sources failed; bans and licenses passed; the command omitted `--config deny.toml`, and its source errors name exactly the git sources that `deny.toml` does not admit

| Finding | Crate | Relation to Casita | Current action |
|---|---|---|---|
| RUSTSEC-2024-0370 (unmaintained) | `proc-macro-error 0.4.12` | new in the lock; reached only from `casita` through `genawaiter 0.99.1` (via `bao-tree` and the `turso` sync crates) | open; no waiver |
| RUSTSEC-2026-0258 (vulnerability) | `h2 0.4.13` | in the HEAD lock; also reached from `casita` through `object_store 0.14.0` | open; no waiver |
| RUSTSEC-2026-0285 (vulnerability) | `rustls 0.23.37` | in the HEAD lock; also reached from `casita` through `object_store 0.14.0` | open; no waiver |
| RUSTSEC-2026-0247 (unmaintained) | `bitmaps 3.2.1` | in the HEAD lock; not reached from `casita` | open; no waiver |
| RUSTSEC-2026-0292 (vulnerability) | `imbl-sized-chunks 0.1.3` | in the HEAD lock; not reached from `casita` | open; no waiver |
| RUSTSEC-2023-0071 (vulnerability) | `rsa 0.9.10` | in the HEAD lock; not reached from `casita` | open; no waiver |
| source policy | four git sources without `allow-git` at the 2026-09-30 run | not from `casita`; independently pinned in manifests, lock, and Nix assertions | exact URLs added to `allow-git` on 2026-10-04; post-change `cargo-deny` execution still unavailable |

The four now allowlisted sources are:

- `ssh://git@github.com/OnixResearch/onix-artifact.git`
  (`artifact-auth-core` and `artifact-auth-ed25519`, revision
  `c932138d880ddf4c2967f4c024b489b5c0022bf1`)
- `https://seed.radicle.garden/zqhtZvsteJhxCJE96dMAZSZ9y1PX.git`
  (`bounded-tree-cap` and `bounded-tree-core`,
  `b0fd0103bc9eed2c1b6d852045959462d105d8f1`)
- `https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git`
  (`durable-file-publication`, `951c27f59003cea9bfdb40ed4d89653d50fada1f`)
- `https://seed.radicle.garden/z4Tky6zvC8w4Y6c4YBzNxVbq5n752.git`
  (`transactional-reconciliation-core`,
  `606489b5f40298181214bb76bc3457b607f225d9`)

Each origin has an independent exact-revision manifest/lock/input assertion in
`flake.nix`; `deny.toml` lists each URL individually with
`unknown-git = "deny"` and no wildcard.

The dependency gate stays open. This change adds no waiver. This record does not claim a clean `cargo-deny` result, Casita correctness, experimental API stability, or release eligibility.

2026-10-04 bounded admission follow-up (not a passing dependency gate):
`scripts/vendor-deps.py` now rejects lock-source or version drift for all nine
locked `turso` workspace packages, not just `turso`, and rejects drift in the
sole `casita`, `astral-tokio-tar`, and `blake3` entries. The Nix admission
assertion checks the same locked package family before selecting the patched
Casita source. The local pin validator accepted the current 987 lock
identities and rejected a changed `turso_core` source in a scratch lock; Nix
evaluated the vendor derivation but did **not** build a clean-source closure.
At the first bounded read, both Rust-from-source self-build plans selected
final compiler 1.94.0, below Casita's declared `rust-version` 1.94.1. They
now declare 1.94.1 with the official Rust distribution source SHA-256 and
matching final-stage/source identities; both typed Nickel plans evaluated
and passed the bounded identity check recorded in the active change's
`evidence/dependency-admission-2026-10-04.md`. This does not qualify a
source-built compiler: its build/requalification and the clean-source vendor
proof remain unrun. T1.2 and the dependency audit gate remain open.

2026-10-05 clean-source follow-up (still not an audit pass): the Nix
`casita-vendor-closure` derivation built, and the separate Nix
`casita-crunch-store-check` built and ran `cargo check`. The default dev
shell compiled patched pinned Casita without an explicit Cargo config;
the generated ignored `vendor-deps/` closure compiled in an isolated Cargo
home, matched a fresh generation (53,491 entries), and passed unpatched,
hand-edited, mutable-checkout and upstream/patch-drift negative controls.
An archive of exact source commit
`2ab0f6dc58eee9978c84667fcb7da490ea7ad702` plus that closure
resolved 850 external packages offline under its extracted `vendor-deps/`;
its recursive unpacked source BLAKE3 SRI is
`blake3-6nOZ9WOsN6DrvnTfKR5/C2DojRZH6cICZlxvNWf8TA8=`.
The initial archive script exited 127 when ambient Cargo was absent; a
no-clobber Nix dev-shell metadata proof passed against the unchanged archive.
Runs 63 and 64 in the active change evidence retain the original failed
log, passing corrective receipt, exact archive hash and patch digest. The
source-built Rust 1.90 sidecar in Run 62 does not qualify the final 1.94.1
source profile, and none of these closures clears the separate unwaived
`cargo-deny` advisory gate or marks T1.2 complete.

Before those four exact-URL admissions, with `cargo-deny 0.20.2` and its available advisory database, the scoped
`nix shell --offline nixpkgs#cargo-deny nixpkgs#cargo --command cargo-deny
--config deny.toml check` exited 9: `advisories FAILED, bans ok, licenses ok,
sources FAILED`. The six source errors were lock packages from the four
then-unadmitted repositories above, not the Casita or Turso sources. Its
advisory output includes the Casita-reached `proc-macro-error 0.4.12`
unmaintained finding and existing vulnerable `rustls 0.23.37`; it also reports
`lru 0.16.4` (RUSTSEC-2026-0253, through Snix) and yanked `chacha20 0.10.0`
and `spin 0.10.0`. No waiver or unrelated dependency update was added.

2026-10-04 no-waiver policy follow-up: a bounded positive/negative fixture
parsed the four manifest pin sets, all six corresponding lock package
identities and their full `?rev=...#...` sources, the separate `flake.nix`
revision/source assertions, and `deny.toml`. All four pinned sets matched;
an alien repository URL was outside the policy, and a scratch changed
revision did not match any expected lock source. This is a static guard
check, **not** a `cargo-deny` pass or proof of Nix build evaluation.
A post-change attempt to run
`nix shell --offline nixpkgs#cargo-deny nixpkgs#cargo --command cargo-deny --config deny.toml check`
scheduled four uncached `cargo-deny 0.20.2` source/vendor/binary derivations
and reported `failed to start SSH connection to 'aspen1.local'`; it was
canceled immediately under the no-long-build instruction. At that 2026-10-04
point there was no post-admission `cargo-deny` exit code; the next configured
run is recorded below.

### Mantle-owned Bao source candidate (2026-10-05)

Before the in-repo Bao cutover, the exact Casita revision reached:

```text
casita -> registry bao-tree 0.16.1 -> genawaiter 0.99.1
  -> genawaiter-proc-macro 0.99.1 -> proc-macro-error 0.4.12
```

The registry Bao crate's `validate` feature used the unmaintained proc macro.
Mantle now patches `bao-tree` 0.16.1 to the complete local candidate under
`third_party/bao-tree/`, keeping Casita at
`90404fcb1cfb3d83f2233715448dfefe913f5fd1` and its `native` and
`experimental` features. The original Bao upstream base is
`2be9abd144783455606424424c29bd3a57f926f8`; the locally validated,
validation-preserving candidate is
`eecfbbb458cc684fd85e056881580d307a1d1868`. The 35-file tracked
`third_party/bao-tree-source.json` records each file's BLAKE3 identity:
`Cargo.toml` is
`d3367e0ef37daeda08d361c2399e9cf4938289e5bb076e4622f1a05412b4eb62`,
and the replacement `src/io/validate.rs` is
`857b4feffd0feb4d3933f867bf335c6186c3ec34b992424cdfba52a1e24bdd8c`.
The importer's offline `check` and optional exact-candidate comparison verify
local file identity, not GitHub authorship, upstream review, or release.

`Cargo.lock` now contains one path-source Bao 0.16.1 and no
`genawaiter-proc-macro`, `proc-macro-error`, `proc-macro-error-attr`,
`proc-macro-hack`, or `syn-mid`. The Bao `validate` feature remains active and
depends on `futures-lite`. `genawaiter` and `genawaiter-macro` **still remain**
through Casita's pinned Turso sync graph; the graph change is the removal of
the unmaintained *Bao-reachable* proc-macro path, not a global no-generator
claim. Host `cargo check --locked --offline -p crunch-store --config
.cargo/vendor-config.toml` compiled the patched Bao, pinned Casita, and
Mantle adapter. Seven individually selected Bao validator tests passed with
explicit `tokio_fsm,validate`, covering ordered corruption/short reads,
right-outboard proof preservation, asynchronous on-demand/error ordering,
and keyed outboard positive, negative, and wrong-key cases. The
repository-owned generator regenerated the checkout-local vendor closure
and compared 53,335 entries; the prior ignored closure was preserved in a
named sibling backup. The clean-source Nix Bao integrity and Casita vendor
closure checks passed. The Nix `casita-crunch-store-check` passed as
`/nix/store/cp7jmx7ajlklg1mnx0lm2x5zz1fjp7lq-mantle-casita-crunch-store-check-0.1.0`;
its log records a real release Cargo check of Bao, pinned Casita, and
`crunch-store`. A Casita trust-policy store archive round trip passed as
an independent consumer smoke, not proof of Bao multi-block validation
semantics.

The configured `cargo-deny 0.19.0` run after the Bao cutover reported
`advisories FAILED, bans ok, licenses ok, sources ok`: Bao-reachable
RUSTSEC-2024-0370 was absent, but five previously locked, unwaived findings
remained. A subsequent offline, targeted lock update moves `h2 0.4.13` to
`0.4.16` (the RUSTSEC-2026-0258 fixed threshold) and `rustls 0.23.37` to
`0.23.45` (the RUSTSEC-2026-0285 fixed threshold). Rustls also requires
`aws-lc-rs 1.18.0`, `aws-lc-sys 0.44.0`, and `rustls-webpki 0.103.14`.
The pinned Casita and Turso sources, local Bao import, and `blake3 1.8.2`
remain unchanged; no deny waiver was added.

On the updated lock, `cargo-deny 0.19.0 check --config deny.toml` exits 1:
`advisories FAILED, bans ok, licenses ok, sources ok`. Neither h2 nor rustls
is reported. Three unwaived findings remain: RUSTSEC-2026-0247
(`bitmaps 3.2.1`, unmaintained, no fixed release) and RUSTSEC-2026-0292
(`imbl-sized-chunks 0.1.3`, fixed only at >=0.2.0) both pass through
`nickel-lang-vector 0.2.0`, whose `^0.1` constraint has no compatible fixed
release; RUSTSEC-2023-0071 (`rsa 0.9.10`, no patched release) comes from
`secretspec 0.17.0`, which requires RSA unconditionally, as does its latest
surveyed release 0.21.1. The audit also warns about yanked
`chacha20 0.10.0` and `spin 0.10.0`. There is no published no-waiver lock
upgrade for these three findings: review future Nickel, SecretSpec, and RSA
upstream releases. T1.3 and T4.7 remain unchecked. This is not a passing
dependency gate, full CI claim, StageX proof, upstream-reviewed Bao release,
Casita-maintainer approval, or release claim.

The selected offline CLI cache-restoration test initially failed while compiling
both shared-root binaries with host nightly Rust 1.99: layout of
`attest_cmd::cmd_attest_async()` exceeded the default compiler query depth.
`src/main.rs` alone now sets `#![recursion_limit = "256"]`; the library crate
did not need that bound. Rerunning the exact test passed (one test) and exercised
Casita and Snix separately: each built a signed fixed output, removed its
physical export, then restored identical BLAKE3 bytes from cache in a fresh
process. That test used a local `file://` input. Separately, an ephemeral
localhost HTTPS cache server with a temporary CA exercised Mantle's
`--store-backend casita store pull` reqwest/rustls client: with `SSL_CERT_FILE`
pointing at the trusted CA, the client fetched `/nix-cache-info` and a
`.narinfo` path, then exited 0 reporting one intentionally missing narinfo
and zero imports. Switching only `SSL_CERT_FILE` to an unrelated CA exited 3
with a generic request transport error and sent no HTTP GET. Both cases used
loopback, and all temporary TLS files were removed. This proves a trusted
HTTPS connection and bounds the untrusted-certificate case; it does not
demonstrate a successful store import or HTTP/2 protocol behavior.

## Remaining waiver inventory

| Finding | Affected crate | Scope | Rationale | Review trigger |
|---|---|---|---|---|
| RUSTSEC-2024-0436 | `paste` | upstream-blocked transitive | path is `crunch-eval` -> `nickel-lang-core 0.18.0` -> `paste`; removing it requires a Nickel upstream release or a carried Nickel core migration | revisit when Nickel publishes a paste-free core release or Mantle intentionally carries that migration |
| RUSTSEC-2026-0173 | `proc-macro-error2` | upstream-blocked transitive build-time proc macro | path is `vendor/snix-build` -> `oci-spec 0.7.1` -> `getset 0.1.6` -> `proc-macro-error2`; latest checked `oci-spec 0.10.0` still uses `getset` | revisit when `oci-spec` removes `getset` or `getset` migrates away from `proc-macro-error2` |
| RUSTSEC-2026-0194 | `quick-xml` | upstream-blocked transitive | path is `vendor/snix-castore` -> `object_store 0.14.0` -> `quick-xml 0.40.1`; `object_store 0.14.0` pins below the fixed 0.41 line | revisit when `object_store` allows `quick-xml >= 0.41` |
| RUSTSEC-2026-0195 | `quick-xml` | upstream-blocked transitive | same `vendor/snix-castore` -> `object_store 0.14.0` -> `quick-xml 0.40.1` cap as RUSTSEC-2026-0194 | revisit when `object_store` allows `quick-xml >= 0.41` |

## Policy rules

- prefer real fixes over waivers when this repo can update the dependency safely
- keep waivers per finding and per crate, never as blanket class suppressions
- every remaining waiver must record scope, rationale, and a review trigger
- treat optional-feature or dev-only exposure differently from shipped runtime exposure, but document that distinction explicitly
- admit Git repositories individually and retain independent immutable-revision checks; repository admission does not authorize floating refs or arbitrary commits
- admit compound SPDX expressions exactly as declared and reviewed; do not replace license checks with crate-wide bypasses or lower confidence

## Claim boundary

A passing audit proves only that the resolved dependency graph satisfies the checked-in advisory, ban, license, and source policy against the advisory database available to that run. It does not prove dependency correctness, permanent absence of future advisories, legal suitability for every downstream distribution, trust in arbitrary commits from an admitted repository, or release eligibility.
