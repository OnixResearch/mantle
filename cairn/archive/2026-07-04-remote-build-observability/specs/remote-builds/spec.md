## ADDED Requirements

### Requirement: Remote build status and reports are bounded diagnostic evidence

r[remote_builds.operator_observability] Mantle MUST expose remote-build status and build reports as bounded, redacted diagnostic evidence. Reports MUST identify route decisions, rejected-route reasons, worker capabilities, queue phases, active and recent jobs, upload classes/counts/bytes, transfer mode, transferred and reused bytes, fallback reasons, signer/trust basis, artifact-attestation references, log cursors, retry class, and non-claims when those facts are available. Reports MUST NOT reveal bearer tickets, private key paths, raw environment values, uploaded content, unbounded argv/path lists, or untrusted log payload as control data.

#### Scenario: status summarizes queue and workers safely

GIVEN a remote coordinator or builder has registered workers, tickets, queued jobs, active jobs, recent failures, and retained log cursors
WHEN an operator requests remote status
THEN Mantle MUST render bounded human and JSON snapshots of those facts
AND secret-bearing fields MUST be redacted or omitted.

#### Scenario: build report explains accepted remote output

GIVEN a remote build output is admitted through verified output trust
WHEN Mantle renders the build report
THEN the report MUST include selected route, endpoint id, upload summary, transfer mode, transferred bytes, reused bytes, fallback reason when present, signer or trust-basis identity, artifact-attestation path, and output identity
AND it MUST distinguish route eligibility, remote execution, output transfer, and output import claims.

#### Scenario: logs and diagnostics cannot become unbounded control data

GIVEN a remote build streams logs, reconnects clients, emits oversized diagnostics, or includes untrusted control-looking bytes in logs
WHEN Mantle records, replays, or reports those diagnostics
THEN Mantle MUST enforce configured byte, chunk, cursor, and redaction limits
AND slow subscribers or oversized logs MUST be truncated or failed according to policy instead of buffering unbounded data.
