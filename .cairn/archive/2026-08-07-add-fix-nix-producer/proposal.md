# Add fix Nix producer

## Why

Mantle's nixpkgs support produces `foreign-derivation-graph-v1` artifacts through a host Nix frontend or through explicit `.drv` file bundles. Both paths depend on evaluation facts that Mantle did not produce and cannot rebuild: the host Nix binary, its version, and its evaluation behavior are ambient producer facts.

`fix` (https://github.com/psyclyx/fix) is a from-scratch parallel evaluator for the Nix language, written in Zig. It parses Nix source, compiles it to bytecode, evaluates lazily, computes derivations and store paths, and speaks the Nix daemon protocol. Its pinned nixpkgs differential evaluates the full CI job universe (about 80,000 derivations) to identical `.drv` store paths against a reference Nix.

Adopting `fix` as a pinned, Mantle-built producer frontend removes the ambient host-Nix dependency from nixpkgs artifact production. Evaluation becomes a receipt-bound Mantle build product with recorded compatibility evidence, while the stable foreign-import ABI stays unchanged.

## What Changes

- Pin the `fix` source tree as an ordinary Mantle source record with a fixed-output hash and a receipt-bound revision.
- Build the `fix` binary from the pinned source through Mantle's ordinary derivation pipeline, with the Zig toolchain and C library dependencies as explicit derivation inputs.
- Add a `fix` producer adapter that runs `fix` to evaluate Nix expressions and instantiate `.drv` closures, then emits `foreign-derivation-graph-v1` and `foreign-package-index-v1` artifacts through the existing direct-`.drv` producer path.
- Run the producer under an explicit bounded process policy with named time, memory, and output limits.
- Record bounded compatibility evidence (language suites, derivation differential results) as receipt data that names the exact pins and agreement counts.
- Keep consumption `fix`-free: validation, translation, planning, substitution, and realization consume accepted artifacts only.

## Dependencies

- The `foreign-derivation-import` spec remains authoritative for the import ABI, translation core, receipts, sandbox audit, and realization. This change adds one producer adapter and does not modify that ABI.
- The existing host-Nix producer adapter and the direct-`.drv` closure producer remain in place. The `fix` adapter reuses the direct-`.drv` producer path and does not replace the host-Nix adapter.
- Source pinning uses the existing Mantle source-record and fixed-output fetch machinery. No new source transport is introduced.

## Non-Goals

- Reimplementing, vendoring, or patching the `fix` evaluator inside Mantle's Rust codebase.
- Claiming semantic equivalence between `fix` and Nix beyond the recorded, pin-bound differential evidence.
- Replacing Nickel as Mantle's own configuration and derivation language.
- Running Nix expression evaluation inside Mantle validation, translation, planning, substitution, or realization paths.
- Using `fix` daemon-protocol store commands (`fix build`, `fix run`, `fix switch`) or adopting its REPL, debugger, and explorer surfaces.
- Building the Zig toolchain from source. A fixed-output binary Zig toolchain is the initial input.

## Impact

- **Affected specs:** new `fix-nix-producer` capability; consumes the `foreign-derivation-import` ABI without modifying it
- **Planned files:** `fix` source pin, Zig toolchain and `fix` build derivations, producer adapter module, bounded execution policy, compatibility evidence records, fixtures, and docs
- **Compatibility:** additive; the host-Nix producer and direct-`.drv` bundle inputs keep working unchanged
- **Testing:** pin mismatch, build provenance, adapter parity, evaluation failure, malformed output, budget exceeded, evidence bound, and Cairn gates
