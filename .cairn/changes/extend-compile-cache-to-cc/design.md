# Design: Extend the compile cache to C and C++ builders

## Goal and scope

The existing Rust unit cache seam has an opt-in C/C++ compiler-driver mode.
The cache stays a machine-local performance device; proofs stay cache-off.
The core, driver, daemon/store path and executable fixtures are implemented;
representative bootstrap and protected-exec acceptance remain open in tasks.
Source delivery and proposal gates are not runtime or producer acceptance.

## Current behavior

`crunch-rust-cache` stores Rust unit results and bounded C object/manifest
records in the existing local store. `crunch-rustc-wrapper::cc` admits only
classified compile-mode operations with complete bounded identities, forwards
unknown operations unchanged, and uses the existing daemon for optional reuse.
`crunch-build::compile_driver` maps only an explicitly selected runtime socket
after derivation conversion; it neither installs a compiler nor admits a
protected executable inventory. StageX/GCC-chain integration, byte parity,
timing and uncached fixed-point proof still require captured execution.

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

## Environment identity and degradation admission

Compiler search environment variables use tagged alias-relative path entries,
not substring rewriting of arbitrary strings. Recognized colon-separated
search variables admit only absolute paths under declared path-independent
roots. Empty or relative search entries, undeclared component-prefix siblings,
parent traversal and path-bearing values with unclassified semantics bypass
cache reuse. This keeps ambient cwd and unmeasured negative searches out of
cache admission, while relocated identical declared roots remain eligible.

Cache connections reuse the slot client's nonblocking Unix connect boundary.
One five-second deadline covers connect and framed request/response I/O, so
an absent daemon or saturated accept backlog cannot block plain compilation
indefinitely. Read/publication/receipt failures remain advisory. The source
fixtures in `scripts/prove-cc-cache-slots.py` must be executed separately;
their presence does not establish bootstrap parity or strict proof acceptance.

