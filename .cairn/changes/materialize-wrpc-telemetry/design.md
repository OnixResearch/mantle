# Optional telemetry design

Pin WebAssembly/wasi-otel commit e5262a84044aff064c9599eebc05013bb3432795,
package wasi:otel@0.2.0-rc.2. Retain the exact complete upstream WIT and archive;
compare vendored WIT against the hash-pinned archive during source materialization.
The selected mantle:wrpc-telemetry-pilot/emitter world imports tracing, not the
upstream world that additionally imports logs and metrics.

The emitter's run(mode:u32)->u32 invokes actual current-span-context/on-start/
on-end functions for each span, preserving host context. Mode0 is safe; mode1
includes the secret authorization attribute; mode2 emits33 paired spans. Literal
zero timestamps require no clock calls. The real consumer rejects/redacts secret
attributes and enforces its32-span queue bound. These are parent execution cases,
not producer observations.

The real exporter materializes the standalone emitter under deny-all WASI policy,
with the explicit tracing effect import. Type-only types/datetime dependencies
never authorize clock functions/resources. Exact generated import inspection
remains fail-closed. Runtime33 profile references use the established consumer
encoding and full selected WIT identity.

A separately compiled actual tracing sink implements all3callbacks and validates
safe-span lifecycle for the closed WAC comparison only. Its deterministic context
is not a trusted UCAN context and its invocation is not telemetry export evidence.
The final emitter retains its open import. The optional outer manifest has its
own schema, telemetry entry and capability-prefix mirror, with measured members.
Hello source/WIT/effect/runtime contracts remain unchanged.
