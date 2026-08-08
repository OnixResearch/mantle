# Backend shell and parity evidence

The producer implementation is in `src/nix_producer.rs`, `src/nix_producer_shell.rs`, and `src/foreign_import_cmd.rs`.

The focused contract suite covers request admission, backend selection, identity facts, success acceptance, closure collection, evaluation failure, malformed output, timeout, missing closure members, output and count limits, and unspawnable binaries.

```text
cargo test -p mantle --bin mantle nix_producer
→ test result: ok. 32 passed; 0 failed
cargo test -p mantle --test foreign_import_cli
→ test result: ok. 16 passed; 0 failed; 1 ignored
```

The ignored test `produce_backend_fix_and_host_nix_emit_parity_artifacts` passed once with the required `fix` binary, `nix-instantiate`, and daemon. Both backends emitted identical closure basenames and graph structure. Only producer identity fields differed.

The bounded shell rejects unknown backends, unavailable binaries, unsupported systems, daemon-required realization commands, malformed output, timeouts, oversized output, missing closure members, count overflow, and wrong-domain identities. Consumption uses the emitted artifacts and does not locate or launch a producer.

The memory limit remains disabled by default because `fix` reserves large virtual address ranges for its parallel garbage collector. This evidence does not claim RSS enforcement, evaluator correctness, nixpkgs-scale parity, or realization readiness.
