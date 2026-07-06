## ADDED Requirements

### Requirement: Freshness probe offline proof rail emits bounded versioned evidence

r[project_workflows.freshness_probe_proof_rail] Mantle MUST provide a bounded local offline proof rail that exercises built-in, command-bounded, and network-requiring freshness probes through `mantle list-stale`, `mantle refresh`, and `mantle check` in no-network default mode, without ambient network services or hidden global state, and MUST emit a versioned, redacted, non-overclaiming evidence record that classifies each input as stale, unchanged, failed, skipped, or network-required, binds the observed value digest and probe kind, and states that freshness is not source integrity, trust, build success, or reproducibility.

#### Scenario: offline rail composes probe list-stale refresh and check

GIVEN a project fixture declares built-in, command-bounded, and network-requiring freshness probes
WHEN the offline proof rail runs `mantle list-stale`, then `mantle refresh`, then `mantle check` in no-network default mode
THEN `mantle list-stale` MUST report per-input status (stale, unchanged, failed, skipped, or network-required) without writing the lockfile, generated inputs, store state, or retention roots
AND `mantle refresh` MUST update only selected stale inputs plus required patches or trust material and exit non-zero on any input or patch resolution failure
AND `mantle check` MUST report soundness without contacting the network.

#### Scenario: network-requiring probe is not run in no-network mode

GIVEN a project input declares a network-requiring freshness probe
WHEN the rail evaluates it in no-network mode
THEN Mantle MUST report the probe as network-required or unavailable without contacting the network
AND it MUST NOT treat the old lock value as freshly observed.

#### Scenario: command probe failures are deterministic

GIVEN a command freshness probe times out, emits oversized output, emits invalid UTF-8 where UTF-8 is required, has a missing executable, or exits non-success
WHEN the rail runs the probe
THEN Mantle MUST report a deterministic probe failure
AND the evidence record MUST classify the input as failed or skipped without overclaiming freshness.

#### Scenario: evidence is versioned redacted and non-overclaiming

GIVEN the rail emits its evidence record
WHEN the record is rendered
THEN it MUST carry a stable schema version, per-input status, observed value digest, and probe kind
AND it MUST omit raw environment values, uploaded content, and unbounded logs and MUST NOT claim source integrity, trust, build success, or reproducibility from freshness alone.
