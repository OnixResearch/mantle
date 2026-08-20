# ADR 0079: Separate source fixed-point evidence from working scratch

## Status

Accepted (2026-08-20)

## Context

The first complete source-built Mantle fixed-point attempt produced matching
stage1 and stage2 binaries. Receipt construction then applied the ordinary
release-tree copy policy to `stagex-transition-execution/`.

That policy admits at most 4,096 entries and preserves only internal relative
links. The protected StageX evidence tree contains 83,046 entries. It also
contains absolute and parent-relative links from build scratch and negative
fixtures. ADR 0052 requires the receipt to bind this complete tree without
admitting it as runtime source.

The complete proof working root also contains Rust compiler scratch, native
store state, Cargo-free execution intermediates, proof home state, and sandbox
temporary directories. The V26 root contained more than 2.5 million files and
links. Some overlay work directories were intentionally unreadable after
execution. These bytes are useful attempt diagnostics, but they are not durable
proof evidence.

## Decision Drivers

- Bind the complete protected StageX execution tree without following links.
- Keep absolute and escaping links rejected from source admission.
- Keep receipt work bounded by the proof resource policy.
- Do not let unreadable or very large working scratch block durable evidence.
- Reject unknown unreadable content instead of silently omitting it.
- Give changed digest semantics a new domain identity.

## Decision

Mantle uses a separate observation-only tree digest for preserved proof
evidence. The shell opens the tree through a no-follow capability root. It
records paths, entry kinds, modes, file lengths and bytes, and exact symlink
target bytes under explicit entry, path, depth, target, and total-byte limits.
It never resolves a symlink target and cannot authorize copying or source use.

Ordinary release-tree source admission remains unchanged. Tests require it to
reject the same absolute and parent-relative links that the evidence observer
can bind.

The source-built proof-bundle digest uses an explicit durable projection. It
excludes only these declared working directories:

- `cargo-free-fixed-point/execution`
- `home`
- `native-state`
- `rust-provider-scratch`
- `tmp`

The complete `stagex-transition-execution/` tree remains in the durable
projection. All other files, links, and directories remain covered. Empty files
are valid evidence members and receive their normal BLAKE3 identity. Unknown
or unreadable content outside the declared working directories fails closed.

The bundle digest domain is
`mantle-source-built-fixed-point-proof-bundle-v2`. The preserved StageX tree
digest uses `mantle-preserved-evidence-tree-v1`.

## Alternatives Considered

### Increase the ordinary release-tree limit

Rejected. It would weaken unrelated source and release-copy policy. It would
still reject the StageX tree's intentional absolute and escaping links.

### Bind only the eight-directory StageX runtime handoff

Rejected by ADR 0052. The handoff is runtime source authority, not complete
transition evidence.

### Hash the complete working root

Rejected. It binds irrelevant compiler and sandbox scratch, requires reading
intentionally inaccessible directories, and makes receipt work depend on more
than 180 GB of transient state.

## Consequences

- V26's exact 83,046-entry StageX tree can be bound without source admission.
- V26's durable proof projection contains 136,135 files and links and stays
  below the existing two-million-entry proof limit.
- Working scratch can remain on disk for diagnostics, but its bytes are outside
  the durable proof identity and all release claims.
- Release packaging must use the durable projection when it later transports
  this source-built proof. This ADR does not claim that generic release-tree
  copying can transport the complete working root.
- The receipt still does not prove compiler correctness, seed correctness,
  kernel isolation, independent reproducibility, or release eligibility.
