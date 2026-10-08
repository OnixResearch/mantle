# Native dynamic derivation admission

Mantle can inspect a completed build for bounded regular `.drv` outputs.
It admits selected metadata before it changes the registry or scheduler.

## Supported forms

The native boundary accepts these ATerm prefixes:

- `Derive(` for covered traditional derivations;
- `DrvWithVersion("xp-dyn-drv",` for the bounded versioned policy.

Versioned recursive requests use a flattened preorder model.
The current versioned execution subset accepts input-addressed outputs only.
Unsupported output forms fail before identity resolution.

## Limits

The default policy names these limits:

| Limit | Value |
|---|---:|
| Candidate bytes | 4 MiB |
| One field | 1 MiB |
| Collection entries | 16,384 |
| Direct parent edges | 4,096 |
| Dynamic nodes | 16,384 |
| Dynamic depth | 64 |
| Parser collections | 65,536 |

These limits are local admission limits.
They are not general package, build, or scheduler limits.

## Parent identity

Every direct parent needs one explicit Mantle BLAKE3 hash fact.
The fact must use the selected logical store prefix.

Admission rejects these parent-fact conditions:

- missing;
- duplicate;
- conflicting;
- unexpected;
- wrong prefix;
- wrong hash role.

Mantle does not use an all-zero fallback.
External parents need a separate admitted compatibility boundary before native admission.

## Stable blocker classes

Diagnostics start with one stable class:

- `dynamic-admission-invalid-policy`;
- `dynamic-admission-platform-limit`;
- `dynamic-admission-limit`;
- `dynamic-admission-depth-limit`;
- `dynamic-admission-syntax`;
- `dynamic-admission-semantic`;
- `dynamic-admission-unsupported-version`;
- `dynamic-admission-unsupported-output`;
- `dynamic-admission-empty-request`;
- `dynamic-admission-mixed-prefix`;
- `dynamic-admission-missing-parent`;
- `dynamic-admission-duplicate-parent`;
- `dynamic-admission-conflicting-parent`;
- `dynamic-admission-unexpected-parent`;
- `dynamic-admission-wrong-parent-prefix`;
- `dynamic-admission-wrong-hash-domain`;
- `dynamic-admission-identity`;
- `dynamic-admission-path-collision`.

A blocker prevents registry-ready state.
The Worker then leaves registry, goal, waiter, ready-queue, and report state unchanged.

## Duplicate and collision behavior

An exact duplicate must match the full admitted identity.
It is an idempotent no-op.

The same path with another identity is a collision.
Mantle rejects the complete candidate batch before its first insertion.
Discovery order does not select a winner.

## Hash domains

Native derivations keep Mantle BLAKE3 identity and configured-prefix paths.
Nix compatibility imports keep Nix-required identity rules in the reviewed adapter.
No implicit conversion crosses these domains.

## Lock-driven native Cargo vendor plan

The `mantle-lock-vendor-producer` reads bounded `Cargo.lock` bytes and a
canonical table containing only hash rows selected for that lock. It emits
one native fixed-output `builtin:fetchurl` unit per external package, named by
its package/version/hash identity. The fetch unit has no source-archive edge:
adding an unrelated locked package, a new reviewed row, or changing the
assembler cannot rewrite the existing package fetch identity.

For up to 240 external packages, the plan has exactly the package fetch
units plus **one** final vendor assembler; no shard units or synthetic
dependency chain intervene. Beyond 240, bounded shard units group fetch
edges so no assembler exceeds the 256-input execution limit. The final unit
must verify every locked package and `.cargo-checksum.json` and use only the
declared Python interpreter, selected table, source bundle, and offline
Rust/Cargo toolchain. Missing reviewed Git transports, fixed hashes, or
tool dependencies are blockers, never ambient compiler/network fallbacks.

Native registration treats source outputs as placeholders until they are
realized, while retaining the full direct parent hash facts and bytes for
completed parents. Exact duplicate plans are idempotent; conflicting
dynamic batches or a collision with a static derivation fail before
registry/goal/ready-queue mutation. A source profile's separately recorded
vendor-tree receipt proves neither that the native plan was admitted nor
that the final store output has a trusted signature: verify these as
separate steps.

## Rollback

Rollback must restore `dynamic.rs`, `registry.rs`, and the Worker adapter together.
The rollback revision restores the former zero-parent fallback.
Operators must record that risk if they use the rollback.

## Validation

Run the focused boundary checks:

```text
cargo test -p crunch-build --lib --tests
cargo -Zscript scripts/check-dynamic-admission-boundary.rs --self-test
cargo -Zscript scripts/check-dynamic-admission-boundary.rs
```

## Non-claims

Admission proves only the recorded metadata checks and registration plan.
It does not prove builder correctness, source trust, sandboxing, output correctness, scheduler quality, or release eligibility.
