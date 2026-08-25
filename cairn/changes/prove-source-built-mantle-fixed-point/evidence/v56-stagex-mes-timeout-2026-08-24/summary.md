# V56 StageX Mes timeout

## Question

Did commit `8d0bbcb2` reach the repaired GCC subprogram boundary in a cold promoted proof?

## Inspected evidence

V56 used source profile BLAKE3 `5424edceda8a4cc29f205b331a0be21fbc03e9bca116a72d87d6898eef94b10a`, orchestrator BLAKE3 `af046b015bd41111d6b37b725f6fa61c15e349a5aaed784372cc801fa8669e8c`, strict hermeticity, no substitution, 16 jobs, and the 700 GB disk bound.

StageX failed while building Mes-linked TinyCC. The protected Mes process exceeded its named 600,000-millisecond limit. The attempt stopped before native construction and before the Rust-provider stage. The transition plan, failure audit, wrapper identity, plan, and attempt status remain preserved.

## Decision

Keep the StageX time limit unchanged. A single timeout does not authorize a weaker resource bound. Do not import partial V56 state. Retry the same immutable source, binary, and profile cold.

## Owner

Mantle StageX protected execution and promoted fixed-point orchestration.

## Next action

Reclaim only derived V56 state and obsolete scratch, then run V57 with all proof limits unchanged.
