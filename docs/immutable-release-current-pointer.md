# Immutable release objects and the current pointer

Mantle defines one bounded release-layout contract. Each release object has a BLAKE3 content identity and immutable bytes.

The current pointer contains one lowercase BLAKE3 identity and one trailing newline. The named object must exist before the shell changes the pointer.

## Contract layers

`crunch-release-core` owns the pure contract:

- It compares the declared BLAKE3 identity with the supplied object bytes.
- It accepts an existing object only when its bytes are identical.
- It plans an initial pointer, a pointer switch, or an unchanged pointer.
- It plans rollback as a switch to a previous object identity.
- It binds metadata, the object role, and the pointer role in canonical evidence.
- It rejects deployment, retention, deletion, distribution, and release-readiness claims.

The `release_current_pointer` shell owns local file effects. The caller supplies the object, pointer, and evidence paths.

The shell requires the object file name to equal its BLAKE3 identity. It creates a new object without replacement.

The shell writes the pointer through a synchronized temporary file and an atomic rename. It reads the pointer again before it writes immutable evidence.

If immutable evidence already exists, its bytes must match exactly. The shell never replaces different evidence bytes.

## Rollback

Keep the previous object identity from the pointer plan. To roll back, select that identity as the next pointer target.

Rollback changes only the pointer. It does not change or delete either release object.

## Authority boundary

The caller owns these effects:

- distribution
- deployment
- retention
- deletion

Mantle records the object and pointer binding. This evidence does not prove deployment, release readiness, durable retention, or deletion.

The Celld release layout is a bounded design reference. Mantle does not claim installer parity or implementation equivalence.
