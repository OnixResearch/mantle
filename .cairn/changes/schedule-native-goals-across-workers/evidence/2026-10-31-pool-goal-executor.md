# Bin-side pool goal executor (2026-10-31)

Scope: execute one prepared pool goal on one admitted endpoint: build the
concrete request from the carried derivation, admit a client-side attempt, run one
framed stdio exchange, verify outputs, and complete the attempt. It owns no
placement policy and persists no output bytes; the pipeline's Builder stores the
returned nodes.

## Shape

- `concrete_pool_goal_request(goal, options)` parses the carried derivation,
  requires the declared drv path under the client store prefix, derives expected
  outputs/input refs/source refs/upload bytes from that derivation, and emits a
  `nix-derivation` payload request with the endpoint binding.
- `RemotePoolGoalExecutor::execute`:
  1. plan with `plan_remote_stdio_request_dispatch`;
  2. load the client coordinator state and admit the attempt through
     `admit_remote_production_dispatch` (worker registration + fenced attempt),
     then persist it;
  3. bind production transfer and prepare input NARs from the owned transfer
     capability;
  4. run one framed child exchange under an `EffectSession` for the attempt;
  5. verify with `validate_remote_stdio_output_import` (full output keys,
     attestation, digests);
  6. map admitted outputs to `BuildRequest.outputs` order by logical-path suffix
     and return their castore nodes as `BuildResult`;
  7. complete the attempt with the admitted output digest and persist the
     coordinator state.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p mantle --bin mantle pool_goal_executor::tests` | 1 passed (malformed derivation, outputless derivation, foreign prefix denied) |
| strict Clippy `mantle --bin mantle`, rustfmt | exit 0 |

## Not claimed

- No factory or CLI flag constructs this executor yet, and no live exchange is
  exercised; wiring and an end-to-end routing fixture remain.
- The executor does not import outputs into the store itself; persistence and
  signing stay with the Builder.
