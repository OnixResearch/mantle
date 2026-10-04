# Durable file publication adoption

Mantle consumes `durable-file-publication` from Radicle RID `rad:z3tAR4For7qw8ZirkJzoDw1VNDDLM` at exact revision `951c27f59003cea9bfdb40ed4d89653d50fada1f`. Cargo and Nix use the governed read-only HTTPS adapter. The RID identifies the repository. The revision identifies the reviewed source.

## Selected boundary

The shared shell publishes immutable remote-attempt segment and anchor files only. Mantle maps these facts into the shared request:

- One admitted destination leaf.
- The exact canonical JSON byte count.
- The Mantle-owned byte limit.
- Eight exclusive stage-name attempts.
- Final mode `0600`.
- Race-free no-replace.
- Required payload and parent synchronization.

Mantle opens the no-follow parent directory before it calls the shared shell. The shared shell then performs all file operations relative to that one capability.

## Outcome handling

`CommittedAndParentSynchronized` records a new immutable object. `DestinationExists` causes a bounded no-follow comparison in Mantle. Exact bytes are idempotent. Different bytes are an immutable conflict.

`CommittedDurabilityUnknown` means rename committed but parent synchronization failed. Mantle reports this state separately. It does not remove the destination or claim durable success.

Uncommitted results retain the primary failure and cleanup result. Cleanup failure cannot become success.

## Mantle-owned policy

Mantle retains canonical JSON, object identity, limits, content equivalence, manifest replacement, chain validation, retention, deletion, receipts, retry policy, and diagnostics. Replaceable manifests and release-bundle directories do not use this dependency.

The previous local immutable publisher remains as an explicit rollback backend. Production selects the shared backend. No automatic fallback runs after a shared-path failure.

## Platform and claim limits

The adoption executes Linux filesystem tests. Android is not executed. Other operating systems remain unsupported for this path.

The dependency proves only bounded mechanical classification over supplied observations. It does not prove source correctness, remote-attempt semantics, release eligibility, retention, recovery, garbage collection, or deletion authority.

## Adoption receipt freshness

The typed source for `evidence/radicle/durable-file-publication-adoption-v1.json`
is the adjacent `.ncl` file; `lib/durable-file-publication-adoption-receipt.ncl`
independently validates its four recorded file digests against expected values.
The receipt's `observed_date` records the original 2026-07-27 adoption, not a
date for later workspace-hash maintenance. At Mantle revision
`787c6af4dafc970e67bba0530e764cff0bb11196`, direct `b3sum` of the
unchanged published input files produced:

| Receipt binding | Workspace file | BLAKE3 |
|---|---|---|
| `cargo.manifest_blake3` | `Cargo.toml` | `20bd9ca126210da3552c86e1ab4a523b49c20b9e90644fbdefba33ba847edd56` |
| `cargo.lock_blake3` | `Cargo.lock` | `accad9f34300f46479f606967b6879b050bcdf6a7ee714d9f107cbc00eb3655b` |
| `nix.flake_blake3` | `flake.nix` | `358a31e6ab9c9fea664e95e3a78064822627570d023ee2102829e723386ca8c7` |
| `nix.lock_blake3` | `flake.lock` | `15cc9a3667d4b1a4439cefbf81a3a0cfcc39e8935eb3ebe51b5036e43caf02bf` |

The Cargo manifest binding was already current. The other three file bindings
and the same three independent validator constants were refreshed. Export JSON
from the typed source with the repository's pinned Nickel 1.17.0:

```sh
nix develop --no-write-lock-file -c nickel export --format json \
  --output evidence/radicle/durable-file-publication-adoption-v1.json \
  evidence/radicle/durable-file-publication-adoption-v1.ncl
b3sum evidence/radicle/durable-file-publication-adoption-v1.json
```

The generated JSON has BLAKE3
`4a7e35d9ac58740ce93f2c6c81a8b66fa3623f9a2aeed645c5b62c7914ad98ad`;
its adjacent `.blake3` holds this plain hex digest.

The Nix adoption check compares normalized Nickel export and JSON, their exact
digest, and current workspace file hashes. This maintenance does not alter the
governed source URL, RID, reviewed revision, locked NAR hash, producer evidence,
publication mapping, or authority boundary.
It does not establish upstream source correctness, a fresh public HTTPS fetch,
a successful full flake gate, self-hosting, release eligibility, or deployment
readiness.

The prepublication receipt-refresh candidate based on Mantle revision
`787c6af4dafc970e67bba0530e764cff0bb11196` passed both adoption Nickel
typechecks, the positive/negative receipt identity fixtures, and this focused
Nix check:

```sh
nix build .#checks.x86_64-linux.durable-file-publication-adoption -L \
  --no-write-lock-file --option substituters https://cache.nixos.org \
  --option builders ''
```

The full flake gate was not rerun for this refresh; focused success does not
establish full-gate success.
