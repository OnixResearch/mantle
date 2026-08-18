# Evidence: refresh-dependency-security-audit

Date: 2026-07-03
Policy: `deny.toml`
Tool: `cargo-deny 0.19.0`

## Audit command

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo-deny --version
cargo-deny 0.19.0

$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo-deny check --config deny.toml
advisories ok, bans ok, licenses ok, sources ok
```

The successful audit uses the checked-in policy path explicitly. Default-policy
or missing-policy output is not accepted as Mantle audit evidence.

## Negative policy check

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo-deny check --config /tmp/mantle-missing-deny-policy.toml
advisories FAILED, bans ok, licenses FAILED, sources ok
```

This negative run shows why a default or missing policy result is not the repo
security-audit result. The accepted result must name `deny.toml`.

## Fixed findings

| Finding | Crate | Classification | Evidence/action |
|---|---|---|---|
| RUSTSEC-2026-0112 | `astral-tokio-tar` | fixed | bumped `vendor/snix-castore` to `astral-tokio-tar 0.6.3` |
| RUSTSEC-2026-0113 | `astral-tokio-tar` | fixed | same `astral-tokio-tar 0.6.3` bump |
| RUSTSEC-2026-0145 | `astral-tokio-tar` | fixed | same `astral-tokio-tar 0.6.3` bump |
| RUSTSEC-2026-0104 | `rustls-webpki` | fixed | lockfile bumped to `rustls-webpki 0.103.13` |
| RUSTSEC-2023-0056 | `vm-memory` | resolved/no longer encountered | stale waiver removed; current audit reported no matching advisory criteria |
| RUSTSEC-2024-0002 | `vmm-sys-util` | resolved/no longer encountered | stale waiver removed; current audit reported no matching advisory criteria |

## Accepted upstream-blocked residual findings

The audit exits zero because these residual findings are intentionally waived in
`deny.toml`; they are still visible here and in `docs/dependency-audit.md`.

| Finding | Crate | Classification | Rationale | Review trigger |
|---|---|---|---|---|
| RUSTSEC-2023-0089 | `atomic-polyfill` | upstream-blocked accepted waiver | comes through `heapless`/`postcard` | revisit when `heapless` or `postcard` moves to `portable-atomic` |
| RUSTSEC-2024-0436 | `paste` | upstream-blocked accepted waiver | comes through `nickel-lang-core` | revisit when Nickel removes or replaces `paste` |
| RUSTSEC-2026-0173 | `proc-macro-error2` | upstream-blocked accepted waiver | comes through `oci-spec` -> `getset`; latest checked `oci-spec 0.10.0` still depends on `getset` | revisit when `oci-spec` removes `getset` or `getset` migrates away from `proc-macro-error2` |
| RUSTSEC-2026-0194 | `quick-xml` | upstream-blocked accepted waiver | comes through `object_store`; latest checked `object_store 0.14.0` still depends on `quick-xml 0.40.1`, below fixed 0.41 | revisit when `object_store` allows `quick-xml >= 0.41` |
| RUSTSEC-2026-0195 | `quick-xml` | upstream-blocked accepted waiver | same `object_store` cap as RUSTSEC-2026-0194 | revisit when `object_store` allows `quick-xml >= 0.41` |

## Dependency-edge evidence

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo tree --locked -i quick-xml@0.40.1 --depth 3
quick-xml v0.40.1
└── object_store v0.14.0
    └── snix-castore v0.1.0 (/home/brittonr/git/mantle/vendor/snix-castore)
        ├── crunch-build v0.1.0 (/home/brittonr/git/mantle/crates/crunch-build)
        ├── crunch-delta v0.1.0 (/home/brittonr/git/mantle/crates/crunch-delta)
        │   [dev-dependencies]
        ├── crunch-store v0.1.0 (/home/brittonr/git/mantle/crates/crunch-store)
        ├── mantle v0.1.0 (/home/brittonr/git/mantle)
        ├── snix-build v0.1.0 (/home/brittonr/git/mantle/vendor/snix-build)
        └── snix-store v0.1.0 (/home/brittonr/git/mantle/vendor/snix-store)

$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo info object_store@0.14.0 -v | grep quick-xml
  cloud-base       = [serde, serde_json, quick-xml, hyper, chrono/serde, base64, rand, http-body-util, form_urlencoded, serde_urlencoded, tokio]
  quick-xml        = [dep:quick-xml]
  quick-xml@0.40.1

$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo tree --locked -i proc-macro-error2@2.0.1 --depth 4
proc-macro-error2 v2.0.1
└── getset v0.1.6 (proc-macro)
    └── oci-spec v0.7.1
        └── snix-build v0.1.0 (/home/brittonr/git/mantle/vendor/snix-build)
            ├── crunch-build v0.1.0 (/home/brittonr/git/mantle/crates/crunch-build)
            ├── crunch-pipeline v0.1.0 (/home/brittonr/git/mantle/crates/crunch-pipeline)
            └── mantle v0.1.0 (/home/brittonr/git/mantle)

$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo info oci-spec@0.10.0 -v | grep getset
 +getset@0.1.3
```

## Lockfile and compile verification

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo metadata --locked --format-version 1 --no-deps >/tmp/mantle-metadata-locked-2026-07-03.json
cargo metadata --locked --format-version 1 --no-deps: ok

$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo check -p snix-castore -p snix-store -p crunch-build
    Checking object_store v0.14.0
    Checking snix-store v0.1.0 (/home/brittonr/git/mantle/vendor/snix-store)
    Checking snix-build v0.1.0 (/home/brittonr/git/mantle/vendor/snix-build)
    Checking crunch-store v0.1.0 (/home/brittonr/git/mantle/crates/crunch-store)
    Checking crunch-build v0.1.0 (/home/brittonr/git/mantle/crates/crunch-build)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 32.76s
```

## Post-archive validation

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 5,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 20,
  "valid": true
}
```
