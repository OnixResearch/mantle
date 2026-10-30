# Design: additive pilot fixture materialization

## Boundaries

NASA source remains `e24cf09355a90497148eb5029fdb8e3400bd63e3`. The
existing source archive, Cargo lock, offline vendor closure, Rust 1.91.1,
reference profile, runner and eight outcome expectations are unchanged.

Two finite Nix authoring modules produce additive members. Host segmentation
uses checked-in WAT and a Nickel-authored descriptor; the full ABI family uses
Nix-authored WAT, also retained as exact portable inputs along with the authoring
module. The existing pinned wasm-tools parses each WAT, validates wasm1, prints
and reparses the binary and compares the readback bytes. The existing real
bundler measures source, descriptor and generated artifact bytes and binds them
in the reference manifest. No new digest or lockfile is written by hand.

## Consumer contract

The segmentation module has observe/pause imports, run and total exports. The
full ABI family has memory, alloc, dealloc, actor entrypoint and declared assert
hostcall signatures matching `molten.wasm.abi.v1`. Descriptor i64 packs high
pointer and low length. Positive modules echo canonical input supplied by the
consumer, avoiding a second canonical Preserves encoder in the producer.
Negative modules target output pointer, canonical output, fixed memory budget,
non-tail stack recursion and unreachable trap boundaries.

The existing reference runner is hostless and cannot execute these modules.
Descriptors explicitly require a consumer-owned execution receipt. Octet owns
artifact admission and ChaosControl/Molten own execution observations and
cumulative fuel semantics. Existing hostless profiles remain usable for their
original scope, never as a substitute for pilot fixtures.

## Integrity versus authority

`mantle-spacewasm-reference verify` remeasures a whole bundle and validates its
identity graph. It does not authenticate a publication. The release owner must
publish and pin an immutable producer revision and expected bundle identity.
Consumers must require the exact selected fixture paths and roles, compare the
externally admitted identity, and reject local builds without producer evidence.
Identical local bytes cannot be distinguished by an integrity hash alone.

## Validation

The existing negative check depends on new real-bundle controls for omitted,
modified and hostless-substituted pilot modules. Binary generation/readback,
independent complete-bundle verification, reference checks and consumer positive
and negative runtime scenarios must execute under the parent-owned integration
rail. This work records source-prefetch measurements, not unrun check receipts.
