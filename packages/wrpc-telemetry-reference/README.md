# Optional OpenTelemetry-WASI component producer

This is a separate, optional materialization. The existing hello component pair,
its unversioned RPC contract, and its deny-all profiles are unchanged.

The reviewed upstream is
[`WebAssembly/wasi-otel`](https://github.com/WebAssembly/wasi-otel/tree/e5262a84044aff064c9599eebc05013bb3432795),
commit `e5262a84044aff064c9599eebc05013bb3432795`, package
`wasi:otel@0.2.0-rc.2`. The complete pinned WIT is retained verbatim, including
`types`, clock datetime type dependencies, metrics/log interfaces and upstream
world metadata. Only the tracing effect is imported by the selected emitter
world; retaining other WIT source does not authorize its effects.

## Exact artifact command and identity

```sh
nix build "path:$PWD#packages.x86_64-linux.wrpc-telemetry-reference-bundle" \
  --no-link --print-out-paths -L
```

The immutable public output address is obtained from the actual Mantle revision
published by the release owner, not a fabricated revision or observed digest:

```sh
: "${MANTLE_PUBLISHED_REVISION:?release owner supplies published Mantle commit}"
test "${#MANTLE_PUBLISHED_REVISION}" -eq 40
nix build "github:OnixResearch/mantle/${MANTLE_PUBLISHED_REVISION}#packages.x86_64-linux.wrpc-telemetry-reference-bundle" \
  --no-link --print-out-paths -L
```

The implementation invokes the real `mantle wasm-component build` exporter,
with actual offline compilation, generated bindings, WAC, deny-all WASI
virtualization, import inspection, validation, smoke and Octet receipts. It
retains the selected standalone emitter, full source/vendor closure, upstream
archive/license, Cargo lock, component/wRPC bindings and tool cohort identities.
The guests are dependency-free `no_std` crates whose bindings the producer
generates with the pinned toolchain `wit-bindgen` CLI (`--runtime-path
crate::guest_rt::rt`); `guest/guest_rt.rs` supplies that runtime surface, the
bounded allocator and `cabi_realloc`. The workspace lock was mechanically
generated with
`cargo generate-lockfile --offline --manifest-path packages/wrpc-telemetry-reference/guest/Cargo.toml`
and is retained verbatim. The isolated consumer Wasmtime/wasmtime-wasi cohort is
33.0.0. Producer CLI identities are separately measured in `toolchain.json`; no
equivalence is implied.

The 2026-10-30 integration run built the components, but the bundle pipeline
stopped at `wasi-virt-failed`. See
`.cairn/changes/materialize-wrpc-producer/evidence/integration-2026-10-30.md`.
No bundle, emitter receipt, trusted host observation, consumer telemetry export
or publication identity is claimed.

## Standalone emitter contract

World `mantle:wrpc-telemetry-pilot/emitter` exports
`run(mode: u32) -> u32`. It returns 42 only after the selected callbacks return.
For **each span** it calls `current-span-context`, then `on-start` with that
exact context, then `on-end` with the same context:

| Mode | Actual emitted input | Consumer acceptance expectation |
|---|---|---|
| 0 | One span, `component-rpc-call`, instrumentation `molten-component-rpc`, empty attributes/events/links, unset status | Accepted by separately admitted host |
| 1 | One span with `authorization` attribute and JSON string value `"secret"` | Rejected/redacted; secret must not reach diagnostic export |
| 2 | 33 spans, each with its own current/start/end sequence | The 33rd exceeds the host's 32-span queue bound |

Literal zero timestamps avoid clock effects. Trace/span context comes from the
actual consumer host; the emitter does not fabricate a consumer receipt. Other
modes trap. `cases.json` retains these positive/negative execution scenarios for
the parent consumer rail; their outcomes are requirements, not observed results.

Only `wasi:otel/tracing@0.2.0-rc.2` effect functions are approved. The consumer
may resolve `wasi:otel/types@0.2.0-rc.2` and
`wasi:clocks/wall-clock@0.2.0` **only as type-only dependencies**, checking that
all exported component items are types, with zero functions/resources. That does
not permit `now`, clock effects, randomness, stdout, filesystem or network.
The actual generated import inventory remains subject to the exporter inspection
receipt and consumer structural checking; no unexecuted inventory is asserted.

## Capability handoff

`manifest.json` schema `mantle-wrpc-telemetry-reference-bundle-v1` includes
`source.repository`, `source.revision`, `capability_root = "capability"`, all
measured file members, and `telemetry` with `bundle`, `bundle_identity_blake3`,
`final_portable`, `runtime_profile`, `runtime_profile_ref`, `world = "emitter"`.
Use `telemetry/materialization-bundle.json` and
`telemetry/runtime-profile.json`; open the declared capability directory below
the authorized package root. It mirrors exact exporter logical paths without an
ambient filesystem-root capability or identity/receipt rewriting.

Profile `molten-wrpc-wasmtime33-otel-tracing-v1` uses the consumer's exact
line/Debug-record reference encoding and the selected WIT source content ref,
not a JSON-digest approximation. Its effect import list is tracing-only and the
ordinary WASI subsystem list is empty.

The source-built `comparison/sink.wasm` implements the actual three tracing WIT
callbacks, validates lifecycle and safe span fields, and supplies a deterministic
comparison context. The exporter binds it to the exact final emitter for a
runtime-only `run(0)` WAC smoke. This is explicitly **comparison-only**:
`telemetry/artifacts/runtime-composed.wasm` and the sink are not admitted as the
standalone emitter or trusted consumer host evidence.

The consumer must independently authorize the actual UCAN-bound host capability,
validate trusted context and lifecycle, enforce redaction/queue limits, and
write bounded redacted diagnostic records only below its admitted file
capability. This producer is not native OTLP, does not claim the OTLP protocol,
and is not evidence that trusted consumer telemetry export was executed. Missing,
mismatched, substituted, locally rebuilt or closed-comparison artifacts must not
repair or satisfy the optional published-producer admission requirement.
