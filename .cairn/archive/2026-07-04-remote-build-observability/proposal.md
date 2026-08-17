## Why

Operators need to understand distributed build decisions and failures without reading raw logs or leaking secrets. Remote status and build reports should explain route selection, upload plans, queue phases, transfer modes, trust roots, and non-claims in bounded machine-readable form.

## What Changes

- Extend remote status with redacted worker, queue, job, capability, ticket, transfer, and recent-failure summaries.
- Extend build JSON and human output with selected route, rejected remote reasons, remote endpoint, upload summary, transfer mode, delta/full byte counts, signer/trust basis, attestation paths, and non-claims.
- Bound logs, argv/path lists, and diagnostic payloads; keep bearer tickets, private key paths, raw env values, and uploaded content out of reports.
- Add reconnect/log-replay diagnostics that identify phase and retry class.

## Impact

- **Files**: remote status/report core, CLI renderers, JSON schema tests, log replay, docs, and Cairn remote-builds/realization-routing spec deltas.
- **Testing**: positive status/report fixtures; negative secret redaction, oversized log, invalid JSON regression, untrusted log control bytes, and overbroad claim fixtures.

## Out of Scope

- Metrics backend storage.
- Web dashboard UI.
- Claiming remote execution success from route-planning evidence alone.
