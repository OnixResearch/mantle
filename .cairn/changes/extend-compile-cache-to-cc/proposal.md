# Proposal: Extend the compile cache to C and C++ builders

## Why

Mantle already caches Rust unit results (`crunch-rust-cache`,
`crunch-rustc-wrapper`, `mantle-rust-cache-daemon`) with receipts and
dispositions, but its dominant iteration cost is the C/C++ bootstrap chain:
multi-hour StageX and GCC proofs that recompile every object when a recipe
changes. The reviewed external reference (`mic92/repkgs`, commit `1cd7b8b`,
see `evidence/repkgs-review.md`) shows the shape that works for C: a compiler
driver that asks a host daemon before compiling, keyed by content identity,
with the cache deliberately not a derivation input so derivation hashes and
outputs are identical with or without it.

Mantle has two structural advantages over that reference: castore identities
are content-addressed by BLAKE3, so path-to-identity masking is native; and
the protected-exec seccomp supervisor is an existing interception point that
keeps a driver inside the declared tool inventory.

## What Changes

- Define a Mantle-owned compiler-driver seam for bootstrap C/C++ toolchain
  invocations, admitted through explicit policy and compatible with the
  protected-exec supervisor. r[mantle.cc_compile_cache.driver_boundary]
- Key cache entries by content identity only: source bytes, normalized
  arguments, tool identity, and learned dependency manifests from depfiles;
  store paths never enter a key. r[mantle.cc_compile_cache.content_keyed_identity]
- Keep the cache outside the derivation graph: identical outputs with and
  without the cache, no derivation input or hash change, fail-open compile on
  cache unavailability. r[mantle.cc_compile_cache.non_input_cache_boundary]
- Extend the strict evidence lanes: fixed-point and release proofs must run
  cache-off or verify content identity independently.
  r[mantle.cc_compile_cache.proof_exclusion]
- Add bounded optional caching of configure-probe results and compile
  failures, with negative controls against wrong-key hits.
  r[mantle.cc_compile_cache.probe_and_failure_cache]

## Impact

- **Immediate consumer**: the source-built fixed-point proof
  (`prove-source-built-mantle-fixed-point`) and every StageX/GCC chain
  iteration session.
- **Immediate outcome**: recipe edits stop recompiling unchanged objects
  across the C/C++ chain; the external reference measured a stage-1 toolchain
  rebuild at roughly half the time with the remainder being non-compiler work.
- **Durable capability**: a machine-local, content-addressed compile cache
  shared across derivations with receipts, reusing the existing
  `crunch-rust-cache` storage and daemon framing.
- **Maintenance owner**: Mantle cache owner, covering `crunch-rust-cache`,
  `crunch-rustc-wrapper`, and the new driver seam.
- **Repeatability evidence**: byte-identical outputs with cache on and off,
  receipt dispositions per compile, negative fixtures for stale and
  wrong-identity hits, and cache-off fixed-point runs.
- **Compatibility**: builds without the daemon must behave exactly as today.

## Scope

The change covers the driver seam for the bootstrap toolchain family
(`x86_64-linux-musl-*`, stage TinyCC/GCC wrappers), the content-key scheme,
depfile manifest learning, probe/failure caching, receipt surfacing, and
proof-lane exclusion.

## Out of Scope

- Rust caching (already owned by the existing seam).
- Any remote or shared multi-user cache; the first slice is per-machine and
  per-user.
- Trust claims: the cache is a performance device, never evidence of
  correctness.
- Cross-compilation driver policy beyond identity masking.

## Success Criteria

- A recipe edit that changes no compilation input produces zero recompiles
  for the affected derivations across a rebuild, with receipts showing reuse.
- Outputs are byte-identical with the cache on and off, proven by the
  fixed-point proof running cache-off.
- A changed source, argument, tool, or learned dependency always misses.
- Killing the daemon mid-session degrades to plain compilation without
  failing builds.
