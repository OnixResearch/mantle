# Validation Notes

## Spec reread

On 2026-04-14, after the final implementation landed, the synced main spec
`openspec/specs/build-pipeline/spec.md` was re-read against the implementation
in:

- `crates/crunch-build/src/build_request.rs`
- `crates/crunch-build/src/orchestrate.rs`
- `vendor/snix-build/src/bwrap/mod.rs`
- `crates/crunch-pipeline/tests/integration_build.rs`

Checked requirements/scenarios:

- `Requirement: Canonical execution envelope`
- `Scenario: Host locale and timezone do not leak into the build`
- `Scenario: Host umask does not leak into the build`

## Command validation

Same-turn validation evidence was recorded with:

- `cargo test -p crunch-build build_request --lib`
- `cargo test -p crunch-pipeline --test integration_build`
- `openspec validate normalize-build-execution-envelope`
