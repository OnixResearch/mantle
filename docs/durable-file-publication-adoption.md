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

At that revision, the generated JSON had BLAKE3
`4a7e35d9ac58740ce93f2c6c81a8b66fa3623f9a2aeed645c5b62c7914ad98ad`.
For the 2026-10-08 worktree remeasurement, regenerated JSON and adjacent
`.blake3` both bound `fd048e0ddc3356cf11a48bd661ae79bd02049c594a0bee4a4c64675f4dcf98f8`.

The Nix adoption check compares normalized Nickel export and JSON, their exact
digest, and current workspace file hashes. This maintenance does not alter the
governed source URL, RID, reviewed revision, locked NAR hash, producer evidence,
publication mapping, or authority boundary.
It does not establish upstream source correctness, a fresh public HTTPS fetch,
a successful full flake gate, self-hosting, release eligibility, or deployment
readiness.

The 2026-10-08 Tiger Style and Nix source-closure repairs, plus existing
workspace dependency changes, require refreshing three current bindings.
Direct BLAKE3 over the Leviathan worktree gives `Cargo.toml`
`0d016551043ed7884bc783432650c6fa5dce799bf4c14c321812b31b88ab28d6`,
`Cargo.lock`
`21eac824a09a21c6f47460ffe2e1e543486b1eef39e1cf37ffe614d075380c0b`,
and `flake.nix`
`098217fce30cb113ffd17feff36999374aedfa03886f2d6c07e0d05498b851de`.
`flake.lock` retained the historical digest at that remeasurement. The typed
receipt and independent validator bound those three then-current hashes; the
table above records the older revision. `observed_date` and the original
validation outcomes remain historical, not replayed adoption evidence.

On 2026-10-10, the Cairn pin and serialized Nix libtest update changed the
Leviathan worktree's `flake.nix` and `flake.lock`. Direct BLAKE3 over these
current files gives `d5e4dd4ac9b8ed26391e02014e10705cdc357654dc6d530d5d4ef8e56efb7667`
and `206b693fe08b816766359ceb1b776748309c953e094d807ed2eba62143a6cf74`,
respectively. Only the two Nix file-digest bindings in the typed receipt and
independent validator were remeasured; Cargo digests and the accepted adoption
fact remain unchanged. Regenerate the adjacent JSON with the pinned Nickel
command above, then refresh its `.blake3` sidecar.

The 2026-10-10 pinned Nickel 1.17.0 export of the updated receipt JSON has
BLAKE3 `3c3ec96556534cee7f39ed5adb35db8561e3c200eaa3d7a2a9db28286f7330e1`;
the adjacent `.blake3` sidecar recorded that historical generated byte digest.

The prepublication receipt-refresh candidate based on Mantle revision
`787c6af4dafc970e67bba0530e764cff0bb11196` passed both adoption Nickel
typechecks, the positive/negative receipt identity fixtures, and this focused
Nix check:

```sh
nix build .#checks.x86_64-linux.durable-file-publication-adoption -L \
  --no-write-lock-file --option substituters https://cache.nixos.org \
  --option builders ''
```

For that prepublication candidate, the full flake gate was not rerun; its
focused success did not establish full-gate success. The 2026-10-10 full gate
failed before this digest rebinding and is not claimed as passed.

After the 2026-10-10 Cairn pin and serialized root libtest cutover, the typed
receipt was regenerated with pinned Nickel 1.17.0 and its current flake/lock
digests validated by `checks.x86_64-linux.durable-file-publication-adoption`.
The root package check passed with 2,565 tests in each binary's unit-test leg;
the full x86_64-linux flake check completed `all checks passed!` with 6,262
nextest passes and 150 skips. These are bounded check results, not a new
adoption outcome, fresh upstream fetch, release claim, or deployment proof.

On 2026-10-12, the bounded Rust unit-execution and lock-vendor source-flow
change added `serde_json` to `mantle-lock-vendor-producer` in `Cargo.lock`.
The remeasured lockfile BLAKE3 is
`ce93be39bed03654a8d97219b58b00581504e95b4c623790eb4d83673ccc63ad`;
the other three file bindings were unchanged at that remeasurement.
The typed receipt and its independent validator bind the current lockfile hash.
This maintenance does not
change the reviewed publication source, historical adoption date or outcome,
or the authority and non-claim boundaries above.

The first pinned Nickel 1.17.0 export for this lockfile change produced JSON
with BLAKE3 `3dfdf6903c8bec130a8ed21af4e0961b71201278c9b384e8c8e734dde34431ac`;
the sidecar recorded that digest before the Nix source-closure repair below.

The full 2026-10-12 flake gate found the tracked
`fixtures/lock-vendor-two-crate/mantle.ncl` missing from filtered Nix test
sources. The `stdlib_tests` lock-vendor evaluation could not import it. Admitting
that fixture subtree changes the `flake.nix` BLAKE3 to
`f843962f4198a77939a9c3e6cf6fc64e60971a5172d38a4612cae1a104132cf7`;
the other three current file bindings remain unchanged. The typed receipt and
independent validator now bind this flake digest. Pinned Nickel 1.17.0
regenerated adjacent JSON with BLAKE3
`d51dce7621f396f4fec96375da411cfd11167e841e4cfd00c74f76be252ad13c`,
recorded in the `.blake3` sidecar. This source-closure maintenance changes
neither the original adoption outcome nor its authority and non-claim boundary.
On 2026-10-22, the API reconciliation and the standard-library shape repair
changed the workspace dependency and build inputs again. Direct BLAKE3 over
these current files gives `Cargo.toml`
`862dcb7150c8c80878b985ebf460aa1a56a68b1e82238c53ad97a03dfda1a27a`,
`Cargo.lock`
`76c8935edc9af74c2866c8532ff034d07434668d782ff32830fb41ee12116e4e`,
`flake.nix`
`506d6a26af501959daa7d0a326b17ad39ba2ed58d9180ea123b0c82aff9cc03b`,
and `flake.lock`
`a24deee6f99134cfdfb141b33fca4959c831651f6a1d7f3815d1b298e66416bc`.
All four file bindings were remeasured, and the typed receipt and the
independent validator bind these current hashes. Pinned Nickel 1.17.0
regenerated the adjacent JSON with BLAKE3
`43a6957c664ec31c58c2b9e1a04d9ae2a4c8db48934fbc155e17dacdd2a5d260`,
recorded in the `.blake3` sidecar. This workspace-hash maintenance changes
neither the reviewed publication source, the historical adoption date or
outcome, nor the authority and non-claim boundaries above, and it establishes
no fresh upstream fetch, no full-gate success, and no release claim.

On 2026-10-26, the sandboxed Clippy check exposed omitted compile-time Python
drivers in the Nix Cargo source closure. The initial `.py` admission repair
produced `flake.nix` BLAKE3
`fc1dc9796253503621b1e3cfd0d7ffdae9853378deae1569ee1efeeb5903748a`
and generated-receipt BLAKE3
`925db102bddc70681be2844c0702241614f21c29b6c5c0584e2727df8f9acd9c`.
Sandboxed Clippy and the adoption-receipt check passed for that repair, but
the full flake run exposed further omitted bootstrap C/header/`.inc` sources
and Rust templates. Those results do not establish a successful full gate.

A syntax-aware include audit then identified 249 literal file inputs, two
manifest-relative archived inputs, and one generated `OUT_DIR` input. The
complete source-closure repair admits embedded C/header/`.inc` sources,
`.rs.in` templates and Python drivers under first-party `src/` and `crates/`,
crate fixture/testdata trees, and the SpaceWasm report contract. A direct Nix
source smoke admitted all 251 file inputs byte-for-byte and excluded Git
metadata; the `OUT_DIR` input remains generated during the build.

The repaired `flake.nix` has BLAKE3
`2611482aa7c2128a17b0aad0e86189f2e9672fae41101b6884a0168dcf5c6f28`.
The typed receipt and independent validator bind that digest. The Cargo
manifest, Cargo lock and flake lock bindings are unchanged by this filter
repair. Pinned Nickel 1.17.0 regenerated the JSON with BLAKE3
`675e6dbfee2fd7b01274d0428424a3a5d7a53707da75e52b3b9235025cd37ba9`,
also recorded in the sidecar. This repairs source admission and receipt
freshness only. Historical gate failures remain failures; this remeasurement
does not by itself establish a successful replacement full gate, fresh
upstream fetch, bootstrap result or release eligibility.
