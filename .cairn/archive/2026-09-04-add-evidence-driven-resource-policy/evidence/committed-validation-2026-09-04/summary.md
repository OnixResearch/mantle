# Committed resource-policy validation

Date: 2026-09-04
Implementation commit: `e0b25b5b`

## Passing evidence

- Formatting and `git diff --check` passed.
- `crunch-resource-policy-core`: 15 positive, negative, replay, property, benchmark, and fault tests passed.
- `crunch-resource-policy`: 6 application-port and Valence tests passed.
- Root adapters: 3 focused tests passed.
- Coordinator integration: 1 focused placement and lease test passed.
- Fixture integration: 6 source, Nickel, machine-contract, benchmark, OnixOS, and ChaosControl tests passed.
- Both resource-policy crates compile for `wasm32-unknown-unknown`.
- Strict Clippy passed for both new crates with all targets and all features.
- Strict first-party root Clippy passed with the documented `--no-deps` boundary.
- The resource-policy architecture checker passed with zero findings and 12 negative fixtures.
- Machine contracts passed with 32 contracted and 66 classified surfaces.
- Nickel typecheck and deterministic export comparison passed.
- `nixfmt --check flake.nix` passed.
- The four Nix checks passed: architecture, core, core-WASM, and integration.
- `nix flake check --no-build -L --option secret-key-files ''` evaluated all checks successfully.
- Cairn validation, proposal gate, design gate, tasks gate, and pre-sync Tracey coverage passed.

## Broad-scope results

`cargo test --workspace --offline` returned 101 after one test could not find `bwrap` in its command environment. The exact test passed after the documented bwrap path was restored. The resource-policy tests had already passed in the broad run.

`cargo clippy --workspace --all-targets --offline -- -D warnings` returned 101 in vendored `fuse-backend-rs` and `nix-compat`. The supported strict first-party root scope and both new crates passed.

## Octet status

The installed scoped Octet run completed with exit status 0 but is warning-only. It reports 237 core findings and 22 application findings. The findings are in the full inventory-only families, chiefly searchable imports, core construction heuristics, path repetition, and file/module size. No warning budget, baseline, or disabled lint was added.

The immutable pinned Octet revision could not run. Nix rejected the known rust-src import:

- specified: `sha256-q/gu/3mAuLgNfJlxV/Sw1jttbi4PIBjN+XH0bGmB5NQ=`
- got: `sha256-WTRv7eyiu+VOfb8+90cALNJrUa3uLwRFIaeEr+tAIjQ=`

This evidence does not claim a strict Octet pass. The existing lint policy was not weakened.

## Evidence boundaries

The checks prove the recorded deterministic policy, adapters, schemas, fixtures, and bounded integration paths. They do not prove future resource sufficiency, fair billing, host isolation, build correctness, production performance, source trust, or release eligibility.
