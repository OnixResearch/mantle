# Build contract hash compatibility

## Scope

The standalone `mantle-build-contract` package now permits Cargo's compatible
BLAKE3 `1.8.2` range. It keeps default features disabled and `pure` enabled.
Its request framing, observation framing, admission functions, schemas, and
original producer fixtures remain unchanged.

The owner workspace still selects BLAKE3 1.8.2. The store's exact dependency
and digest-trait boundary remain unchanged. Independent fixture workspaces
select exactly 1.8.2 and 1.8.7. They use only same-repository path dependencies.
Those paths are test fixtures, not downstream product dependencies.

Base source: published main `2efea1b98bae1070ce7f59340c3930593927b81c`.
The initial modern consumer failed to resolve before the manifest fix:

```text
error: failed to select a version for `blake3`.
versions that meet the requirements `=1.8.2` are: 1.8.2
previously selected package `blake3 v1.8.7`
```

Published contract source: `5884354802f4d87dfb25dfe0b3f978e1bf7f134b`.
Neural Stream fetched that exact source through authenticated Nix and Cargo Git.
Its source NAR is `sha256-olB+d6ShR1Q6dSj+BwimmDxjdHABZJpyslK1BcbkuQQ=`.
The actual linked consumer proof passed, as recorded in the final section.
No native executor is admitted here.

## Focused owner and independent consumer results

Commands ran in the pinned `nix develop` environment with a separate Cargo target.
Cargo generated both fixture lockfiles. No lockfile was edited by hand.

```sh
cargo test -p mantle-build-contract --all-targets --locked
cargo test --manifest-path fixtures/mantle-build-contract/hash-compat/minimum/Cargo.toml --all-targets --locked
cargo test --manifest-path fixtures/mantle-build-contract/hash-compat/current/Cargo.toml --all-targets --locked
cargo check --manifest-path fixtures/mantle-build-contract/hash-compat/minimum/Cargo.toml --lib --target wasm32-unknown-unknown --locked
cargo check --manifest-path fixtures/mantle-build-contract/hash-compat/current/Cargo.toml --lib --target wasm32-unknown-unknown --locked
cargo clippy -p mantle-build-contract --all-targets --all-features --locked -- -D warnings
cargo clippy --manifest-path fixtures/mantle-build-contract/hash-compat/minimum/Cargo.toml --all-targets --all-features --locked -- -D warnings
cargo clippy --manifest-path fixtures/mantle-build-contract/hash-compat/current/Cargo.toml --all-targets --all-features --locked -- -D warnings
```

Observed owner results:

```text
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Each independent consumer returned:

```text
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Both wasm checks and all three scoped Clippy checks passed. The tests retain
original request, direct-success, cached-success, and failure identities.
Negative controls reject stale identities, changed builders, missing products,
absent cache sources, contradictory unknown receipts, and missing non-claims.
Two BLAKE3 known-answer vectors also match their fixed values.

`cargo run -p mantle-build-contract --example emit-fixtures --locked` reproduced
the original producer file byte-for-byte. An exact Git diff confirmed unchanged
owner lock, store manifest, contract Rust source, and original producer fixtures.

## Sandboxed Nix checks

The matrix and Nickel checks passed with local builders, two cores, one job,
and a 15-minute outer timeout with a 30-second termination grace:

```sh
nix build .#checks.x86_64-linux.mantle-build-contract-hash-minimum \
  .#checks.x86_64-linux.mantle-build-contract-hash-current \
  .#checks.x86_64-linux.mantle-build-contract-nickel \
  --no-link --print-out-paths --builders '' --cores 2 -j 1 -L
```

Each matrix check ran seven release tests and its wasm compilation. The
Nickel check retained both positive and expected-negative fixtures.

Observed outputs:

```text
/nix/store/s71x3haxkafyizm5dssvgy0gcf4ji0b9-mantle-build-contract-hash-minimum-test-1
/nix/store/m8a58yx82sn1vv76fwbllcxc2m2pj1i8-mantle-build-contract-hash-current-test-1
/nix/store/zkgh2zbclrn4spkcz9qkqnr80nda3ga3-mantle-build-contract-nickel
```

## Inherited lint prerequisite

The first full pinned Tiger Style check rejected three pre-existing findings
in `crunch-repair-core/src/legacy_archive.rs`: assertion density in two
functions and one compound root-bound condition.

A separate maintenance commit splits that existing guard and states existing
postconditions with debug assertions. It does not change migration decisions,
rejection order, trust requirements, or content checks. No lint is suppressed.

The repair-core baseline passed before the cleanup. After the cleanup, the
focused checks returned:

```text
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The four additional closure controls cover accepted reachability, exact and
excessive bounds, duplicate roots, and distinct malformed-member rejections.
The strict repair-core Clippy check passed.

The unchanged full Tiger Style gate then passed under the pinned Nix check:

```sh
nix build .#checks.x86_64-linux.tigerstyle --no-link --print-out-paths --builders '' --cores 2 -j 1 -L
```

Output: `/nix/store/cw4zf8lzzxq405jdwaihgl5cg1nnji9j-tigerstyle-consumer-check`.
The check used a 20-minute outer timeout and a 30-second termination grace.
It completed normally, not through timeout.

## Exact committed-source and linked consumer evidence

The matrix, Nickel, and full Tiger Style Nix checks ran again at exact local
Git source `5884354802f4d87dfb25dfe0b3f978e1bf7f134b`, already published on the
change branch. All four checks passed. The two matrix lanes each returned:

```text
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Committed-source outputs:

```text
/nix/store/jswpkgysihm61bwg84zs1qr4x3wb49xw-mantle-build-contract-hash-minimum-test-1
/nix/store/i8mb187bjcv95n729wavlp3mrvdhrvs0-mantle-build-contract-hash-current-test-1
/nix/store/g263gsiwarr2pcnbvvpy1rxaw9f1cz9w-mantle-build-contract-nickel
```

The full Tiger Style output is `/nix/store/zchccg440ydhvdl8d014ijc5654l5xyz-tigerstyle-consumer-check`.
The maintained owner lifecycle validation also passed.

Neural Stream source `4d0396cdefbdd7a9c1bae6ebb6f7154983942fe7` pins the
published contract in the real Memory IR shell graph. Cargo added only the
contract package and dependency edge. Animus `4f08f89d...`, BLAKE3 1.8.7,
Serde 1.0.228, and serde_json 1.0.151 remain selected. No override or sibling
product dependency was used. A source-only Nix input binds the same revision.

The linked consumer returned:

```text
test result: ok. 54 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

Four new linked controls preserve producer identities and rejection behavior.
Eleven registry tests, ten negative self-controls, strict shell Clippy, and all
four zero-finding Octet profiles passed. All eleven existing campaigns replayed
after the source commit. The consumer's full `devenv test` passed in 40.7 seconds,
with FCIS 118 Rust core files, three Python core files, and four shell bindings.
The consumer retains separate native CLI and inspection-library source roles.

This establishes linked contract compatibility, not the missing native
request-to-observation adapter. Neural materialization remains 5/11. No training,
content reader, runtime decoder, native CLI update, or promotion was activated.

## Evidence retention and non-claims

Raw baseline, pre-fix failure, focused, matrix, and Tiger Style logs remain in
the operator's persistent Neural Stream `.pi/drains/2026-09-06-mantle-hash-compat/`.
The owning change records final source, linked consumer, and lifecycle receipts
before task completion. Independent test graphs are not independent human reviews.

This evidence does not certify all future BLAKE3 versions, native CLI package
execution at this source, NAR or store authenticity, training, reproducibility,
model quality, production authority, or promotion. The earlier native package
acceptance still names its original exact source and output. No trusted-store
signature policy, default executor, or authority dependency changed.
