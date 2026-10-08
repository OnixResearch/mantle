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

The typed policy is `config/cc-compile-cache/default.ncl`
(`mantle-cc-compile-cache-policy-v1`); its separate inventory contract is
`config/cc-compile-cache/contracts.ncl`
(`mantle-cc-compiler-inventory-v1`). A declaration of executable path and
BLAKE3 digest still requires runtime byte measurement and supervisor audit.

## Decision Drivers

- Reuse local daemon/state ownership without conflating C records and Rust castore results.
- Admit only explicitly inventoried C/C++ driver and real compiler identities.
- Avoid header-stale hits and store-path-dependent keys.
- Preserve derivation identity and byte parity with the daemon absent.
- Keep cache receipts outside fixed-point and release proof authority.
- Fail closed for unknown inputs but compile plainly if the daemon is down.

## Decision

Extend the existing **per-user** `mantle-rust-cache-daemon` and state authority
with a separately versioned C/C++ action/manifest mode. C object bytes and
manifests belong in per-record-bounded local file records at
`state_dir/cc-compile-cache`, not RustUnitAction, Rust's castore-backed
artifacts, its remote cache, or its shared retention policy. The initial
bounds are 1,048,576 object bytes, 262,144 depfile bytes, 8 candidates per
base key with FIFO eviction, 8,388,608 encoded frame bytes, and 4,096
manifest dependencies. There is **no global storage quota** or retention
guarantee for these C records. The driver is admitted only for the selected
bootstrap compiler families after exact driver
and real-compiler path and measured BLAKE3 identity checks against the session
inventory **and** protected-exec supervisor inventory. The daemon must perform
its own real-compiler authorization; a readable root or trusted peer UID alone
is insufficient. Unclassifiable invocations forward unchanged to that
compiler, not to an ambient PATH selection. A missing daemon degrades to a
plain compile with a recorded disposition.

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
identity remains eligible for reuse. Optional probe-result and cached-failure
execution is **not implemented** by the daemon; this v1 policy requires both
flags off. The core's probe domain and failure decision reserve future
semantics only. A later reviewed enablement must bind probe script,
toolchain, dependencies, platform, and flags, record every read disposition,
and refuse failure replay without a known complete manifest. A failed
compiler's depfile cannot prove negative-read completeness (a missing header
may appear later); do not mark its manifest complete or replay it from
depfile evidence alone. Independent read provenance is a prerequisite for
any later failure replay decision. Result presence is not proof of trust.

The cache socket must eventually be mapped into authorized sandboxes at
execution time, not bound into derivations or stored in a derivation-hashing
environment. **No sanctioned mapping seam exists in the present protected
sandbox**: cache use in that lane remains blocked until the build-shell owner
implements and audits one. Cache configuration must not introduce a writable
cache mount as an input. Strict fixed-point and release lanes must run
cache-off and record this mode; a cache receipt can never serve as rebuild
evidence. Until the strict lanes reject CC cache state in observed execution,
the corresponding proof acceptance remains open. Any alternate independently
verified proof boundary needs its own accepted decision and evidence.

## Alternatives Considered

- **Separate C/C++ daemon:** rejected because Rust's daemon already owns the
  local lifecycle and receipts. C/C++ nevertheless needs separate
  per-record-bounded files; Rust's castore action records cannot safely
  represent a C action.
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
