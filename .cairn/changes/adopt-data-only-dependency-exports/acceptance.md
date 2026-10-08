# Change-local acceptance

a[dependency-exports.original-scope] The original proposal, design, tasks, metadata, and spec delta MUST be preserved byte-for-byte under `evidence/original-scope/` with the source commit and Git blob identities recorded, and their original tasks MUST remain unchecked.

a[dependency-exports.admission-review] The admission review MUST record the searched repositories, revisions, commands, and results against the workspace rule, MUST assess the self-declared `run-cargo-build-scripts-as-plan-units` candidate, and MUST state whether a current consumer, target outcome, adoption path, and maintenance owner exist.

a[dependency-exports.decision-record] The rejection MUST cite ADR 0109, including its revisit triggers and non-claims, without adding a second ADR.

a[dependency-exports.no-accepted-requirement] The change MUST carry no delta spec, MUST select the `no-spec-delta` profile, and MUST leave every `mantle.dependency_exports.*` requirement out of `.cairn/specs`; validation, all three gates, and an explicit no-op sync MUST pass.

a[dependency-exports.closure] A closure record MUST state that the change is closed as rejected at admission and not implemented, and the archive preflight MUST report no blockers before archive execution.
