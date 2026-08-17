# Native observation v2: transition-source symlink blocker

## Result

Pueue task `7420` failed closed after `6,184s`.
The repaired protected StageX transition completed and retained its full execution tree.
The proof then rejected the first absolute symlink while it tried to import that complete tree as a source.
It did not publish the StageX provider or start native-provider construction.

The complete transition tree had 75 symlinks.
Fifty-five were absolute proof-local tool or logical-prefix links.
Eleven of those absolute links were intentionally unresolved outside their install namespace.
Two relative libtool links used a parent component.
Strict source admission correctly rejected these forms.

## Decision

ADR `0052` separates the complete execution evidence from the runtime handoff.
The proof now executes StageX under `stagex-transition-execution/` and preserves that tree without mutation.
StageX provider publication reads the complete tree.

The reserved logical transition path is a create-new projection of exactly eight declared runtime output directories.
It excludes binutils scratch, negative fixtures, and tool-namespace links.
The final receipt binds the complete execution tree, `transition-report.json`, and `protected-exec-audit.json`.

## Validation

Positive and negative tests cover exact projection, preservation of excluded execution data, missing-input rejection before destination creation, and normalized provider identity.
Pueue task `7739` passed the focused shell tests.

Pueue task `7763` projected the completed retained transition and imported it through strict source admission.
The projection retained four safe relative runtime aliases and contained zero absolute or parent-relative links.
The original execution tree remained present.

The three focused receipt tests passed in pueue task `7793`.
Strict first-party Clippy and focused `git diff --check` passed after the final edits in pueue task `7519`.

## Non-claim

This repair proves only that the completed StageX execution can supply a bounded, strictly admitted runtime handoff.
It does not prove native-provider admission, the Rust provider, the Mantle fixed point, or release eligibility.
