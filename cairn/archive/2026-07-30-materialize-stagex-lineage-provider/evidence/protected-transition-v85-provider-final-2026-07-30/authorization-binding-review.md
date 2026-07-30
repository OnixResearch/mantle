# Authorization-binding review checkpoint

## Question

Can a complete stage report bind an executable authorization whose identity was not observed by the protected seccomp audit?

## Inspected evidence

- v85 `transition-plan.json` and `protected-exec-audit.json`
- `authorization-binding-gap.json`
- `StagexStageReport.executable_event_ids` validation in `crunch-bootstrap-core`
- `ProtectedSeccompAuditEvent` production in `protected_exec_seccomp.rs`
- VibeThinker adversarial review of the proposed path-plus-digest rule

## Decision

Reject the prior declared-set/raw-audit split. Require every declared authorization path and BLAKE3 digest identity to appear in an allowed protected `execve` or `execveat` audit decision. One event can satisfy identity-equivalent IDs because the policy deduplicates identical executable identities. Remove the 24 never-observed binutils authorizations instead of adding synthetic probe executions.

A protected audit event is created from an intercepted exec syscall. There is no separate `phase = executed` event. The receipt therefore claims an observed execution decision, not successful process completion or behavior.

## Owner

Mantle StageX provider publication and receipt validation.

## Next action

Commit the narrowed plan and strict receipt rule, run a fresh protected transition from that commit, publish twice, replace the scaffold receipt, and rerun parity and lifecycle gates.
