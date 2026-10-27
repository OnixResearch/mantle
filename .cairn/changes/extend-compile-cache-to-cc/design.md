# Design: Extend the compile cache to C and C++ builders

## Goal and scope

The existing Rust unit cache seam has an opt-in C/C++ compiler-driver mode.
The cache stays a machine-local performance device; proofs stay cache-off.
The core, driver, daemon/store path and executable fixtures are implemented;
representative bootstrap and production protected-build integration remain open.
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

## Proof-runner cache exclusion

`scripts/compile-cache-proof-boundary.sh` owns the common admission check for
the self-hosting and real-release shell entrypoints. It rejects the complete
`MANTLE_CC_CACHE_*` and `MANTLE_COMPILE_*` families before prerequisite lookup,
scratch creation, or release-output changes, including empty values, future
keys, and raw names that Bash cannot import. NUL framing preserves name
boundaries; diagnostics quote names and never include binding values.

Successful admission records the selected excluded mode in runner stderr.
Direct Cargo-free fixed-point admission also rejects inherited cache/scheduling
bindings through the existing cold-proof guard, before output preparation.
The native before/after observations and retained failed harness attempts are
in `evidence/proof-exclusion-2026-10-27.json`.

Self-hosting manifests/summaries and Cargo-free preflight/final reports record
`compiler_cache_mode: excluded`. Release and cold source-built deterministic
receipts add `compiler-cache=excluded` to the existing execution envelope;
cold source-built attempts also write `transcripts/compiler-cache-mode.txt`.
Development attempts do not acquire that cold transcript. The mode is a
diagnostic observation, never rebuild or admission authority.

The existing deterministic and provider-stage receipt contracts already
reject cache-only records. Public CLI regressions retain valid typed-schema
fixtures, then replace only the rebuild citation with a compiler-cache receipt
at external, bundled, and provider-stage intake boundaries. This is source and
bounded diagnostic integration, not an executed cold source-built cohort or
the representative uncached/cached proof acceptance required by T3.2/T4.2.

## Declared-driver inventory fixture

`examples/protected_cc_driver.rs` reuses `ProtectedExecPolicy` and the existing
seccomp supervisor; it does not implement a second authorization mechanism.
The runner claims a fresh audit file, installs the irreversible filter, waits
for the real command and audit quiescence, and records the observed decisions.
Its observation is explicitly fixture-only and cannot admit a proof.

Select `--protected-exec-runner` and `--sandbox-shell` together in
`scripts/prove-cc-cache-slots.py`. The script generates an inventory from
explicitly selected binaries, their measured BLAKE3 identities and bounded
actual version-command output. The compiler's own helper queries supply the
helper paths. The sandbox entry and shell remain explicit required seed roles;
all entries have `test-fixture` provenance, not source-built provenance.

Set `NATIVE_CC` and `NATIVE_CXX` to the selected GCC-compatible native compiler
executables, and `ARTIFACTS` to an absent/empty short absolute directory. In the
repository development shell:

```sh
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target}"
cargo build --locked -p mantle --example protected_cc_driver
cargo build --locked -p crunch-rustc-wrapper --bin mantle-cc-driver --bin mantle-rust-cache-daemon
python3 scripts/prove-cc-cache-slots.py \
  --driver "$CARGO_TARGET_DIR/debug/mantle-cc-driver" \
  --daemon "$CARGO_TARGET_DIR/debug/mantle-rust-cache-daemon" \
  --compiler "$NATIVE_CC" --cxx "$NATIVE_CXX" \
  --work-dir "$ARTIFACTS" \
  --protected-exec-runner "$CARGO_TARGET_DIR/debug/examples/protected_cc_driver" \
  --sandbox-shell "$SNIX_BUILD_SANDBOX_SHELL"
```

A successful selected fixture must observe real bwrap, driver, C/C++ compiler
and assembler executions matching the declared paths, digests and inventory
IDs. Preprocessing with a space-bearing macro argument must preserve the
direct compiler's streams and status. A cache hit must retain identical object
bytes without a compiler exec. Removing the driver/compiler entry or changing
its expected digest must deny that exact execution, with no fallback decision.

Compile-cache and protected-exec maintainers own this executable boundary rail.
The fixture does not install a driver into a production bootstrap recipe,
prove derivation-identity parity, establish representative-chain timings, or
replace the cache-off fixed-point obligation. T3.1/T3.2/T4.2 retain those
separate acceptance requirements.

The inventoried static BusyBox seed's `env -i` constructs the exact declared
compiler environment after entering the sandbox. Bubblewrap adds `PWD` even
after `--clearenv`/`--unsetenv PWD`; silently ignoring that binding would weaken
driver admission. The fixture instead leaves environment admission unchanged
and observes the seed execution in the protected audit.


