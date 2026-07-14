## Why

Mantle reports remote phases, bounded logs, transfer facts, trust outcomes, and recent failures, while the active `persist-remote-attempt-observability` change is adding immutable attempt logs and telemetry. Operators still lack one portable, redacted artifact that gathers the exact declared action, route, worker capability, input, sandbox, log, transfer, workspace, and admission facts needed to understand or replay a failed remote build.

Ephemeral sandboxes make remote failures especially difficult to diagnose. Preserving whole workspaces by default would leak secrets and consume unbounded storage, so debug preservation must be explicit, content-addressed, bounded, and separate from build truth.

## What Changes

- Add `mantle-remote-failure-debug-bundle-v1`, a bounded manifest over immutable refs for the failed action, route/assignment, current attempt/fence, worker capabilities, declared inputs, sandbox/network policy, logs, transfer/admission state, workspace mode, failure phase/reason, and non-claims.
- Export metadata-only bundles by default; include selected failed-sandbox artifacts only under explicit typed capture, sensitivity, byte/file, and retention policy.
- Redact bearer material, private key paths, raw secret values, undeclared environment values, host paths, and unbounded argv/log/content lists before bundle publication.
- Add inspect and replay-plan operations. Replay reconstructs a new declared execution request from immutable refs and policy; it never resumes stale authority or treats prior failure data as trusted output.
- Route replay through ordinary local/remote planning, sandbox enforcement, transfer, output admission, and action-result policy.
- Integrate immutable attempt log refs when available without duplicating the active observability change.

## Impact

- **Surfaces**: remote status/reporting, immutable attempt logs, sandbox failure cleanup, CAS/store roots, typed Nickel debug policy, operator CLI, and evidence bundles.
- **Dependencies**: depends on `persist-remote-attempt-observability` for immutable attempt log refs; optionally records stateful-workspace facts without requiring mutable workspace support.
- **Non-claims**: no hosted UI, live SSH/GUI attachment, trusted timestamp, automatic secret capture, byte-for-byte sandbox resurrection, output trust from replay, or proof that replay reproduces the failure.
- **Validation**: canonical bundle tests, redaction/escape/quota negatives, failed remote fixture export, fresh-host inspect/replay plan, replay divergence reporting, retention/GC tests, and Cairn gates.
