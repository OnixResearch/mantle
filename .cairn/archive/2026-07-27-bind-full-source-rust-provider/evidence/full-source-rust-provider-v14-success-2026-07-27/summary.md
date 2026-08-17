# Full-source Rust provider v14

## Question

Did a fresh run materialize a complete Rust provider from the admitted full-source native provider and authenticated offline Rust sources?

## Inspected evidence

- Driver status: `0`.
- Run interval: 2026-07-27 14:54:37 through 19:00:04 local time.
- Provider: `/home/brittonr/.cargo-target/mantle-full-source-rust-provider-detached-v14-20260726`.
- Retained scratch: `/home/brittonr/.cargo-target/mantle-full-source-rust-scratch-detached-v14-20260726`.
- Provider metadata BLAKE3: `11766cf2be6892caf5b08fadf576baee90e112673d53f91afc1ab42206218d93`.
- Build receipt BLAKE3: `5e9efdf40ce362380ae0641254d470873907f53cda2aab44fbe1e3176b4cd757`.
- Binding receipt BLAKE3: `2c9987050e52d3d8830f0b2055c097a004811eed561555fa5fa71dbc7c315bbb`.
- Native-provider output BLAKE3: `f36d3759145d09b45ce9d45fcb832eeca3677e2e75527ef0d9e1553100acf66f`.
- Source-closure BLAKE3: `7e93ccc7a29bacc1da6f75c10ae90655ee83afd0d95ca282c8d270c225372f45`.
- Linux-header tree BLAKE3: `0fdac41832c7b489a3896fb70d7395110c7542a18bbf511432c78fe3757e3778`.
- Host-tool manifest BLAKE3: `40770f9c28943bd07b65c31d837bc19882ecde79155ec06157be4066c18b3dac`.
- Rust stages: mrustc to Rust 1.90.0, then Rust 1.91.1, 1.92.0, 1.93.1, and final Rust 1.94.0.
- Each retained stage has a plan, generated script, build record, provider receipt, provider metadata, and smoke summary.
- Each stage receipt binds the plan BLAKE3, script BLAKE3, `parallel-jobs=4`, native provider, source closure, admission report, host-tool manifest, and Linux-header tree.
- The final binding has `source_policy=authenticated-offline-only`, `ambient_tool_discovery=false`, no seed exceptions, and no fallback events.
- The built-in provider smoke compiled a target rlib with BLAKE3 `ddf23d812b8e6a5f805238cb377f8638d9d07513037c15a603267b88a5be3ab7`.
- A linked positive binary ran in an empty environment and printed `mantle-full-source-rust-v14-ok`. Its BLAKE3 is `bb031151252c51c0465a0cde3bc636c9bb97203ccb17ee5946f4ddcdc93649cc`.
- Malformed Rust source returned nonzero status and created no output.

`selected-evidence.blake3` records BLAKE3 values for the retained source manifests, stage logs, provider metadata, receipts, and linked smoke binary.

The first manual linked smoke used the target's default static-PIE mode. The source-built musl archive contains non-PIE objects, so that link failed before output publication. The accepted smoke uses explicit `-C relocation-model=static` with the receipt-bound source-built linker. It then ran successfully. The failed diagnostic is retained as `positive-static-pie-link-failure.stderr.txt.zst` and is not completion evidence.

## Decision

The v14 provider materialization is accepted as the real full-source Rust provider for this change. It does not use prebuilt Rust, Nix or rustup provenance, ambient tool discovery, imported-provider evidence, or a runtime fallback event.

## Owner

Mantle full-source bootstrap implementation.

## Next action

Run the current source tests, source-pin audit, formatting, Clippy, build, Cairn validation, and all three Cairn gates. Commit the implementation before archiving the change.
