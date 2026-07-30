# Protected transition v39 predecessor timeout

## Result

The create-new v39 and v40 transitions did not reach the TinyCC-to-musl bridge.

- v39 scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v39-tcc-musl-prep-20260728`
- v39 pueue task: `2786`
- v39 runtime: 995.78 seconds
- v40 scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v40-tcc-musl-prep-20260728`
- v40 pueue task: `2810`
- v40 runtime: 1344.52 seconds
- Exact repeated error: `building Mes-linked TinyCC: Mes process .../mes-stage/mes-0.27.1/bin/mes-m2 exceeded 300000 ms`

Both failures occurred while the existing protected Mes compiler rebuilt the TinyCC 0.9.26 predecessor. Both runs stopped before the new bridge stage. The retained plans and failure audits are diagnostics only. They are not resumable completion evidence.

## Next action

Raise the named Mes process budget from five minutes to ten minutes. Retry the same checked source and manifest from a fresh create-new scratch directory. Do not reuse v39 or v40 outputs.
