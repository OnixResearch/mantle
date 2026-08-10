# ADR 0002: Dynamic Derivations

## Status

Accepted

## Context

Mantle inspects completed build outputs for `.drv` files.
It can register these files as new goals during the same Worker run.

The earlier path parsed and registered one mutable derivation value.
It used an all-zero parent hash when the registry lacked a referenced parent.
That fallback produced a deterministic but unsupported identity.
It could schedule work under a path that did not match the declared graph.

Native Mantle derivations use BLAKE3 and one explicit logical store prefix.
The Nix compatibility adapter uses Nix-required identity rules.
These identity domains must remain separate.

## Decision

Use a staged pure admission core before any registry or scheduler mutation.

The state sequence is:

```text
candidate bytes
  -> parsed dynamic derivation
  -> validated dynamic derivation
  -> identity-resolved dynamic derivation
  -> registry-ready dynamic derivation
  -> Worker-owned mutation
```

Each state has a private Rust type.
Only constructors in the pure core can create the next state.

### Detection and bounds

The Worker inspects bounded regular files whose store-path names end in `.drv`.
The core accepts the traditional `Derive(` prefix.
It also accepts `DrvWithVersion("xp-dyn-drv",` under the declared policy.

Named limits cover bytes, fields, collections, parent edges, dynamic nodes, parser collections, and depth.
Unknown versions, malformed input, excess depth, and excess size fail before identity work.

### Versioned input model

Versioned recursive requests use a flattened preorder model.
Each dynamic node records its parent index, output name, and requested outputs.
Parsing and traversal use explicit stacks instead of recursive calls.

The execution projection keeps direct outputs and top-level dynamic output names as parent dependencies.
The full native identity also binds the original versioned bytes.
Nested requests therefore cannot disappear from identity.

Versioned execution currently accepts only input-addressed outputs.
Other versioned output semantics fail with a stable unsupported-output blocker.

### Complete parent identity

The Worker observes one immutable hash fact for each direct parent.
Each fact binds the logical parent path, BLAKE3 digest, and native digest role.

Missing, duplicate, conflicting, unexpected, wrong-prefix, and wrong-domain facts fail closed.
The core never substitutes zero bytes or another sentinel.

Traditional covered derivations keep their prior BLAKE3 identity when all parent facts are complete.
Custom logical prefixes flow through parsing, hashing, path calculation, registry keys, and diagnostics.
Mixed-prefix candidates fail before registration.

### Registry and scheduler boundary

The pure core creates an insertion or exact-duplicate plan.
An exact duplicate must match the full admitted identity.
A path collision with another identity fails closed.

The Worker prepares every candidate before it applies any registry mutation.
Batch collisions fail before the first insertion.
Only `RegistryReadyDynamicDerivation` can enter the dynamic registry insertion method.

Goal creation, waiter changes, ready queues, scheduling evidence, and success reports remain after registry admission.
Any earlier failure leaves those states unchanged.

### Functional core and imperative shell

The pure core owns these decisions:

- candidate classification;
- syntax parsing;
- semantic and output validation;
- recursive request flattening;
- limit enforcement;
- parent-fact completeness;
- native BLAKE3 identity;
- configured-prefix path calculation;
- duplicate and collision planning.

The Worker shell owns these effects:

- castore discovery and bounded blob reads;
- registry observations;
- diagnostics and tracing;
- registry mutation;
- goal and waiter mutation;
- scheduler dispatch.

`scripts/check-dynamic-admission-boundary.rs` enforces this source boundary.

## Rejected alternatives

### Keep the zero-hash fallback

This choice makes unknown parents appear complete.
It was rejected because the calculated identity does not bind the declared parent graph.

### Use `nix-derivation` in the native core

This choice would mix Nix compatibility identity with Mantle-native identity.
It was rejected by ADR 0077 and the dependency source guard.

### Add evaluator suspension

Full import-from-derivation needs evaluator suspension and resumption.
That work is outside this bounded post-build admission change.

## Compatibility and rollback

Covered traditional fixtures retain their BLAKE3 identity and configured-prefix path.
The recorded baseline includes the former zero-fallback identity and path.

Rollback must restore the old core and Worker adapter together.
Rollback evidence must state that missing parents again receive an unsupported zero-hash fallback.

## Consequences

- Missing parent facts now block admission.
- Supported versioned requests have bounded iterative traversal.
- Registry insertion is atomic for each discovered output batch.
- Exact duplicates remain idempotent.
- Colliding identities fail before scheduler mutation.
- The Worker shell remains responsible for all effects.

## Claim boundary

Admission proves only the recorded parsing, validation, identity, path, and registry-plan facts.
It does not prove builder safety, sandbox enforcement, source trust, output correctness, scheduling quality, or release eligibility.
