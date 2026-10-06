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

## Historical receipt and current source pin

The accepted typed receipt at
`evidence/radicle/durable-file-publication-adoption-v1.ncl`, its JSON export,
and the JSON BLAKE3 sidecar record the bounded adoption with
`observed_date = "2026-07-27"`. The four Cargo/Nix whole-file digests in that
receipt describe files captured for the historical evidence; unrelated later
manifest, lock, or flake maintenance does not change the reviewed source or
make those recorded digests current-workspace requirements. Keep the receipt
bytes and the independent historical validator constants in
`lib/durable-file-publication-adoption-receipt.ncl` unchanged rather than
redating or refreshing the accepted observation.

The `durable-file-publication-adoption` Nix check typechecks the typed receipt,
tests its positive and negative evidence fixtures, compares its normalized
export with the checked-in JSON, and checks the sidecar digest. Separately it
checks the **current** Cargo dependency and lock source against the exact
reviewed repository and revision, evaluates the live
`inputs.durablePublicationSource.url` Nix declaration, and checks both Nix
lock URLs, revisions, and NAR hash. It rejects executable GitHub and sibling
path fallbacks. An unrelated-comment flake fixture must still pass the scoped
source check; a wrong-revision fixture must fail it.

```sh
nix build --offline --no-link --no-write-lock-file \
  .#checks.x86_64-linux.durable-file-publication-adoption
```

This checks source selection and historical receipt integrity, not upstream
source correctness, a fresh HTTPS fetch, full-flake success, self-hosting,
release eligibility, or deployment readiness.
