# `crunch-store` strict repair evidence

## Verdict

The targeted store repair is complete. The focused and repository Tiger Style
runs contain no `crunch-store` finding. The repository gate advances to 19
`crunch-build` findings and one `crunch-rustc-wrapper` finding.

No Tiger allowance, disabled lint, warning budget, finding baseline, or target
scope reduction was added.

## Counts

- focused baseline: 183 findings across 13 `crunch-store` files;
- prior repository baseline: 139 location-backed findings across the same files;
- accepted store result: zero findings;
- post-change tests: 357 unit and 2 integration tests, zero failures;
- strict `crunch-store` Clippy: pass;
- Mantle binary compatibility check: pass;
- formatting and diff checks: pass;
- `nix flake check --no-build -L`: pass.

## Next strict blocker

The repository Tiger derivation exits 1 after it passes `crunch-store`:

- `crates/crunch-build/src/build_request.rs`: 9 findings;
- `crates/crunch-build/src/execution_profile.rs`: 5 findings;
- `crates/crunch-build/src/ca_plan.rs`: 3 findings;
- `crates/crunch-build/src/registry.rs`: 1 finding;
- `crates/crunch-build/src/worker.rs`: 1 finding;
- `crates/crunch-rustc-wrapper/src/lib.rs`: 1 finding.

The local full check emits all 20 locations. The ordinary full check emits a
16-location subset because Cargo stops parallel units after the same build
family fails. Both runs contain zero `crunch-store` location.

The exact diagnostics remain actionable. This change does not suppress or repair
them.

## Architecture audit

- Store lifecycle cores still own deterministic policy and decisions.
- Shell helpers retain filesystem, service, network, publication, and mutation
  effects.
- GC still plans before mutation and preserves operation order.
- Provenance traversal uses explicit worklists and bounded container parsers.
- Invalid external data remains on typed rejection paths.
- Layer indices use cross-platform `u32` values at the public boundary.

## Portfolio result

- **Invariant-preserving local repair:** validated.
- **Functional helper extraction:** validated where functions exceeded the
  strict length or interface limits.
- **Policy suppression:** rejected.
- **Adversarial checks:** malformed inputs, path escape, stale metadata,
  incomplete closure, trust failure, overflow, publication failure, and
  container bounds remain covered.
- **Budget used:** nine of ten focused Tiger rounds. One added round verified
  post-Clippy source adjustments.

## Non-claims

This evidence does not claim a green repository Tiger gate, a green full flake
check, complete store correctness, safe deletion, content trust beyond measured
facts, release eligibility, or resolution of the later build findings.
