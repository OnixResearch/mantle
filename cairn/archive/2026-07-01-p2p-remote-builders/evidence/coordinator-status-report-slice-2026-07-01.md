# Coordinator/status/report slice — 2026-07-01

## Question

Can the `p2p-remote-builders` change close the remaining coordinator, lifecycle, status, and JSON report gaps without claiming a production Iroh/libp2p transport?

## Decision

Implemented the remaining deterministic Mantle-owned core and shell surfaces for this change:

- pure coordinator worker registration with protocol-version checks, capability labels, sandbox/network modes, concurrency, signing-key identities, and resumable job summaries;
- normalized concrete-build request keys that exclude per-attempt transport/session identity;
- coordinator dispatch decisions for capability/trust matching, identical-request attach, finished-result redelivery, live-output conflict rejection, and unavailable restart-state phase reporting;
- bounded log retention/replay planning that drops/truncates instead of buffering unbounded slow subscribers;
- redacted coordinator status snapshots and a thin `mantle remote status` shell;
- stable reconnect and session-lease release decisions;
- remote client JSON rendering through the existing `crunch-build-report-v1` shape, including artifact attestation references and substitution mode/byte/fallback fields.

Production Iroh/libp2p remains a non-claim for this change. The implemented transport contract is the shared frame protocol plus deterministic loopback/stdio/SSH-stdio-compatible boundary; later production P2P can plug into the same `RemoteFrame` state machine.

## Evidence

Baseline before this slice:

```text
$ cargo test -p mantle --bin mantle remote_build::
test result: ok. 64 passed; 0 failed; 0 ignored; 0 measured; 966 filtered out; finished in 0.01s
```

Focused validation after this slice:

```text
$ cargo test -p mantle --bin mantle remote_build::
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 968 filtered out; finished in 0.02s
```

```text
$ cargo test -p mantle --bin mantle remote_client_json_report_uses_build_report_schema_and_substitution_fields
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1037 filtered out; finished in 0.00s
```

```text
$ cargo test -p mantle --bin mantle remote_status_cli_accepts_endpoint_and_concurrency
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1038 filtered out; finished in 0.00s
```

```text
$ cargo test -p mantle --test remote_stdio_cli
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

```text
$ rustfmt --edition 2024 src/main.rs src/remote_build.rs
completed successfully
```

```text
$ git diff --check
completed successfully
```

A first-party clippy probe was attempted but is blocked outside this slice by pre-existing `src/build_correctness.rs` `clippy::collapsible-if` diagnostics under `-D warnings`; no remote-builder-specific clippy diagnostics were surfaced before that pre-existing blocker.

## Post-archive validation

```text
$ CAIRN_ARCHIVE_DATE=2026-07-01 nix run path:/home/brittonr/git/cairn#cairn -- archive p2p-remote-builders --root . --execute && nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 8,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 19,
  "valid": true
}
```

Final same-turn validation after appending archived evidence:

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 8,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 19,
  "valid": true
}
```

## Next action

No active `p2p-remote-builders` action remains. Future production Iroh/libp2p transport work should open a separate change that reuses this archived protocol core.
