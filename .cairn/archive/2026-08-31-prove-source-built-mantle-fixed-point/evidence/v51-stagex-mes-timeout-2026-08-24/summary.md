# V51 StageX Mes timeout

## Question

Did V51 reach the repaired Rust-provider action boundary?

## Inspected evidence

- source commit `6c73b940`
- verified source profile BLAKE3 `0717fe3b24afc1113a2a5a95a69e011d8cbedd4002e8e294cc2c6bd69a1e8a2d`
- detached wrapper PID `1726253` with `/proc` start ticks `11598643`
- proof plan BLAKE3 `0a8e4c651e896d5a3ee47198fa25099b738bb4c4bd8a9bbc47a8e24377b5ceb6`
- wrapper and preserved attempt status

## Decision

V51 did not reach native or Rust-provider construction. It failed closed in StageX while a Mes process built the Mes-linked TinyCC. The process exceeded the existing named `600000 ms` limit. V50 had completed the same StageX path under the same source lineage and host. One timeout does not justify weakening the bound.

## Owner

Mantle StageX execution shell and promoted-proof operator workflow.

## Next action

Retry the same immutable source, profile, binary, limits, strict hermeticity, and no-substitution contract as V52. Preserve V51. Change the limit only if repeated complete evidence proves the existing bound invalid.
