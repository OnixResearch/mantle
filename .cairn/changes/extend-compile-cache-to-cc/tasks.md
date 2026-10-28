# Tasks: Extend the compile cache to C and C++ builders

Source implementation and selected native cache/protected-inventory fixtures
are present. Checked tasks identify delivered source and bounded fixtures, not
a representative bootstrap rebuild, production protected-build integration,
performance result, or final gate. `evidence/gates-2026-09-10.md` remains
proposal-stage evidence only.

The scoped native runtime retry passed with the retained immutable driver and
daemon: 20 cache cases, real C/C++ compilation, FIFO-gated contention,
saturated-backlog degradation, and authority death during compilation.
`evidence/runtime-final-immutable-retry-2026-10-23.json` retains the commands,
identities, outcomes, failed predecessor, and unchanged acceptance requirements.
This does not close protected-bootstrap integration or T3.2/T4.2's chain and
cache-off fixed-point obligations. Repeat with `scripts/prove-cc-cache-slots.py`
and fresh private roots; compile-cache maintainers own this rail.

The 2026-10-27 proof-runner continuation closes an inherited-environment gap in
`prove-self-hosting.sh` and `prove-real-release-determinism.sh`. Native checks
reproduce cache bindings passing the original self-hosting preflight, then
observe 32 refusals before attempt creation, clean/near-prefix preflight
success, and cache-exclusion reporting after the fix. The real-release clean
control still requires a content-bound toolchain archive; no release or
self-hosting proof runs. Exact captures and earlier harness blockers are in
`evidence/proof-exclusion-2026-10-27.json`. The strict receipt/transcript source
continuation completes T3.3: direct Cargo-free admission rejects inherited
authority before creating an attempt, proof reports persist the excluded mode,
and public receipt consumers reject cache-only rebuild citations. Focused
regressions and a native first-stage compile with an explicitly refused second
build are bounded observations, not an admitted proof. Representative-chain
identity/timing, the cache-off fixed point, and an owner-admitted cold
source-built cohort remain unaccepted. Compile-cache and proof-admission
maintainers own these report/guard surfaces and their regression rails.

The declared-driver continuation completes T2.2. The final frozen-subject run
observes real protected C/C++ compilation, space-bearing argument forwarding
matching the direct compiler's streams/status, and an identical-object cache
hit without a compiler execution. Four controls deny the undeclared driver or
compiler and their wrong-digest variants. Exact 1 MiB inventory input is
accepted; oversized and malformed inputs fail before audit creation.
`evidence/protected-driver-2026-10-27.json` retains the selected checks, raw
audits/streams, source and executable identities, and failed predecessors.
The inventoried BusyBox seed constructs the exact child environment with
`env -i`; driver admission still rejects undeclared bindings such as bwrap's
implicit `PWD`. Compile-cache and protected-exec maintainers own this rail.

The T3.1 continuation passes the production endpoint/inventory seam. Pueue 124
observes seven real `mantle build` cases over the frozen binaries: plain and
protected cold builds, an object hit that executes no compiler or helper, an
absent-daemon fallback, and undeclared-driver, wrong-compiler-digest, and
output-overlapping-audit refusals. Cold, warm, absent-daemon, and plain runs
retain one derivation key, logical output path, object BLAKE3, object mode, and
recipe identity. Every protected case records `proof_evidence: false`.

The predecessor run exposed a supervisor defect: it opened the guest
executable through `/proc/<pid>/root`, discarded the handle, then hashed the
reported pathname in the supervisor namespace. A guest-only driver therefore
failed with ENOENT. The supervisor now hashes the retained file for policy and
diagnostic reads; the regression pairs guest-byte admission with refusal of
different host bytes at the same pathname. Binaries share the library
supervisor instead of compiling a second private copy. Exact captures, frozen
artifact identities, and failed predecessors are in
`evidence/protected-build-shell-2026-10-27.json`.

This closes T3.1 only. T3.2/T4.2's representative chain and cache-off fixed
point, T4.3's full Nix-check execution, and T4.4's specs/archive remain open.

## Phase 1: Baseline and contract

- [ ] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record the current cache seam (`crunch-rust-cache`, `crunch-rustc-wrapper`, daemon, strict lanes), a timed uncached baseline of one representative bootstrap chain rebuild, and focused test output. r[mantle.cc_compile_cache.proof_exclusion]
- [x] [serial] T1.2 Define the versioned C/C++ cache contract: driver boundary policy, key normalization, manifest learning, dispositions, and non-claims as a typed Nickel export. r[mantle.cc_compile_cache.driver_boundary] r[mantle.cc_compile_cache.content_keyed_identity]
- [x] [serial] T1.3 Record the reuse-existing-daemon and depfile-manifest decisions in an ADR, including the trust framing and proof-lane exclusion. r[mantle.cc_compile_cache.proof_exclusion]

## Phase 2: Core and driver

- [x] [serial] T2.1 Implement pure key normalization, manifest comparison, and typed admission decisions in a new core module beside `crunch-rust-cache-core`. r[mantle.cc_compile_cache.content_keyed_identity]
- [x] [serial] T2.2 Implement the driver seam with forward-unchanged behavior for unclassifiable invocations and protected-inventory compatibility. r[mantle.cc_compile_cache.driver_boundary]
- [x] [serial] T2.3 Extend the daemon with compile-object serving and depfile manifest storage. r[mantle.cc_compile_cache.content_keyed_identity]
- [x] [parallel] T2.4 Add positive fixtures: identical-content hit, rebuilt-identical dependency hit, forward-unchanged receipt. r[mantle.cc_compile_cache.content_keyed_identity] r[mantle.cc_compile_cache.driver_boundary]
- [x] [parallel] T2.5 Add negative fixtures: changed source, argument, tool, or manifest input misses; unknown-manifest failure replay refusal; daemon-unavailable degradation. r[mantle.cc_compile_cache.content_keyed_identity] r[mantle.cc_compile_cache.non_input_cache_boundary]

## Phase 3: Build integration and proof lanes

- [x] [serial] T3.1 Wire execution-time endpoint mapping into the build shell for protected bootstrap builds without derivation input changes. r[mantle.cc_compile_cache.non_input_cache_boundary]
- [ ] [serial] T3.2 Prove byte-identical outputs with cache on and off on the representative chain rebuild, with disposition records. r[mantle.cc_compile_cache.non_input_cache_boundary]
- [x] [serial] T3.3 Extend strict evidence lanes to reject cache receipts as rebuild evidence and record cache mode in proof transcripts. r[mantle.cc_compile_cache.proof_exclusion]
- [x] [serial] T3.4 Implement bounded probe-result and failure caching with per-read dispositions. r[mantle.cc_compile_cache.probe_and_failure_cache]

## Phase 4: Verification

- [x] [parallel] T4.1 Add probe-cache negative controls: changed probe script, toolchain identity, dependency identity, platform, or flags must not reuse a cached result. r[mantle.cc_compile_cache.probe_and_failure_cache]
- [ ] [serial] T4.2 Run one cache-off fixed-point proof pass and one cached iteration pass on the representative chain; record both transcripts and timing deltas without claiming the cache as evidence. r[mantle.cc_compile_cache.proof_exclusion]
- [ ] [serial] T4.3 Run focused core and shell tests before and after changes, strict Clippy, policy freshness, and relevant Nix checks. Preserve exact blockers. r[mantle.cc_compile_cache.driver_boundary]
- [ ] [serial] T4.4 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.cc_compile_cache.proof_exclusion]

## Source checkpoint (2026-10-23; execution not accepted)

- Owners: `config/compile-cache/default.ncl`, ADRs 0099/0116,
  `crunch-rust-cache-core::cc`, `crunch-rust-cache::cc`, and
  `crunch-rustc-wrapper::cc`. The existing daemon dispatches object and manifest
  requests; records and retained object roots share the existing local store.
- Keys bind complete bounded root contents, source/compiler/environment/platform
  identities, ordered options and revalidated learned dependencies. Unknown
  manifests, volatile inputs, incomplete `-MMD`, links and preprocess actions
  are not promoted to reusable results. Optional probes are compile-mode only.
- Environment search paths now use component-bounded tagged root identities,
  not substring replacement. Only declared absolute entries in recognized
  compiler search variables are normalized. Relative/empty search entries,
  undeclared sibling roots, path-dependent roots and unclassified embedded
  paths forward plainly. Literal aliases cannot masquerade as declared paths.
- Cache connection setup is nonblocking, including a saturated Unix backlog;
  request/response I/O shares one five-second deadline rather than resetting
  the budget for every partial frame read. Cache and receipt failure preserve
  the actual compiler outcome. Old environment-key encodings simply miss.
- `crunch-build::compile_driver` already maps an explicitly selected local
  socket after derivation conversion; it does not install a compiler inventory.
  Protected bootstrap inventory/audit parity, representative-chain identity,
  timed rebuilds, strict proof transcripts and native acceptance gates remain
  outstanding. No accepted spec was changed or archived.
- Behavioral fixtures live in the core/driver modules and
  `scripts/prove-cc-cache-slots.py`. The latter uses real C/C++ compilers,
  depfiles, native objects, FIFO-gated contention, namespace credentials,
  saturated-backlog degradation and authority death during a live compile.
  Neither these fixtures nor project checks were run in this source-only pass.

Parent verification in the pinned development environment, with the parent's
scratch `CARGO_TARGET_DIR` retained (commands are not captured results):

```sh
cargo test -p crunch-rust-cache-core --lib cc::
cargo test -p crunch-rustc-wrapper --lib cc::
cargo test -p crunch-build --lib compile_driver::
cargo build -p crunch-rustc-wrapper --bin mantle-cc-driver --bin mantle-rust-cache-daemon
python3 scripts/prove-cc-cache-slots.py \
  --driver "${CARGO_TARGET_DIR:-$PWD/target}/debug/mantle-cc-driver" \
  --daemon "${CARGO_TARGET_DIR:-$PWD/target}/debug/mantle-rust-cache-daemon" \
  --compiler /nix/store/4yxdxw3kppr2pbic95p4mri520rkdag2-gcc-15.3.0/bin/gcc \
  --cxx /nix/store/4yxdxw3kppr2pbic95p4mri520rkdag2-gcc-15.3.0/bin/g++ \
  --work-dir /tmp/mantle-cc-20261023
```

Use an absent/empty, short work directory; retain its `evidence.json` and all
streams. The native compiler paths above were read from the installed wrappers;
the wrappers themselves are not substitute measured compiler/runtime closures.
The parent owns the captured pinned development-shell/toolchain and Linux
`bwrap` availability; the ordinary interactive PATH alone lacks both `cargo`
and `bwrap`. No substitute toolchain or provider is authorized here.
The smoke is a bounded local observation, not the representative bootstrap or
cache-off fixed-point acceptance still required by T3.2/T4.2.
