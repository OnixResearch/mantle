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
- `turso` 0.8.0-pre.7, a `casita` dependency, from `https://github.com/cachix/turso.git` at revision `dca55133caa690f90dcdd58d3c4329fb0703659c`
- `deny.toml` admits both repositories individually; the `casitaSourceAdmitted` assertion in `flake.nix` checks the Casita revision, features, lock source, and `Apache-2.0` license, and `scripts/vendor-deps.py` checks the Casita manifest pin and the locked `casita`, `turso`, and `astral-tokio-tar` entries
- `blake3` stays at `1.8.2`; `astral-tokio-tar` moves from `0.6.3` to `0.6.4`
- `patches/casita-blake3-finalize.patch` changes one hunk in `crates/casita/src/nar.rs` for this revision only: method resolution in Mantle's graph selects `sha2::Digest::finalize` there, so the patch calls the BLAKE3 method explicitly; the Nix vendor closure and `scripts/vendor-deps.py` both apply it
- [ADR 0082](../adr/0082-select-store-backends-explicitly-and-admit-casita.md) lists the experimental Casita API that Mantle uses; a new Casita revision or feature set needs its own reviewed change that updates ADR 0082 and this record and reruns the checks below

At the pinned revision, `casita` declares `Apache-2.0` and the `turso`
packages declare `MIT`; both licenses are already admitted in `deny.toml`.
The transitive `cfg_block 0.1.1` declares a `license-file` rather than an SPDX
field; its vendored `LICENSE` contains Apache-2.0 text. The configured audit
reports this as a warning, not a passing review of all downstream legal
obligations. Retain the pin and experimental API inventory in ADR 0082; a
revision or feature bump must re-review those APIs, source identities,
licenses, and advisories and rerun the complete audit before updating this
record. Repository admission alone does not authorize a floating revision.

Commands:

```sh
nix build --no-link --print-out-paths .#checks.x86_64-linux.casita-vendor-closure
nix develop --offline --no-write-lock-file --command python3 scripts/vendor-deps.py generate
nix develop --offline --no-write-lock-file --command python3 scripts/vendor-deps.py check
nix develop --offline --no-write-lock-file --command \
  /nix/store/xqnwl8qppzhr3dq5nzw2ly6yx4lz9xyr-cargo-deny-0.19.0/bin/cargo-deny check
```

Results:

- the Nix check built `/nix/store/skq3d11jy1b67by802kq0d5mmkdhzkmi-vendor-cargo-deps` from the dirty working tree; it is not T1.2 clean-checkout proof
- `generate` wrote `vendor-deps/` without replacing existing data; `check` regenerated the closure, matched every file (53,491 entries), and resolved 766 external package names from locked offline metadata
- the patched `nar.rs` bytes are identical in the Nix closure and in `vendor-deps/`
- `cargo-deny` exited 9: advisories and sources failed; bans and licenses passed; the command omitted `--config deny.toml`, and its source errors name exactly the git sources that `deny.toml` does not admit

A separate full configured run after the APK lock handoff used:

```sh
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop --offline --no-write-lock-file --command \
  /nix/store/xqnwl8qppzhr3dq5nzw2ly6yx4lz9xyr-cargo-deny-0.19.0/bin/cargo-deny \
  check --config deny.toml
```

It exited **9**: advisories and sources failed, bans and licenses passed. The
only source errors were the six locked packages from the four unrelated Git
repositories below; neither Casita nor Turso raised a source-policy error.
The advisory errors were the six IDs in the table below. An additional
`cfg_block` missing-SPDX-field license warning and registry-index lookup
warnings (yanked-crate status could not be checked) prevent a stronger
completeness claim. This is not passing audit evidence.

After a coordinated APK lock handoff, Cargo's offline registry resolver (not
the dev shell's then-stale fixed vendor index) updated `h2 0.4.13` to
`0.4.16` and `rustls 0.23.37` to `0.23.45`, with their compatible
`aws-lc-rs 1.18.1`, `aws-lc-sys 0.45.0`, and `rustls-webpki 0.103.15`
dependencies. Locked offline metadata passed against the registry index;
`blake3` remained `1.8.2`. A second full
`cargo-deny 0.19.0 check --config deny.toml --hide-inclusion-graph`, using
the original Cargo registry to resolve that lock, also exited **9**.
RUSTSEC-2026-0258 (`h2`) and RUSTSEC-2026-0285 (`rustls`) no longer appear.
The remaining four advisory errors are `RUSTSEC-2024-0370`,
`RUSTSEC-2026-0247`, `RUSTSEC-2026-0292`, and `RUSTSEC-2023-0071`.
Sources still fail on the same four unrelated repositories; bans and licenses
pass. This run queried the registry index and warned that the locked
`chacha20 0.10.0` and `spin 0.10.0` are yanked; both versions already
occurred in the pre-update HEAD lock. At the time of this run the Nix vendor
closure still held the older TLS/HTTP packages. Audit success and a coherent
vendor build cannot be inferred from the lock update or this failed audit.

After the vendor owner regenerated the same locked sources, the Nix
`casita-vendor-closure` check produced
`/nix/store/z4pv4a36rgdq95l35wxzgx3xqgr7gdw4-vendor-cargo-deps`.
The repository-owned `generate` and `check` paths reported 766 external
package names in locked offline metadata and 53,596 matching vendor entries.
In the default Nix shell, `cargo metadata --locked --offline --no-deps`
passed. A full `cargo-deny check --config deny.toml` with inclusion-graph
output hidden still exited **9** with those same four advisory
errors and six source errors; bans and licenses passed. That shell's fixed
vendor Cargo home lacks a registry index and therefore reported warnings
that yanked status could not be checked. The original-registry audit above
did query it and identified the two preexisting yanked versions. These
checks do not turn the failed dependency gate into a pass.

With that refreshed closure, the default Nix shell also completed a scoped
real compilation: `cargo check --locked --offline -p crunch-store --lib`
exited **0** after checking `h2 0.4.16`, `rustls 0.23.45`, pinned Turso and
Casita, and `crunch-store`. This validates those resolved dependencies on
that toolchain; it does not resolve the failed advisory/source gate or prove
the clean-checkout Nix build required separately by T1.2.

The checkout-local `vendor-deps` comparison now reads uncached, bounded 64-KiB
byte chunks: its isolated Python regression passed (1 passed, 0 failed) after
rejecting a same-size, preserved-mtime `nar.rs` edit. The checked source
SHA-256s are `scripts/vendor-deps.py`
`7f9945d6e13869d12286549238b6deb786be9156365388f129543c940612793b`
and `tests/test_vendor_deps.py`
`ea6b45c0f6ee5e1cfd61284b406a51153dc5d0357616cac1405aea14231f1e8f`.
In disposable scratch, the tracked patch applied to the exact upstream hunk
(exit 0) and rejected changed context (exit 1); the dev-shell source-map
checker admitted patched `nar.rs` and rejected original and edited bytes.
These local checks are not clean-checkout Nix/Cargo or configured `cargo deny`
proof: T1.2, T4.6, and T4.7 remain unchecked.

| Finding | Crate | Relation to Casita | Current action |
|---|---|---|---|
| RUSTSEC-2024-0370 (unmaintained) | `proc-macro-error 0.4.12` | introduced by the Casita graph through `genawaiter 0.99.1` (both `bao-tree` and the `turso` sync crates); now present in the HEAD lock | open; no safe package upgrade or waiver |
| RUSTSEC-2026-0258 (vulnerability) | `h2 0.4.13` in HEAD | also reached from `casita` through `object_store 0.14.0` | lock updated to patched `0.4.16`; absent from configured audits of new lock; refreshed-vendor scoped store compile passed |
| RUSTSEC-2026-0285 (vulnerability) | `rustls 0.23.37` in HEAD | also reached from `casita` through `object_store 0.14.0` | lock updated to patched `0.23.45`; absent from configured audits of new lock; refreshed-vendor scoped store compile passed |
| RUSTSEC-2026-0247 (unmaintained) | `bitmaps 3.2.1` | in the HEAD lock; not reached from `casita` | open; no waiver |
| RUSTSEC-2026-0292 (vulnerability) | `imbl-sized-chunks 0.1.3` | in the HEAD lock; not reached from `casita` | open; no waiver |
| RUSTSEC-2023-0071 (vulnerability) | `rsa 0.9.10` | in the HEAD lock; not reached from `casita` | open; no waiver |
| source policy | four git sources without `allow-git` | in the HEAD lock and absent from the HEAD policy; not from `casita` | open; predates this change |

At this immutable Casita revision, `native` enables `turso` and `bao-tree`.
Pinned `turso` depends unconditionally on `turso_sync_sdk_kit`, which depends
on `genawaiter 0.99.1`; `bao-tree`'s default `validate` feature also enables
`genawaiter`. The advisory marks `proc-macro-error` unmaintained with no safe
upgrade. A lockfile-only bump cannot remove either upstream path; addressing it
requires a separate reviewed Casita/Turso/bao-tree source change, including
the experimental-API and vendor-closure review in ADR 0082. This is not
grounds for an advisory waiver.

The other unmaintained `bitmaps` and vulnerable `imbl-sized-chunks 0.1.3`
come through `nickel-lang-vector 0.2.0`; the latter is fixed in
`imbl-sized-chunks 0.2.0`, but Nickel's older dependency still resolves 0.1.3.
The `rsa 0.9.10` path comes through `secretspec 0.17.0`; its RustSec
advisory lists no patched release. These are existing, non-Casita findings,
not reasons to bypass the complete gate or call T4.7 resolved.

The four preexisting unrelated sources are:

| Repository (exact Cargo source) | Locked revision | Independent source check |
|---|---|---|
| `ssh://git@github.com/OnixResearch/onix-artifact.git` | `c932138d880ddf4c2967f4c024b489b5c0022bf1` | `artifactAuthSourceAdmitted` checks manifests, Nix source revision, lock source, package set, and `MIT OR Apache-2.0` license |
| `https://seed.radicle.garden/zqhtZvsteJhxCJE96dMAZSZ9y1PX.git` (`bounded-tree`) | `b0fd0103bc9eed2c1b6d852045959462d105d8f1` | `boundedTreeSourceAdmitted` checks manifests, Nix source revision, lock source, package set, Radicle identity, and `AGPL-3.0-or-later` license |
| `https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git` (`durable-file-publication`) | `951c27f59003cea9bfdb40ed4d89653d50fada1f` | `durablePublicationSourceAdmitted` checks manifest, Nix source revision, lock source, package identity, Radicle identity, and `MIT OR Apache-2.0` license |
| `https://seed.radicle.garden/z4Tky6zvC8w4Y6c4YBzNxVbq5n752.git` (`transactional-reconciliation-core`, dev-only) | `606489b5f40298181214bb76bc3457b607f225d9` | `transactionalReconciliationSourceAdmitted` checks manifest, Nix source revision, lock source, package identity, Radicle identity, and `MIT` license |

Each is a candidate for an individually reviewed `allow-git` entry, with the
existing independent immutable-revision checks preserved; none is a Casita or
Turso source. This change does not admit them or weaken `unknown-git = "deny"`.

The dependency gate stays open. This change adds no waiver. This record does not claim a clean `cargo-deny` result, Casita correctness, experimental API stability, or release eligibility.

## Validation dependency catalog

| Dependency | Pinned revision | Transport | Plane | License choice | Scope |
|---|---|---|---|---|---|
| [`fault-injection`](https://github.com/komora-io/fault-injection) | crates.io `=1.0.10` | `crates.io` | `validation` | Apache-2.0 (upstream also offers MIT) | `crunch-store` dev-dependency only; test-build store-shell I/O fixtures |

The process-global counter starts at `u64::MAX`; each serial fault fixture sets
an explicit counter and restores that default, with `SLEEPINESS=0`. Run the
injection fixtures with `cargo test -p crunch-store --lib io_fault_ -- --test-threads=1`.
Injection covers only the named I/O paths. It does not
establish sandbox hermeticity, store correctness, or release eligibility.

Store blob-read faults retain `Error::Store` classification; NAR stream faults
retain `Error::Export`. Post-build action-result cache-record writes and
fsync faults retain the existing `action-result-publication-write-temp` and
`action-result-publication-sync-temp` diagnostics. Publication failure remains
diagnostic for ordinary completed builds. Floating CA and CA-resolved IA
intermediates instead require signed action-result publication before dependent
consumption; failure is `ca-realisation-untrusted`. Injection changes neither rule.

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
