# Integration run, 2026-11-18

Branch `cairn/wrpc-producer-publication-20261118` on `origin/main`
`cecd04e5e`. Nix built the bundles locally with `builders =`, `max-jobs = 8`,
`cores = 16`.

## Producer bundle

```sh
nix build --no-link --print-out-paths --builders '' .#wrpc-reference-bundle
```

Exit 0:
`/nix/store/p5046s4gilgxgacw8ccaqsdhn4yl5clh-mantle-wrpc-reference-bundle-aa01409ed38bf7a978570ff567c4475376925df6`.
Its `manifest.json` has schema `mantle-wrpc-reference-bundle-v1`,
`manifest_identity_blake3`
`4ea1e0629d793fb8cc683f8e030296dab3a3018630b67047e6bd07ee9ddb84cc` and 262
members. The `octet-report-not-admitted` blocker from 2026-10-30 no longer
occurs: the toolchain now selects Octet's `wrpc-reference` artifact profile.

```sh
MANTLE_WRPC_REFERENCE_BUNDLE=/nix/store/p5046s4gilgxgacw8ccaqsdhn4yl5clh-mantle-wrpc-reference-bundle-aa01409ed38bf7a978570ff567c4475376925df6 \
  cargo test -p mantle-wasm-consumer-verifier --features std \
  --test wrpc-producer -- --ignored --test-threads 1
```

`test result: ok. 5 passed; 0 failed`: actual dual-bundle acceptance, missing
component without a local guest rebuild, same-length drift, substitution of
the other real component, and a local-build symlink replacement.

Provenance caveat: the toolchain pins Octet `f081ed9682d478e585f4cede89bb89b542755ab9`
("wasm-artifact: admit the exact wrpc reference component profile",
2026-11-03). After Octet's 2026-11-12 public-history rewrite, that commit is not
on Octet `main`: GitHub's compare reports `diverged`, 690 ahead and 1,290
behind. Octet `main` has no `wrpc-reference` profile. The admission above is
bound to the pinned revision through the flake lock; it is not admission under
Octet's current mainline.

## Telemetry bundle

`wasm-tools component wit` on the compiled emitter reports these imports:

```text
import wasi:otel/types@0.2.0-rc.2;
import wasi:clocks/wall-clock@0.2.3;
import wasi:otel/tracing@0.2.0-rc.2;
```

The printed `wasi:clocks@0.2.3` package declares only
`record datetime { seconds: u64, nanoseconds: u32 }`; the vendored upstream
dependency is `wasi:clocks@0.2.0`. The 2026-10-30 record's "no WASI imports"
diagnosis was incomplete: the clock import is a type-only dependency.

The Nix bundle run on the emitter alone stopped at `wasi-virt-failed` with
`no dependencies of component ... were found`. Outside Nix, with the pinned
toolchain, the emitter composed with the comparison sink (remaining imports
`wasi:otel/types` and the clock) failed the same way under `wasi-virt
--wasi-version 0.2.3`, although the deny-all adapter exports
`wasi:clocks/wall-clock@0.2.3`; `wac plug` of that adapter into the same
component succeeded. ADR 0160 records the decision: type-only WASI imports do
not require WASI-Virt and must match the expected remaining imports exactly.
`mantle-wrpc-author telemetry` now expects all three actual imports.

```sh
nix build --no-link --keep-going --builders '' .#wrpc-telemetry-reference-bundle
```

Exit 1. The pipeline now passes virtualization and stops at
`{"code":"octet-report-not-admitted","stage_key":"octet-validation"}`.
Running the toolchain's `cargo-octet evidence collect --rail wasm-artifact
--profile wrpc-reference` on the compiled emitter reports the same class of
denial: `disallowed-import` for
`component::wasi:clocks/wall-clock@0.2.3::Instance(1)`,
`component::wasi:otel/tracing@0.2.0-rc.2::Instance(5)` and
`component::wasi:otel/types@0.2.0-rc.2::Instance(0)`, plus `resource-limit`:
`imports count 3 exceeds profile limit 2`. The `wrpc-reference` profile admits
only the hello pair, so this denial is correct under that profile.

## Checks on the final source

| Check | Result |
| --- | --- |
| `cargo test -p crunch-wasm-component-core` | `test result: ok. 40 passed; 0 failed` |
| `cargo test -p crunch-wasm-component --lib` | `test result: ok. 21 passed; 0 failed` |
| `cargo clippy -p crunch-wasm-component-core -p crunch-wasm-component --all-targets --no-deps -- -D warnings` | passed |
| `./scripts/check-first-party-tigerstyle.sh -p crunch-wasm-component -p crunch-wasm-component-core` | exit 0 |

## Remaining decisions

- Telemetry admission needs an Octet artifact profile for the emitter's exact
  imports. The profile's import facts must come from the pipeline's final
  normalized component, because they carry instance indices. Octet owns that
  profile and its publication.
- Both wRPC profiles must reach Octet's current `main` before Mantle can move
  its Octet pin off the rewritten-history commit.
- Aspen's telemetry consumer currently admits `wasi:clocks/wall-clock@0.2.0`
  as a type-only dependency; the actual emitter imports `0.2.3`.

## Non-claims

These runs do not establish remote RPC execution, telemetry export, consumer
host behavior, sandbox effectiveness, Octet admission of the emitter, or
publication of an immutable package address.
