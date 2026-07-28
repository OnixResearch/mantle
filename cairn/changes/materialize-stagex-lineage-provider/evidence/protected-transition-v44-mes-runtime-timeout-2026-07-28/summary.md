# Protected transition v44 Mes runtime timeout

The create-new v44 transition stopped before the musl-linked TinyCC stage.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v44-tcc-musl-20260728`
- Pueue task: `2898`
- Runtime: 348.20 seconds
- Exact error: `Mes process .../mes-stage/mes-0.27.1/bin/mes-m2 exceeded 300000 ms`

This limit came from `stagex_mes_lib`, not the already raised transition-shell budget. The retained plan and failure audit are diagnostics only. They are not resumable evidence.

The next attempt must use a ten-minute named Mes library process budget and a fresh create-new scratch directory.
