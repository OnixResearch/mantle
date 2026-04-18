# Dependency audit policy

## Entry point

Run dependency audit from repo root:

```sh
cargo deny check
```

Root policy lives in `deny.toml`.

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

Initial transcript: `openspec/changes/classify-dependency-audit-findings/evidence/cargo-deny-initial.txt`

Current exit-zero transcript after direct fixes and narrow waivers:
`openspec/changes/classify-dependency-audit-findings/evidence/cargo-deny-final.txt`

| Finding | Crate(s) | Current classification | Current action |
|---|---|---|---|
| RUSTSEC-2026-0066 | `astral-tokio-tar` | first-party actionable (`vendor/snix-castore`) | fixed locally by bumping to `0.6.0`; targeted `cargo check -p snix-castore -p snix-store -p crunch-build` passes |
| RUSTSEC-2026-0002 | `lru` | first-party actionable (`vendor/snix-store`) | fixed locally by bumping to `0.16.4`; targeted `cargo check -p snix-castore -p snix-store -p crunch-build` passes |
| RUSTSEC-2026-0097 | `rand` 0.8/0.9/0.10 lines | transitive but tractable | fixed locally by patching lockfile to `0.8.6`, `0.9.3`, and `0.10.1` |
| RUSTSEC-2026-0098 | `rustls-webpki` | transitive but tractable | fixed locally by patching lockfile to `0.103.12` |
| RUSTSEC-2026-0099 | `rustls-webpki` | transitive but tractable | fixed locally by patching lockfile to `0.103.12` |
| yanked | `fastrand` | transitive but tractable | fixed locally by patching lockfile to `2.4.1` |
| RUSTSEC-2023-0089 | `atomic-polyfill` | vendored or upstream-blocked | waiver candidate; current path runs through `heapless` -> `postcard` |
| RUSTSEC-2024-0436 | `paste` | vendored or upstream-blocked | waiver candidate; current path runs through `nickel-lang-core` |
| RUSTSEC-2023-0056 | `vm-memory` | vendored or upstream-blocked | waiver candidate; attempted direct bump is blocked by `fuse-backend-rs` and the `vhost`/`virtiofs` version stack |
| RUSTSEC-2024-0002 | `vmm-sys-util` | vendored or upstream-blocked | waiver candidate; attempted direct bump is blocked by `fuse-backend-rs` and the `vhost`/`virtiofs` version stack |

## Remaining waiver inventory

| Finding | Affected crate | Scope | Rationale | Review trigger |
|---|---|---|---|---|
| RUSTSEC-2023-0089 | `atomic-polyfill` | upstream-blocked transitive | enters through `heapless`/`postcard`; no repo-local patch-level upgrade removes it cleanly today | revisit when `heapless` or `postcard` moves to `portable-atomic` |
| RUSTSEC-2024-0436 | `paste` | upstream-blocked transitive | enters through `nickel-lang-core`; removing it requires Nickel upstream or a larger carried fork | revisit when Nickel publishes a release that removes `paste` |
| RUSTSEC-2023-0056 | `vm-memory` | vendored/upstream-blocked | current `fuse-backend-rs` line plus `vhost`/`virtiofs` pins block a clean repo-local bump to `0.12.2` | revisit when vendored `fuse-backend-rs` or the virtio stack can move together |
| RUSTSEC-2024-0002 | `vmm-sys-util` | vendored/upstream-blocked | current `fuse-backend-rs` line plus `vhost`/`virtiofs` pins block a clean repo-local bump to `0.12.x` | revisit when vendored `fuse-backend-rs` or the virtio stack can move together |

## Policy rules

- prefer real fixes over waivers when this repo can update the dependency safely
- keep waivers per finding and per crate, never as blanket class suppressions
- every remaining waiver must record scope, rationale, and a review trigger
- treat optional-feature or dev-only exposure differently from shipped runtime exposure, but document that distinction explicitly
