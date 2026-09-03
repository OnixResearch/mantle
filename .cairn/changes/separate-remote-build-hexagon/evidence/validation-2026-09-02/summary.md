# Focused validation summary

## Baseline and compatibility

- Clean baseline: 117 `crunch-build` distributed tests and 146 Mantle `remote_build::` tests passed.
- Final `crunch-remote`: 5 application and port tests passed.
- Final `crunch-remote-core`: 12 core tests and one compile-fail doctest passed.
- Final `crunch-build distributed`: 118 tests passed.
- Final Mantle `remote_build::`: 146 tests passed.
- Final Mantle `remote_hexagon::`: 3 adapter and wire-projection tests passed.
- The normalized remote-build key and realization key match their retained legacy algorithms.
- Existing and extracted wire-frame projections match byte-for-byte for accepted hello, queued, and error fixtures.

## Architecture and quality

- Host and `wasm32-unknown-unknown` checks for `crunch-remote-core` passed.
- The architecture checker reports zero findings across the core, application ports, Snix adapter boundary, and six outer adapter modules.
- All 13 forbidden-authority negative fixtures are detected. The positive Mantle-contract fixture is accepted.
- Focused Nix checks for the architecture rail, core tests, and core `wasm32` build passed.
- Strict first-party Clippy passed with `-D warnings`.
- The exact Tiger Style Nix gate passed without an allowance, baseline, or reduced scope.
- Machine-contract generation and validation passed with 24 contracted and 57 classified surfaces.
- The durable-publication adoption gate passed after exact Cargo and flake bindings were refreshed.

## Octet diagnostic

The pinned deterministic-core profile reported no ordinary Dylint source rows. Its required function-address scan remained non-green with 128 diagnostics. A fresh official deny-all diagnostic also inventories stricter source-shape findings for this existing workspace. These Octet runs are recorded as diagnostics, not acceptance evidence, and no baseline or disabled lint was added.

## Oracle checkpoint

- **Question:** Does the extraction create an inward dependency direction while preserving accepted remote behavior?
- **Inspected evidence:** The new core is `no_std + alloc`; the application defines eight Mantle-owned ports; Snix types are confined to `distributed::snix_adapter`; focused old-path and new-path tests pass with unchanged counts; wire, identity, retry, fencing, transfer-limit, output-trust, adapter-failure, and replay cases pass.
- **Decision:** Accept the focused extraction. Keep active gateway, resource, transfer, telemetry, external-batch, nominal-type, failure-debug, and Trellis policy with their current owners.
- **Owner:** Mantle maintainers.
- **Next action:** Commit the implementation, then run committed-source Cairn, Tracey, and full Nix checks.

## Non-claims

The evidence does not prove worker honesty, transport confidentiality, effect success without an observation, compiler correctness, output trust without admitted facts, global scheduling optimality, reproducibility, or release eligibility.
