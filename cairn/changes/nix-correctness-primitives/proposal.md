## Why

Mantle can become a stronger no-Nix-runtime backend for OnixOS only if it grows the correctness invariants that make Nix trustworthy: declared build actions, explicit input closure, content-addressed outputs, reference scanning, sandbox policy, deterministic receipts, and admissible output reuse. The goal is not to clone Nix or interpret Onix/NixOS modules. The goal is to make Mantle's build-tool boundary capable of carrying Nix-like correctness evidence for frontend-owned artifacts.

Onix can then lower evaluated Nickel/Onix semantics into Mantle action specs and consume Mantle receipts as evidence. Mantle remains frontend-neutral and proves only generic build/action/store facts.

## What Changes

- Define a generic derivation-like action spec with declared inputs, toolchains, args, environment, outputs, platform, sandbox policy, network policy, and expected reference policy.
- Define BLAKE3 action refs and content-addressed object refs that do not rely on path names for identity.
- Add CAS object-store admission semantics for produced outputs and frontend-supplied input objects.
- Add hermetic execution policy and fail-closed blockers when the requested sandbox/network policy cannot be enforced.
- Add output reference scanning so built artifacts can prove their runtime references are declared and policy-compliant.
- Add deterministic action receipts and reuse/substitution admission that binds action refs, object refs, reference-scan results, sandbox policy, and producer policy.
- Preserve the build-tool boundary: Mantle MUST NOT learn Onix roles, tags, providers, Nickel contracts, or NixOS module semantics.

## Impact

- **Files later**: action/CAS DTOs, build execution planning, artifact admission, receipt structs, CLI reports, tests, examples/docs.
- **Spec domains**: `build-correctness`, `build-tool-boundary`, and `verification-evidence`.
- **Compatibility**: Existing Mantle examples and Rust planning may remain supported through narrower evidence. New correctness claims require the new receipts.
- **Cross-repo dependency**: Onix consumes these primitives through its `onixos-mantle-correctness-envelope` change.

## Non-goals

- Do not implement the Nix language, Nix derivation format, Nix store protocol, NixOS module system, or Onix module semantics in Mantle.
- Do not claim compiler correctness, source-to-binary reproducibility, full bootstrap correctness, or physical-target determinism.
- Do not make path roots identity. Paths may be views over content-addressed objects.
- Do not serialize decrypted secret bytes in action specs, object manifests, reference scans, receipts, logs, or diagnostics.

## Verification Expectations

Implementation is not archive-ready until focused tests prove canonical action refs, CAS object admission, hermetic policy blockers, output reference scanning, receipt binding, reuse/substitution rejection on stale evidence, and frontend-neutral boundary preservation. Evidence must include current command output for focused tests, `cargo fmt`, relevant workspace tests, `cairn validate --root .`, and proposal/design/tasks gates.
