## Context

`run_build_with_stream_output` already owns stdout, the bounded event receiver, and `EvaluationCancellation`. It currently selects only pipeline results and stream records. Default process signal behavior can therefore terminate the process before the pipeline classifies unresolved roots.

Success requires observable subprocess evidence: after one supported signal, stdout ends with one admitted `run-summary`, its disposition is `cancelled`, every root is terminal, and the process returns status 130. A repeated signal must not produce false success or wait without a bound.

False completion includes a test-only cancellation call, a process killed by the default signal action, a cancelled status without a summary after the first signal, or signal handling that changes non-stream commands.

## Decisions

### Decision: Keep signal effects in the stream CLI shell

**Choice:** Register platform signal listeners only inside explicit evaluation-stream execution. Feed signal observations into the existing cancellation handle.

**Rationale:** The stream shell already owns output and process status. The pure core must not register signals, spawn tasks, or inspect process state.

### Decision: Use a pure observation-count policy

**Choice:** Add a pure decision that maps the first signal to cooperative cancellation and later signals to forced interruption.

**Rationale:** This keeps first-versus-repeated signal policy deterministic and directly testable without process-global handlers.

### Decision: Listen directly through Tokio

**Choice:** On Unix, select `SIGINT` and `SIGTERM` with `tokio::signal::unix`. On other targets, use Tokio's portable Ctrl-C listener.

**Rationale:** Tokio is already built with its full feature set. A new signal-handler dependency or a global main-level handler adds authority without improving this bounded stream path.

### Decision: Keep listening after the first signal

**Choice:** The shell continues to monitor signals while it drains records and waits for the pipeline. A second signal returns the cancelled process status immediately.

**Rationale:** Tokio's process signal registration can suppress the default action. Ignoring later signals could make a blocked shutdown impossible to stop.

## Approach Registry

| Family | Mechanism | State | Evidence or blocker |
|---|---|---|---|
| stream-shell-tokio | Mode-local Tokio listeners and cooperative cancellation | active | Existing runtime and cancellation handle are available. |
| global-main-handler | Install one handler for every command | rejected | Expands behavior beyond stream mode and cannot preserve command-local output authority. |
| signal-hook-thread | Add a global atomic handler and helper thread | rejected | Adds a dependency and process-global state without a required benefit. |
| default-process-exit | Keep operating-system termination behavior | rejected | Cannot produce complete terminal accounting after the first signal. |

## Risks / Trade-offs

- Signal handlers are process-global after registration. Restrict registration to the short-lived stream command.
- A second signal can end output without a summary. This is an explicit forced-interruption path and never reports success.
- Subprocess tests can race. Read `run-start` before sending the signal, use enough roots to enforce channel backpressure, and apply a bounded wait.
- Signal names differ across platforms. Test Unix signals under `cfg(unix)` and retain a compile-safe portable Ctrl-C path elsewhere.
