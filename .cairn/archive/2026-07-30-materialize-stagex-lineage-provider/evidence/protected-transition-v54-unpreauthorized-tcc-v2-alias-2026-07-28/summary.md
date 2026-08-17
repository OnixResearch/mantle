# Protected transition v54: unpreauthorized TinyCC musl-v2 alias

The create-new v54 transition reached the TinyCC self-host build. Seccomp then rejected `tcc-musl-v2-stage/runtime/output/bin/tcc` because the supervisor had authorized the exact versioned compiler path instead.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v54-tcc-selfhost-20260728`
- Pueue task: `3296`
- Runtime: 1385.67 seconds
- Exact error: `spawning .../tcc-musl-v2-stage/runtime/output/bin/tcc: Permission denied (os error 13)`

The retained plan and failure audit are diagnostic evidence only. The repair executes the already-bound versioned path `bin/tcc-0.9.27-musl-v2`; it does not broaden the authorization set.
