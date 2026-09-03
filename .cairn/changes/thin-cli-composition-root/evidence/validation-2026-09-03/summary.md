# Focused validation summary

## CLI compatibility

- `src/main.rs` has 365 lines. The checked limit is 450 lines.
- The 12-case command corpus is byte-identical to the baseline.
- Both manifests have BLAKE3 `04993d806d317478dd5e4b6d5011e71afa158c2f572149f591605dfcbf9b6fa6`.
- Ten help cases returned status `0`.
- Two invalid-input cases returned status `2`.
- The recursive diff is empty.
- Pueue task `2816` built and compared the final binary after the compile-fail docs were made source-closure safe.

## Application tests

Pueue task `2830` recorded the exact final output in `focused-tests.log`.

- `mantle-application`: 3 passed.
- `mantle-application-core`: 5 passed.
- Compile-fail port cases: 2 passed.
- CLI architecture bridge: 6 passed.
- Moved compatibility application tests: 92 passed.
- All focused suites reported zero failures.

The tests cover deterministic planning, invalid commands, typed failures, missing ports, provider-type rejection, all 12 family ports, adapter faults, wrong observations, wrong error bindings, DTO mapping, mutation classes, presentation failures, and exact request binding.

## Architecture and quality

- Pueue task `2769`: the maintained architecture checker reported zero repository findings. It detected 13 adversarial source fixtures and two compile-fail fixtures.
- Pueue task `2823`: Nix application host tests, `wasm32-unknown-unknown` compilation, and the Nix architecture check passed.
- Pueue task `2820`: strict first-party Clippy passed with `-D warnings`.
- Pueue task `2824`: the exact Tiger Style Nix gate passed. No allowance, warning budget, baseline, or reduced scope was added.
- Pueue task `2708`: machine-contract generation and validation passed with 24 contracted and 57 classified surfaces.
- Pueue task `2730`: durable-file-publication adoption passed after exact Cargo and flake BLAKE3 bindings were refreshed.
- Pueue task `2728`: required Cargo formatting, both new crate formatting checks, and the diff whitespace check passed.
- Pueue task `2762`: Cairn validation, proposal gate, and design gate passed.
- Pueue task `2765`: Tracey reported 155/155 before the new delta spec was synchronized.

## Architecture result

The root now parses input, initializes requested tracing, calls one application entry, presents the result, and selects the exit status.

`mantle-application-core` owns deterministic command admission, effect identities, limits, and observation classification. `mantle-application` owns the 12 family ports.

The CLI operation adapter binds errors to the exact effect identity. It rejects command, family, or effect drift before it constructs a terminal outcome.

Clap, Snix, filesystem, process, async-runtime, environment, clock, random, network, provider, store-service, and rendering authority remain outside the no-std contracts.

## Oracle checkpoint

- **Question:** Does the split make the CLI root mechanical without changing accepted command behavior?
- **Inspected evidence:** The byte-parity corpus, 108 focused positive and negative tests, two compile-fail cases, architecture diagnostics, no-std builds, WASM builds, Clippy, Tiger Style, machine contracts, and durable-publication bindings.
- **Decision:** Accept the focused implementation. Keep command-family effects in the compatibility shell and outer adapters until a later family-specific extraction replaces them.
- **Owner:** Mantle maintainers.
- **Next action:** Commit the implementation, run committed-source lifecycle and Nix checks, synchronize the accepted requirements, and archive the change.

## Independent boundaries

The ordinary full flake check is still blocked by the known imported Rust-source hash mismatch. The local-builder full check also retains the known filtered-nextest source omission. These boundaries are independent of this CLI split.

The focused evidence does not prove external effect success, provider correctness, complete CLI semantics, deployment success, reproducibility, or release eligibility.
