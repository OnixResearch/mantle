# Design: Extend the compile cache to C and C++ builders

## Goal and scope

The existing Rust unit cache seam gains a C/C++ compiler-driver mode for the
bootstrap toolchain family. The cache stays a machine-local performance
device; proofs stay cache-off. This proposal defines the contract; it does not
implement it.

Planning success means a native change package with requirements, ownership,
positive and negative tasks, and a recorded baseline. Gate success proves
package structure only.

## Current behavior

`crunch-rust-cache` stores unit results in castore-backed form with local
policy, receipts, and dispositions; `crunch-rustc-wrapper` plus
`mantle-rust-cache-daemon` serve Rust builds through `rust_plan` with
`local_rust_cache`/`shared_rust_cache` CLI modes. Nothing caches C/C++
compiles. StageX and GCC chain sessions recompile every object on recipe
change; proof runs measure hours. The accepted `rustc-cache-adapter` spec
already demands strict evidence lanes exclude ambient wrapper caching; the
present change extends the same discipline.

The external reference implements the C-side shape: a driver that is `cc` on
PATH, content-masked keys, depfile-learned manifests, cached failures and
configure probes, and a bitcask daemon with machine-wide build slots
(`evidence/repkgs-review.md`). Its trust model is explicit: whoever can write
the cache can inject object code; CA outputs make tampering detectable, not
impossible.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| sccache-style wrapper | Environment wrapper around each compiler | Rejected for protected lanes: ambient PATH mutation outside inventory | Supervisor audit parity |
| Declared driver seam | Policy-admitted driver in the tool inventory, forwarding what it cannot classify | Selected direction | Forward-unchanged and inventory fixtures |
| Wrapper inside sandbox with cache bind | Bind cache dir into sandbox as input | Rejected: makes the cache a derivation input | Byte-identity with cache on/off |
| Host daemon via mapped socket | Execution-time endpoint mapping, not a derivation input | Selected storage framing | Identical outputs with and without |

## Contract and component ownership

- Pure core: key normalization, manifest comparison, admission decisions, and
  typed dispositions in a new `*-core` module mirroring
  `crunch-rust-cache-core`.
- Shell: the driver binary (thin dispatch, forwards to the real compiler), the
  daemon extension in the existing daemon, receipt writing, and the
  sandbox endpoint mapping in the build shell.
- Policy: typed Nickel export admitting the driver per toolchain family with
  explicit non-claims.
- Storage: the existing local cache store; no remote sharing in this change.

## Decisions

### Decision: Reuse the existing daemon and store

**Choice:** Extend `mantle-rust-cache-daemon` and the local store instead of a
new daemon.

**Rationale:** The seam, receipts, dispositions, retention policy, and strict
lanes already exist and are gated; a second daemon would duplicate them.

### Decision: Depfile-learned manifests, not argument-only keys

**Choice:** Keys include the learned set of dependency file identities.

**Rationale:** Argument-only keys miss header edits — precisely the rebuild
case the chain needs to catch. The reference's manifest design covers this and
is testable with positive and negative fixtures.

## Risks / Trade-offs

- Trust: cache writes can inject object code; mitigation is per-machine
  per-user scope, receipts, and proof-lane exclusion, matching the reference's
  own framing and Mantle's evidence rules.
- Driver classification must track compiler flag churn; unclassifiable
  invocations forward unchanged and are logged, never guessed.
- Probe-cache keys over toolchain identity need the identity set the protected
  inventory already records.
