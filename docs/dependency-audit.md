# Dependency audit policy

## Entry point

Run dependency audit from repo root with the checked-in policy:

```sh
SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo-deny check --config deny.toml
```

Root policy lives in `deny.toml`. Default-policy or missing-policy output is not Mantle audit evidence.

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

## Current triage snapshot

Historical transcript: `openspec/changes/classify-dependency-audit-findings/evidence/cargo-deny-final.txt`

Latest transcript: Cairn evidence for `refresh-dependency-security-audit` on 2026-07-03.

| Finding | Crate(s) | Current classification | Current action |
|---|---|---|---|
| RUSTSEC-2026-0112 / RUSTSEC-2026-0113 / RUSTSEC-2026-0145 | `astral-tokio-tar` | fixed | bumped `vendor/snix-castore` to `astral-tokio-tar 0.6.3` |
| RUSTSEC-2026-0104 | `rustls-webpki` | fixed | bumped lockfile to `rustls-webpki 0.103.13` |
| RUSTSEC-2026-0194 / RUSTSEC-2026-0195 | `quick-xml` | upstream-blocked accepted waiver | latest `object_store 0.14.0` still caps `quick-xml` below the fixed 0.41 line |
| RUSTSEC-2026-0173 | `proc-macro-error2` | upstream-blocked accepted waiver | path is `oci-spec` -> `getset`; latest `oci-spec 0.10.0` still depends on `getset` |
| RUSTSEC-2023-0089 | `atomic-polyfill` | upstream-blocked accepted waiver | current path runs through `heapless` -> `postcard` |
| RUSTSEC-2024-0436 | `paste` | upstream-blocked accepted waiver | current path runs through `nickel-lang-core` |
| RUSTSEC-2023-0056 | `vm-memory` | resolved/no longer encountered | stale waiver removed after current audit reported no matching advisory criteria |
| RUSTSEC-2024-0002 | `vmm-sys-util` | resolved/no longer encountered | stale waiver removed after current audit reported no matching advisory criteria |

## Remaining waiver inventory

| Finding | Affected crate | Scope | Rationale | Review trigger |
|---|---|---|---|---|
| RUSTSEC-2023-0089 | `atomic-polyfill` | upstream-blocked transitive | enters through `heapless`/`postcard`; no repo-local patch-level upgrade removes it cleanly today | revisit when `heapless` or `postcard` moves to `portable-atomic` |
| RUSTSEC-2024-0436 | `paste` | upstream-blocked transitive | enters through `nickel-lang-core`; removing it requires Nickel upstream or a larger carried fork | revisit when Nickel publishes a release that removes `paste` |
| RUSTSEC-2026-0173 | `proc-macro-error2` | upstream-blocked transitive build-time proc macro | enters through `oci-spec` -> `getset`; latest checked `oci-spec 0.10.0` still uses `getset` | revisit when `oci-spec` removes `getset` or `getset` migrates away from `proc-macro-error2` |
| RUSTSEC-2026-0194 | `quick-xml` | upstream-blocked transitive | enters through `object_store`; latest checked `object_store 0.14.0` still selects `quick-xml 0.40.1` below the fixed 0.41 line | revisit when `object_store` allows `quick-xml >= 0.41` |
| RUSTSEC-2026-0195 | `quick-xml` | upstream-blocked transitive | same `object_store` cap as RUSTSEC-2026-0194 | revisit when `object_store` allows `quick-xml >= 0.41` |

## Policy rules

- prefer real fixes over waivers when this repo can update the dependency safely
- keep waivers per finding and per crate, never as blanket class suppressions
- every remaining waiver must record scope, rationale, and a review trigger
- treat optional-feature or dev-only exposure differently from shipped runtime exposure, but document that distinction explicitly
