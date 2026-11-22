# Design: realize build cores

## Decision

`realize --build-cores N` sets `NIX_BUILD_CORES` for every sandbox build the realization runs.
The flag is a caller-chosen capacity knob with no effect on derivation or action identity; it is
recorded only as a fact of the run in receipt v2.

## Compatibility rules

- `N` is an integer in `1..=64` (`FOREIGN_BUILD_CORES_MIN`/`FOREIGN_BUILD_CORES_MAX`), default
  `1`, enforced by the clap parser and re-checked by `validate_foreign_build_cores` before the
  store is opened.
- A value other than `1` is refused on every route except `cache-inputs-build-roots-v2`, because
  receipt v1 has no field to record it. The refusal precedes `preflight_store` and any write.
- `roots[].built.build_cores` is `Option<u32>` with `serde(default, skip_serializing_if)`:
  receipts written before the field keep parsing, and serializing `None` reproduces their exact
  canonical bytes, so their digests are unchanged. A run records `Some(N)` only for `N` above the
  default, so a default run's receipt is byte-identical to one written before the option existed.
- Reuse classification never reads `build_cores`; a root built with any core count is reusable.

## Threading

`BuildConfig` carries `build_cores: u32` (default `1` in every constructor). The pipeline passes
it to the `Builder` (`set_build_cores`), which hands it to `derivation_to_build_request`. The
value seeds `default_sandbox_environment` before `overlay_derivation_environment`, so a
derivation-declared `NIX_BUILD_CORES` still wins. The declared-only (Nix-protocol) profile
environment has no sandbox defaults; it receives the value only when it is above the default, so
default builds keep the variable unset exactly as before.

The realize shell reads the value from `ForeignRealizationRequest::build_cores`, which
`run_realize` fills from the flag after the route check.

## Rejected alternatives

- Deriving the value from `--jobs`: the two knobs are independent (the Octet job needs four
  build cores while the scheduler may run one job at a time).
- Mutating the derivations' environment: that changes the derivation identity and the receipt's
  recorded plan relationship.
- A generic environment-override channel: unnecessary surface for one documented variable.

## Evidence

- `tests/cache_inputs_build_route.rs`: `--build-cores 4` writes `4` into the built output and
  records `build_cores: 4`; the default run's builder sees the variable unset and the receipt has no
  `build_cores`; route v1 is refused; `0` and `65` are refused; an index receipt without
  `build_cores` still classifies as reusable.
- `crates/crunch-build` unit tests: the sandbox default is `1`, the requested count is applied to
  native and foreign profiles, and a declared value still wins.
