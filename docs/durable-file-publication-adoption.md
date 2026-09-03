# Durable file publication adoption

Mantle consumes `durable-file-publication` from Radicle RID `rad:z3tAR4For7qw8ZirkJzoDw1VNDDLM` at exact revision `951c27f59003cea9bfdb40ed4d89653d50fada1f`. Cargo and Nix use the governed read-only HTTPS adapter. The RID identifies the repository. The revision identifies the reviewed source.

## Selected boundary

The shared shell publishes these immutable files:

- remote-attempt segments;
- remote-attempt anchors;
- source records;
- source-observation sidecars;
- source pins.

Mantle maps these facts into the shared request:

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

Mantle retains canonical JSON, object identity, limits, content equivalence, source-ingest policy, source readiness, manifest replacement, chain validation, retention, deletion, receipts, retry policy, and diagnostics. Replaceable manifests and release-bundle directories do not use this dependency.

The previous local immutable publisher remains as an explicit rollback backend. Production selects the shared backend. No automatic fallback runs after a shared-path failure.

## Platform and claim limits

The adoption executes Linux filesystem tests. Android is not executed. Other operating systems remain unsupported for this path.

The dependency proves only bounded mechanical classification over supplied observations. It does not prove source correctness, remote-attempt semantics, release eligibility, retention, recovery, garbage collection, or deletion authority.
