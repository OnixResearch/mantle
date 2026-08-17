# Protected transition v50: unpreauthorized TinyCC 0.9.26 path

The create-new v50 transition reached TinyCC musl-v2. Seccomp then rejected the exact versioned TinyCC 0.9.26 host path because that path was absent from the supervisor preauthorization set.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v50-tcc-musl-v2-20260728`
- Pueue task: `3222`
- Runtime: 1477.60 seconds
- Exact error: `spawning .../tinycc-stage/runtime/output/bin/tcc-0.9.26: Permission denied (os error 13)`

The retained plan and failure audit are diagnostics only. The repaired attempt must preauthorize the exact absolute path, BLAKE3, and `tinycc-final-materialization` producer stage before protected execution.
