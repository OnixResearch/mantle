# Integration merge evidence

Date: 2026-09-03

`origin/main` advanced to `50120c1dff95c654e8823cf9ae09edb8e03b9661` after the Radiance branch was first published. The branch merged that revision in commit `62940810416e5cd061c9af6b79826c12b08e1158`.

Both branches had assigned ADR 0116. The merge preserves mainline ADR 0116 and renumbers the Radiance decision to ADR 0117. This is a documentation identity repair only.

The incoming gateway added two root JSON producer files without a machine-contract inventory decision. The integration adds one explicit compatibility-family classification and a public marker. Contract generation also refreshed three producer identities affected by the incoming CLI changes. Check mode now passes with 30 contracted and 64 classified surfaces.

The final merged validation passed:

- Radiance core: 14 tests.
- Radiance shell: 8 tests.
- Remote gateway focused tests.
- Strict Radiance core Clippy.
- Strict first-party root Clippy with the documented `--no-deps` scope.
- Root binary build.
- V16 receipt replay with the same fixed-point, receipt, and audit identities.
- Machine-contract and Radiance architecture checks.
- Cairn validation and Tracey coverage.
- Formatting and diff checks.

`commands.attempt1.log` records the known vendored `fuse-backend-rs` Clippy findings from the unsupported broad dependency scope. `commands.attempt2.log` records the stale mainline machine-contract inventory boundary. `commands.log` is the passing final run.

No proof was relabeled or rerun. V16 remains bound to implementation commit `5e35c8a518e871cbbf844598b274ddb7842c9316`, and V98 remains unchanged.
