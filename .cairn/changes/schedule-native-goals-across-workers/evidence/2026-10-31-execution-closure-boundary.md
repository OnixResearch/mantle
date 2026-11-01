# Execution closure boundary (2026-10-31)

Scope: why `build --builder-pool` can admit, select, attach, and dispatch a goal
but cannot yet complete a routed build, and what the envelope must carry to fix
it soundly.

## Verified boundary

- The pool executor sends a single `nix-derivation` payload (root derivation +
  declared inputs).
- The receiver's local-build executor (`local_derivation_registry`) registers the
  recursive `CrunchDerivation` closure and converts it; it rejects the
  single-derivation payload with
  `remote-local-executor-nix-derivation-unsupported`.
- Registering a single derivation is not sound without its inputs' identities:
  `nix_compat::derivation::Derivation::hash_derivation_modulo` requires each
  direct input derivation's HDM, and the Worker holds only the root derivation;
  uploaded inputs are store paths without HDMs.
- The Worker cannot send the closure instead: conversion in `crunch-pipeline`
  reduces `CrunchDerivation` to `nix_compat::Derivation` and does not retain it.

This is the ADR's stated reason for a prepared envelope: "A bare Snix
BuildRequest is insufficient if conversion lost these bindings; extend the
existing prepared execution envelope rather than infer them afterward."

## Options

1. **Carry the HDM/derivation closure in the envelope.** The Worker resolves the
   direct input derivations' HDMs (from stored `.drv` files or the registry) and
   the envelope carries them; the receiver registers the root derivation with
   those identities and executes from uploaded inputs. No Nickel closure needed.
2. **Retain the Nickel closure per goal.** Conversion keeps the source
   `CrunchDerivation` (in memory or a store sidecar) and the executor sends it;
   the receiver keeps its current path. Requires retention plumbing and, for
   store-loaded dependency goals, a durable sidecar.
3. **Content-addressed action executor.** Add a worker execution path that runs
   one action from declared inputs and expected outputs without HDM/registry
   identity (Buck2-style). Largest change; bypasses current Builder identity.

Option 1 matches the ADR's envelope intent with the least new state; option 2 is
the smallest conceptual change but needs retention; option 3 is the cleanest
long-term action model.

## Not claimed

- No routed build has executed; the flags, admission, selection, attachment, and
  dispatch are verified only up to the worker's payload rejection.
- Nothing here changes the receiver's rejection; it remains fail-closed.
