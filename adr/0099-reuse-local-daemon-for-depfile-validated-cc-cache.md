# ADR 0099: Reuse the local daemon for depfile-validated C/C++ compilation

## Status

Proposed (2026-10-01). This record defines the intended boundary; it does not
claim a completed bootstrap rebuild, cache-on/off parity run, or accepted
fixed-point proof. The `extend-compile-cache-to-cc` Cairn change carries the
implementation and acceptance evidence.

## Context

`crunch-rust-cache` and `mantle-rust-cache-daemon` already provide local unit
storage, action-result admission, and receipts for Rust. Bootstrap StageX and
GCC C/C++ objects still rebuild when a recipe changes. The reviewed repkgs
compiler driver uses depfiles to learn transitive dependencies and a daemon
to share results across build directories, but a local cache writer can inject
object bytes. Mantle additionally has a protected executable inventory and
strict fixed-point and release evidence lanes. A performance mechanism cannot
be smuggled into their proof authority.

The policy source is `config/compile-cache/default.ncl`: it owns the
`mantle-cc-driver-policy-v1` contract and the separately exportable
`mantle-cc-cache-contract-v1` description. Scheduling policy belongs to
`config/compile-slots/default.ncl`, not a derivation input. A declaration of
executable path or content identity does not replace protected-exec authority.

## Decision Drivers

- Reuse local daemon/state ownership without conflating C records and Rust castore results.
- Admit only explicitly inventoried C/C++ driver and real compiler identities.
- Avoid header-stale hits and store-path-dependent keys.
- Preserve derivation identity and byte parity with the daemon absent.
- Keep cache receipts outside fixed-point and release proof authority.
- Fail closed for unknown inputs but compile plainly if the daemon is down.

## Decision

Extend the existing **per-user** `mantle-rust-cache-daemon` and store authority
with a separately versioned C/C++ object/manifest protocol. Reuse the existing
castore and retention roots without pretending that C records are Rust unit
actions or enabling remote sharing. The shell bounds retained records to 256,
objects to 8,388,608 bytes, each diagnostic stream to 262,144 bytes, and learned
dependencies to 4,096. Existing record JSON admission remains separately
bounded. These bounds do not constitute a whole-store disk quota.

Admit the driver and actual compiler explicitly; never replace ambient
`CC`, alter `PATH`, or discover an alternative compiler on failure. Unknown
invocations forward the original arguments to the selected compiler. Missing
cache authority, malformed advisory records, or cache-publication failure must
not replace an actual compiler result with a cache error. Old draft records
without required output/depfile modes are misses, not a compatibility protocol.

A candidate key binds source bytes, measured tool identity, normalized
output-affecting arguments and selected environment. Depfiles from a miss
learn a complete set of stable logical dependency names and content digests.
The ordered include-search directory trees must also have complete,
bounded content identities so a newly shadowing header invalidates reuse.
On a proposed hit, the driver revalidates every dependency and each
include-search tree identity before deriving the final key. Neither an
absolute store path nor its basename can substitute for content identity.
If a complete manifest or safe path normalization cannot be established,
do not reuse the object or replay a failure. A dependency rebuilt at
another store path with identical content and the same stable logical
identity remains eligible for reuse. Probe identity, platform, flags and
tool/runtime contents participate in the key. Failure replay additionally
requires a known complete dependency manifest and complete bounded output
observations; a partial depfile alone is not negative-read provenance.
Unclassified probes, link operations, volatile time/date input and incomplete
manifests compile normally instead of inventing reusable results. Result
presence is not proof of trust.

An explicit local build option maps only a Unix socket to the fixed guest
`/run/mantle/compile-cache.sock` immediately before execution. The mapping and
build identifier do not enter derivation conversion, graph inputs, hashing,
remote dispatch or the normal derivation environment. The default executor
remains unchanged. Strict, offline-proof and self-build paths do not opt in;
inherited cache/scheduling authority is rejected rather than silently adopted.
Cache receipts cannot serve as rebuild evidence. See
[ADR 0116](0116-share-local-compile-cache-and-slot-authority.md) for the
same-daemon slot lifecycle and sandbox boundary. Source implementation alone
does not establish protected-exec parity, fixed-point acceptance or timing
benefit; both Cairn changes retain their executable acceptance obligations.

## Alternatives Considered

- **Separate C/C++ daemon:** rejected because the existing daemon already owns
  local lifecycle, bounded storage and retention. Protocol schemas distinguish
  C objects from Rust actions without creating a second authority or store.
- **Only compiler command line and source digest as the key:** rejected because
  changed headers and include-search shadowing can return stale objects.
- **Hash every file in the entire source tree, including directories never
  searched by this compile:** rejected as unnecessary work. The selected
  boundary does require complete bounded fingerprints of *ordered effective
  include-search directory trees* as well as a depfile manifest; if either
  cannot be measured, bypass caching rather than guess resolution.
- **Ambient PATH/CC or sccache-style wrappers:** rejected in protected lanes
  because they evade exact executable inventory and audit authority.
- **Cache directory bind-mounted as an input:** rejected because it makes
  local cache availability part of the derivation's input boundary.
- **Treat content-addressed object bytes or a signed cache receipt as proof:**
  rejected; content identity detects byte drift but cannot certify compilation
  or authorize a release.

## Consequences

The local writer remains able to inject object code, and neither a digest nor
a cache hit proves source correctness, compiler correctness, complete input
closure, determinism, fixed-point convergence, or release eligibility. Cache
misses and bypasses will occur for unknown flags, incomplete depfiles,
path-sensitive output, or changed include resolution. That reduced hit rate is
preferable to a false identity claim. Acceptance requires negative wrong-key
controls, exact protected-exec audit observations, cache-on/cache-off output
byte comparison, and a separate uncached fixed-point proof.
