# ADR 0117: Bound Radiance as an optional external reference

## Status

Accepted

## Context

Radiance has a small self-hosting compiler, a C99 bootstrap implementation, and
an RV64 emulator. It can give Mantle an independent bootstrap comparison.
However, its Git repositories use SHA-256 object identities, and compatible
revisions must be selected as one cohort. A mutable branch name is not enough.

A direct build also has too much ambient authority. It can discover a compiler,
linker, emulator, source checkout, or network path from the host. Equality from
such a build is useful research, but it is not proof evidence.

## Decision

Mantle keeps Radiance as an optional external-reference fixture. Normal builds
and release acceptance do not depend on it.

Connected preparation admits exactly three clean Git SHA-256 checkouts. Each
checkout must match its repository URL, commit, canonical source BLAKE3, and MIT
license BLAKE3. Preparation writes one complete source bundle and one canonical
source-cohort record. The later proof accepts both identities explicitly and
performs no source fetch.

The proof has two separate routes. The seed route starts with the admitted RV64
seed. The C99 route starts with a compiler built from the admitted
`radiance.s0` source. Both routes build three explicit outputs. Each output
names its immediate predecessor and the exact `.` source projection. Stages two
and three must match inside each route.

The operator supplies the C compiler launcher, its exact compiler driver, the
linker, the CRT directory, and the libgcc directory by absolute path. Mantle
hashes each executable and both canonical runtime trees before use. It verifies
the runtime tree identities again after the build. Each translation unit is one
protected compiler root. The direct `ld.lld --threads=1` invocation is a
separate protected root. Ptrace validates each executable before execution. A
child seccomp filter traps network syscalls. Successful execution therefore
records zero live source requests. The same policy protects the built emulator,
C99 compiler, and generated stage predecessors.

Mantle publishes both route-local fixed points, the C99 compiler, the emulator,
the raw protected-execution audit, and a sealed receipt. Cross-route divergence
remains valid evidence, but it does not have a successful proof disposition.

## Consequences

- Source acquisition and proof execution are separate operator actions.
- Replay can validate the receipt and immutable artifacts without a network or
  the original checkouts.
- A changed source, compiler driver, link-runtime tree, predecessor, stage
  output, audit, or publication fails exact-identity validation.
- The fixture adds explicit operator work and Linux ptrace/seccomp requirements.
- A compatible future Radiance cohort needs a new reviewed profile and receipt.

## Non-claims

Route-local or cross-route equality does not prove compiler correctness, seed
trust, semantic equivalence, or universal reproducibility. Mantle does not
transfer release authority, source ownership, or default-build policy to the
Radiance projects.

## Rejected alternatives

### Patch incompatible upstream revisions

Rejected. A source rewrite would measure Mantle's fork instead of the selected
upstream cohort.

### Fetch during the proof

Rejected. A live fetch would make the source boundary depend on proof-time
network state.

### Use PATH discovery

Rejected. A command name does not bind executable bytes or lineage.

### Treat equal outputs as compiler-correctness evidence

Rejected. Exact equality is a bounded convergence observation only.
