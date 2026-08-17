# Remote route plan validation

Task-ID: distributed-route-planning
Covers: r[realization_routing.remote_route_plan_cli]
Date: 2026-07-04

## Baseline

Direct `cargo` was unavailable on this host (`pueue task 190`: `sh: line 1: cargo: command not found`), so the focused baseline used the repository dev shell.

`pueue task 197`:

```text
Command: nix develop -c cargo test -p mantle --bin mantle realization_routing

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 1151 filtered out; finished in 0.00s
```

## Focused unit validation

`pueue task 211`:

```text
Command: nix develop -c cargo test -p mantle --bin mantle realization_routing && nix develop -c cargo test -p mantle --bin mantle build_plan_remote_facts

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 1152 filtered out; finished in 0.00s

test tests::build_plan_remote_facts_are_pure_and_redacted ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1170 filtered out; finished in 0.00s
```

## Positive CLI plan fixture

`pueue task 216` captured `mantle --json build --plan examples/hello.ncl --builder builder-1 --ticket ticket-1:super-secret --trusted-builder-key builder-key --no-substitute` into `/tmp/mantle-remote-plan.json` and asserted the ticket secret was absent.

```text
"selected_route": "p2p-remote-builder",
"selected_reason_code": "builder-capability-and-output-trust-match",
"selected_detail": "endpoint=builder-1; capabilities=stdio-default; upload_objects=0; upload_bytes=0; output_trust_keys=1; non_claim=route-eligibility-only",
ticket_secret_redacted=true
```

## Negative CLI plan fixture

`pueue task 217` captured the same plan without `--trusted-builder-key`, expected a non-zero status because both local preflight and remote output trust failed, and asserted the ticket secret was absent.

```text
"reason_code": "output-trust-missing",
negative_plan_status=1
ticket_secret_redacted=true
```
