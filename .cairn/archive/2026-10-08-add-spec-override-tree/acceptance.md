# Change-local acceptance

a[spec-override-tree.original-scope] The original proposal, design, tasks, metadata, and spec delta MUST be preserved byte-for-byte under `evidence/original-scope/`, with the source commit and Git blob identities recorded, and their original tasks MUST stay unchecked.

a[spec-override-tree.admission-review] The admission review MUST record the searched repositories, revisions, commands, and results against the workspace rule, and MUST state whether a current consumer, target outcome, adoption path, and maintenance owner exist.

a[spec-override-tree.decision-record] The rejection MUST be recorded in ADR 0109, with its `adr/README.md` index row, revisit triggers, rejected alternatives, and non-claims.

a[spec-override-tree.no-accepted-requirement] The change MUST carry no delta spec, MUST select the `no-spec-delta` profile, and MUST leave every `mantle.spec_override_tree.*` requirement out of `.cairn/specs`. Validation and all three gates MUST pass, and sync MUST be an explicit no-op.

a[spec-override-tree.closure] A closure record MUST state that the change is closed as rejected at admission, not implemented, and the archive preflight MUST report no blockers before archive execution.
