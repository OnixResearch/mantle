## ADDED Requirements

### Requirement: Resumable production transfer gallery

r[examples.resumable_remote_transfer_workflow] Mantle SHOULD provide a supported deterministic local gallery workflow that exercises the production remote-build streaming path across interruption and resume without promoting transfer completion into output trust or network-deployment claims.

#### Scenario: interrupted output resumes through production stdio
GIVEN a checked-in gallery project produces an output spanning multiple bounded transfer chunks
WHEN the debug validation rail interrupts after a durable production output acknowledgement and reruns the same fenced request in fresh client/server processes
THEN Mantle MUST request only content absent from verified receiver state, preserve the manifest BLAKE3 identity, and report reused bytes
AND equal-content chunks at different artifact-local indices MUST be matched to their own full descriptors rather than conflated by digest
AND the final output MUST pass ordinary signed PathInfo, content, requested-output, store-prefix, and attestation admission before success.

#### Scenario: tampered acknowledged receiver content blocks admission
GIVEN a production output checkpoint acknowledges a receiver chunk
WHEN the acknowledged receiver bytes no longer match their declared BLAKE3 identity before resume
THEN Mantle MUST reject or safely recompute without trusting the checkpoint cursor
AND the gallery negative rail MUST prove no output is admitted or reported successful from the tampered state.

#### Scenario: documentation preserves the validation boundary
GIVEN the gallery documents deterministic interruption and resume
WHEN an operator reads the runbook, catalog, or remote-transfer guide
THEN the docs MUST identify the interruption control as debug-test-only and name the production stdio client/server path actually exercised
AND they MUST NOT claim exactly-once delivery, production P2P or SSH deployment, general remote-worker honesty, output trust from transfer alone, or release reproducibility.
