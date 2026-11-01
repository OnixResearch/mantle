# Request dispatch planner (2026-10-31)

Scope: plan one concrete remote request for a pool goal executor, without the
Nickel closure. The request carries the `nix-derivation` payload, expected
outputs, and transfer policy.

## Change

- `plan_remote_stdio_request_dispatch(request, options)` validates the client
  options and request, requires the request's store prefix to match the client
  options, and returns the stdio dispatch plan labelled `remote-pool-goal`.
- `plan_remote_stdio_replay_dispatch` now shares the same internal planner after
  its replay-specific checks (no stale authority, replay binding present),
  keeping one construction path.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --bin mantle remote_build::tests::request_dispatch_plans` | 1 passed |
| strict Clippy (mantle targets), rustfmt | exit 0 |
| `cairn validate --root . --strict` | 0 issues |

The `remote_build` test module reports 205 passed / 1 failed; the failing
`attenuated_admission_tests::denial_reason_codes_appear_only_at_debug_level`
fails identically on the parent commit and passes when run alone, so it is a
pre-existing order/environment failure, not a planner regression.

## Remaining gap

The executor still needs the per-goal attempt and coordinator lifecycle: the
production receiver validates the attempt binding against its own coordinator
state, so a goal executor must create and drive attempts through the existing
protocol (or the application ports) rather than mint them locally. That
lifecycle, plus the CLI flags and routing fixture, remains.
