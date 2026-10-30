# NASA SpaceWasm reference cohort

This directory defines Mantle's bounded, diagnostic-only materialization of
[`nasa/spacewasm`](https://github.com/nasa/spacewasm) at exact commit
`e24cf09355a90497148eb5029fdb8e3400bd63e3`. Octet's reviewed
`spacewasm-mvp` support projection selects this commit; it replaces the earlier
proposal review revision `30cd6e9b91f84a39278edcb5d66514b773011ccc`.
The replacement is not assumed equivalent: the Nix lane rebuilds and replays
the complete declared fixture set and emits `reports/replay-evidence.json`.

`profile.ncl` is the typed source of truth. `generated/profile.json` is its
checked deterministic export. The profile binds the source archive, Cargo lock,
dependency manifest, Rust toolchain, host/wasm targets, MVP support matrix,
runner bounds, fixtures, corpus descriptors, outcome states, retention policy,
and non-claims.

Build and independently rehash the immutable bundle with:

```sh
nix build .#spacewasm-reference-bundle
nix run .#spacewasm-reference-bundler -- verify ./result
```

The build uses the pinned Nix source fetch and Cargo lock import, then sets Cargo
to offline mode for all source builds. It retains the exact source archive,
locked vendor closure, minimal Rust `1.91.1` host/`wasm32-unknown-unknown`
toolchain, host and wasm libraries, bounded host runner, generated fixtures,
canonical corpus archives, descriptors, licenses/notices, result report,
revision replay evidence, materialization report, parent edges, and BLAKE3
member identities.

The bounded result states are exact. Host/wasm builds, host runner build,
upstream unit tests, the upstream `address` spectest lane, and all eight fixture
classes are expected to pass. The full spectest suite is explicitly skipped in
this bounded lane; the absent `spacewasm-check` workflow and continuous fuzzing
run are recorded unavailable; post-MVP feature build admission is recorded
unsupported. Mantle does not synthesize any missing upstream tool result.

This materialization is not a claim of SpaceWasm correctness, memory safety,
WebAssembly conformance, flight qualification, sandbox effectiveness, consumer
runtime admission, production readiness, or release eligibility.

## Additive Molten host segmentation fixture

The bundle also retains `fixtures/wat/molten-host-segments.wat`, its generated
`fixtures/wasm/molten-host-segments.wasm`, and the Nickel-authored descriptor
`fixtures/descriptors/molten-host-segments.json`. The existing pinned
`wasm-tools` generator validates wasm1 (MVP plus mutable-global exports), then
prints and reparses the binary before publication. The manifest binds every member
by BLAKE3 and size; no artifact hash or generated binary is hand-authored.

This fixture imports `molten.observe(i32) -> ()` and `molten.pause() -> ()`,
exports `run() -> ()` and mutable `i32` global `total`, and declares the transcript
`observe(7), pause(), observe(11)` with final `total = 18`. It has no memory,
table, start function, or growth. Octet's `spacewasm-mvp-host-pilot` profile and
ChaosControl's consumer probe own separate admission and execution evidence.
Fuel must remain cumulative across SpaceWasm resume. Wasmtime `41.0.3` uses an
ordinary host return for pause; transcript comparison does not establish equal
suspension semantics.

The existing eight hostless checks and their profile are unchanged. In
particular, the reference runner does **not** execute the additional fixture.
A pilot consumer must require these three paths explicitly, verify the entire
bundle, and bind the selected fixture member identity before execution. An old
valid hostless bundle is not sufficient. Missing members, digest mismatch, a
hostless substitute, or a locally compiled replacement must fail consumer
admission; a locally matching byte sequence does not replace producer identity.

The immutable public package address is
`github:OnixResearch/mantle/<published-revision>#spacewasm-reference-bundle`.
The release owner supplies the published revision after integration; this
checkout does not invent a publication identity. Use the build and independent
verification commands above with that exact revision, never a sibling path or
mutable branch. Source acquisition was independently rechecked on 2026-10-29:
NASA commit `e24cf09355a90497148eb5029fdb8e3400bd63e3` exists and Nix prefetch
and BLAKE3 agree with the admitted archive identities in `profile.ncl`.

`nix/spacewasm-molten-abi.nix` additionally authors the full
`molten.wasm.abi.v1` family. Each name below produces WAT, generated Wasm,
and a descriptor under `fixtures/{wat,wasm,descriptors}`:

- `molten-abi-roundtrip`: canonical input is returned unchanged.
- `molten-abi-post-dealloc-state`: output aliases canonical input at 1024, while
  each deallocation increments a private mutable `i32` global and writes its
  low byte to memory offset zero. Input and output alias, so deallocation occurs
  once: private count and memory byte must equal one, with no double-free and
  canonical output unchanged.
- `molten-abi-hostcall`: declared `molten:hostcall` / `assert` receives the input
  and the actor returns the same canonical value.
- `molten-abi-invalid-pointer`: packed pointer `0xffffffff`, length one.
- `molten-abi-invalid-output`: a single invalid `ff` byte is returned.
- `molten-abi-memory-limit`: two fixed memory pages, rejected by a one-page budget.
- `molten-abi-stack-limit`: non-tail recursion, bounded by stack or cumulative fuel.
- `molten-abi-trap`: `unreachable` traps.
- `molten-abi-out-of-fuel`: an infinite loop requires cumulative fuel exhaustion.

The ABI fixtures export bounded `memory`, `molten_alloc(i32)->i32`,
`molten_dealloc(i32,i32)->()`, and `molten_run(i32,i32)->i64`, packing the
pointer into the high 32 bits and length into the low 32 bits. Their allocation
area begins at 1024. Inputs must remain inside the consumer's admitted bounds.
These are authoring fixtures, not a general-purpose guest allocator.

The existing `spacewasm-reference-negative` Nix check now also verifies the
complete bundle and rejects missing, mismatched and hostless-substituted pilot
fixtures. The consumer must separately reject a local build lacking the pinned
producer/publication identity; a byte verifier cannot distinguish identical
locally compiled bytes by itself. No new execution receipt is claimed here.


