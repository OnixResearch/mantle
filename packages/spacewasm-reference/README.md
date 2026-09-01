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

The build uses the pinned Nix source fetch and Crane vendoring from the pinned
Cargo lock, then sets Cargo to offline mode for all source builds. It retains
the exact source archive,
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
