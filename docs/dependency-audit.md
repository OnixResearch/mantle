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
| source policy | four git sources without `allow-git` | in the HEAD lock and absent from the HEAD policy; not from `casita` | open; predates this change |

The four sources are `ssh://git@github.com/OnixResearch/onix-artifact.git`, `https://seed.radicle.garden/zqhtZvsteJhxCJE96dMAZSZ9y1PX.git` (`bounded-tree`), `https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git` (`durable-file-publication`), and `https://seed.radicle.garden/z4Tky6zvC8w4Y6c4YBzNxVbq5n752.git` (`transactional-reconciliation-core`).

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

With `cargo-deny 0.20.2` and its available advisory database, the scoped
`nix shell --offline nixpkgs#cargo-deny nixpkgs#cargo --command cargo-deny
--config deny.toml check` exited 9: `advisories FAILED, bans ok, licenses ok,
sources FAILED`. The source errors are six lock packages from the four
already-unadmitted repositories below, not the Casita or Turso sources. Its
advisory output includes the Casita-reached `proc-macro-error 0.4.12`
unmaintained finding and existing vulnerable `rustls 0.23.37`; it also reports
`lru 0.16.4` (RUSTSEC-2026-0253, through Snix) and yanked `chacha20 0.10.0`
and `spin 0.10.0`. No waiver or unrelated dependency update was added.

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
