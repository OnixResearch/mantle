# Add Nix producer adapter

## Why

Mantle's nixpkgs support produces `foreign-derivation-graph-v1` artifacts through a host Nix frontend or through explicit `.drv` file bundles. The producer frontend is implicit in those paths: there is no versioned contract that names what a Nix producer must accept, what it must emit, or which evaluator implementation stands behind it. Swapping the evaluator means editing producer plumbing instead of selecting a backend.

`fix` (https://github.com/psyclyx/fix) is a from-scratch parallel evaluator for the Nix language, written in Zig. Its pinned nixpkgs differential evaluates the full CI job universe (about 80,000 derivations) to identical `.drv` store paths against a reference Nix. It is a strong candidate producer, but adopting it directly would hard-code another implicit frontend.

This change introduces a versioned, backend-neutral `nix` producer adapter. The adapter owns the producer contract: expression inputs in, concrete `.drv` closures plus producer identity facts out. `fix` is the first pinned, Mantle-built backend. The existing host-Nix path becomes an explicit ambient backend. New backends (Lix, `snix-eval`, a future evaluator) plug into the same contract without touching the import ABI.

## What Changes

- Define a versioned `nix-producer-v1` backend contract: bounded expression and argument inputs in, `.drv` closure directory and producer identity facts out.
- Make backend selection explicit policy data. Unknown or unavailable backends fail closed. No ambient fallback.
- Add the `fix` backend: pin the `fix` source as a fixed-output record, build the binary through Mantle's ordinary derivation pipeline with an explicit Zig toolchain input, and run it under a bounded process policy.
- Keep the host-Nix backend as an explicit selectable backend with its ambient trust posture recorded in receipts.
- Route every backend's `.drv` output through the existing direct-`.drv` closure producer, so `foreign-derivation-graph-v1` and `foreign-package-index-v1` artifacts stay unchanged.
- Record bounded compatibility evidence per backend as pin-bound receipt data.
- Keep consumption evaluator-free: validation, translation, planning, substitution, and realization consume accepted artifacts only.

## Dependencies

- The `foreign-derivation-import` spec remains authoritative for the import ABI, translation core, receipts, sandbox audit, and realization. This change adds a producer adapter layer and does not modify that ABI.
- The existing host-Nix producer code and the direct-`.drv` closure producer remain in place. The adapter wraps them behind the backend contract.
- Source pinning uses the existing Mantle source-record and fixed-output fetch machinery. No new source transport is introduced.

## Non-Goals

- Reimplementing, vendoring, or patching any Nix evaluator inside Mantle's Rust codebase.
- Claiming semantic equivalence between any backend and Nix beyond recorded, pin-bound differential evidence.
- Replacing Nickel as Mantle's own configuration and derivation language.
- Running Nix expression evaluation inside Mantle validation, translation, planning, substitution, or realization paths.
- Using `fix` daemon-protocol store commands (`fix build`, `fix run`, `fix switch`) or adopting its REPL, debugger, and explorer surfaces.
- Building the Zig toolchain from source. A fixed-output binary Zig toolchain is the initial input.
- Adding Lix or `snix-eval` backends in this change. The contract admits them later without schema changes.

## Impact

- **Affected specs:** new `nix-producer-adapter` capability; consumes the `foreign-derivation-import` ABI without modifying it
- **Planned files:** producer contract core, backend policy, `fix` source pin, Zig toolchain and `fix` build derivations, backend shells, bounded execution policy, compatibility evidence records, fixtures, and docs
- **Compatibility:** additive; existing host-Nix and direct-`.drv` inputs keep working, now addressable as explicit backends
- **Testing:** contract conformance, backend selection, pin mismatch, build provenance, backend parity, evaluation failure, malformed output, budget exceeded, evidence binding, and Cairn gates
