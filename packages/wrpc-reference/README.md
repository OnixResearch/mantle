# wRPC component producer

This producer retains real synchronous component RPC imports and exports from
[`bytecodealliance/wrpc`](https://github.com/bytecodealliance/wrpc/tree/aa01409ed38bf7a978570ff567c4475376925df6),
release 0.15.0, commit `aa01409ed38bf7a978570ff567c4475376925df6`.
It is not a TCP fixture. The upstream unversioned `wrpc-examples:hello/handler`
WIT is retained byte-for-byte and compared against the pinned source archive.
Mantle's small wrappers expose a deterministic callable `run() -> u32` rather
than a WASI CLI command that prints output. The server additionally exports the
real `hello() -> string` interface. The client imports that interface, checks
that its result is exactly `hello from Rust`, and returns 42. Both wrappers are
source under `guest/`; the full transitive WIT sources remain in the bundle.

## Producer command

From this worktree, including newly authored files:

```sh
nix build "path:$PWD#packages.x86_64-linux.wrpc-reference-bundle" \
  --no-link --print-out-paths -L
```

The public package output is `packages.x86_64-linux.wrpc-reference-bundle` in
`OnixResearch/mantle`. The release owner must publish these source changes and
supply the resulting exact 40-hex revision. The immutable package address is
constructed only from that actual published revision:

```sh
: "${MANTLE_PUBLISHED_REVISION:?release owner supplies published Mantle commit}"
test "${#MANTLE_PUBLISHED_REVISION}" -eq 40
nix build "github:OnixResearch/mantle/${MANTLE_PUBLISHED_REVISION}#packages.x86_64-linux.wrpc-reference-bundle" \
  --no-link --print-out-paths -L
```

Do not replace this address with a sibling checkout, mutable branch, ordinary
transport fixture, locally rebuilt guest, or a different materialization bundle.
No publication revision, generated component digest, store output path, or
successful execution receipt is asserted by this source-only implementation.
Molten imports/authorizes the published package using its existing immutable
source binding procedure; it must never invoke Cargo, Nix guest compilation, or
Mantle guest builds to repair a missing artifact.

The Nix producer invokes the actual exporter, once for each component:

```sh
mantle --json --store "$STORE" --state-dir "$STATE" \
  wasm-component build "$RESOLVED_REQUEST" --out "$COMPONENT_OUTPUT" \
  --scratch-parent "$OUTPUT_PARENT"
```

`mantle-wrpc-author` resolves the Nickel request from measured source trees,
real compiled baseline components, the verified twelve-tool cohort, and the
consumer runtime profile reference. It never synthesizes an execution report.
The existing exporter executes offline WKG resolution, binding generation,
locked offline Rust compilation, WAC composition, WASI virtualization/import
inspection, wasm-tools validation/normalization, Wasmtime smoke execution,
Octet admission and real materialization/attestation publication.

WKG resolves `wrpc-examples:hello` only through the checked-in `guest/wkg.toml`
path override to `guest/wit-deps/hello`, which the producer compares byte-for-byte
with the pinned upstream archive. The local registry stays empty, so resolution
cannot fetch or substitute a different interface. The telemetry package applies
the same rule to its `wasi:otel`, `wasi:clocks` and `wasi:io` WIT.

## Open RPC versus closed comparison

The client's **final portable artifact retains** exactly the open
`wrpc-examples:hello/handler` import. Its manifest disallows every WASI subsystem;
no clocks, randomness, network, filesystem, environment, poll, or stdio authority
is granted. The server has no remaining imports. Deny-all virtualization never
means denying the explicitly declared non-WASI RPC import.

The optional `runtime_composition` request binds the exact final client once to
the exact measured server with WAC, then runs that closed component with
`wasmtime run --invoke 'run()'`. Its source, dependency identities, command
receipt and generated Wasm are retained. This is **comparison-only evidence**:
`client/artifacts/runtime-composed.wasm` is not the remoted client and must not
be selected for the Molten RPC pilot. Production remote execution must use the
open client with upstream `wrpc-runtime-wasmtime::link_item` and execute the
server through upstream `wrpc-runtime-wasmtime::call`, implementing
`wrpc_transport::Invoke`/`Serve` over admitted Molten fabric frames. Transport
acknowledgements are not RPC results. Those remote execution observations belong
to Molten, not this producer's closed smoke.

## Stable handoff

`manifest.json`, schema `mantle-wrpc-reference-bundle-v1`, contains:

- `client` and `server`: `bundle`, `bundle_identity_blake3`, `final_portable`,
  `runtime_profile`, `runtime_profile_ref`, and `world`.
- `capability_root = "capability"`: open this directory beneath the authorized
  immutable package capability. It mirrors the exporter logical path prefix
  without granting access to the ambient filesystem root. The original
  receipts/attestations and logical member identities are not rewritten.
- `members`: every retained file (including the capability mirror), sorted by
  path, with `path`, `role`, `digest_blake3`, `size_bytes`; at most 40,000 members
  and 512 MiB per member. The manifest itself is excluded from that inventory.
- `manifest_identity_blake3`: BLAKE3 of compact sorted-key JSON before adding
  this field, with no trailing newline.
- `remote_interface`, `remote_function`, `client_entrypoint`, expected results,
  `comparison_only`, source cohort, and explicit non-claims.

Select `client/materialization-bundle.json` and
`server/materialization-bundle.json`, using the existing
`mantle-wasm-component-materialization-bundle-v1` DTO and published
`mantle-wasm-consumer-verifier` API. Give it the opened `capability/` root, not
`/`. Each `final_portable` is independently byte-verified before instantiation.
The private consumer snapshot must copy/revalidate the source-closure directory
under capability-relative traversal and hold the directory capability throughout.

`client/runtime-profile.json` and `server/runtime-profile.json` have Molten's
`RuntimeProfile` shape. `profile.py` uses the consumer's exact line/Debug-record
reference protocol, **not JSON hashing**, and binds the selected world WIT
content reference. `runtime_profile_ref` is `blake3:` plus the raw profile digest
in the materialization bundle. These isolated Wasmtime 33 profiles do not claim
equivalence with Molten's ordinary Wasmtime 45 component profile.

## Exact cohorts and retained inputs

The upstream lock pins Wasmtime/wasmtime-wasi **33.0.0**,
wrpc-runtime-wasmtime **0.29.0**, wrpc-transport **0.28.4**,
wit-bindgen **0.36.0**, and wit-bindgen-wrpc **0.10.0**. `cohort.json` retains
these identities. The unmodified upstream Cargo lock and license, source archive,
wrapper authoring sources, locked vendor closure, full WIT, component bindings,
and wRPC Rust bindings are retained. `bindings/*/generation.json` binds actual
generator version/binary digest, WIT/component digests and generation command.
`toolchain.json` records the producer's separately pinned Rust 1.90.0,
wasm32-wasip2, WKG, binding CLI, component linker, wasm-tools, WAC, wasi-virt,
Wizer, Wasmtime CLI, bubblewrap and Octet binaries/configuration identities.
The producer smoke CLI cohort and remote runtime33 cohort are distinct facts.

The guests are dependency-free `no_std` crates. The producer generates their
bindings with the pinned toolchain `wit-bindgen` CLI into `<role>/src/<role>.rs`
inside the measured source closure, and `guest/guest_rt.rs` supplies the bounded
allocator, `cabi_realloc`, `memcmp` and panic handler. No binding macro, build
script or host linker enters the offline guest build. The workspace lock was
mechanically generated with
`cargo generate-lockfile --offline --manifest-path packages/wrpc-reference/guest/Cargo.toml`
and is retained verbatim. Both component compile paths use `--locked --offline`.

## Parent execution acceptance

The 2026-10-30 integration run built both components, but the bundle pipeline
stopped at `octet-report-not-admitted` (see
`.cairn/changes/materialize-wrpc-producer/evidence/integration-2026-10-30.md`).
After materializing the real package, the parent can run the opt-in real-byte rail:

```sh
MANTLE_WRPC_REFERENCE_BUNDLE="$IMMUTABLE_BUNDLE" \
  cargo test -p mantle-wasm-consumer-verifier --features std \
  --test wrpc-producer -- --ignored --test-threads 1
```

It covers actual dual-bundle acceptance, missing component without local
rebuilding, same-length drift, real client/server substitution, and a local
replacement symlink. Equal locally compiled bytes alone cannot be distinguished
by a byte verifier: consumer publication/source admission must independently
reject local-build provenance. Molten must additionally execute the real remote
request/result path and show that a composed substitute or local guest build
cannot satisfy its producer/publication identity requirement.

These artifacts do not establish remote RPC execution, compiler correctness,
component behavior correctness, runtime authority, sandbox effectiveness,
reproducibility, production readiness, or release eligibility.
