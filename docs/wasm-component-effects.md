# Consumer-hosted effect-world component production

## Authority and scope

Mantle produces exact source/WIT/artifact/cohort/receipt bytes. Octet owns the
closed `molten-component-effects-v1` static profile. Molten owns
`molten:component-effects@1.0.0`, canonical Preserves, application effect
admission, and real actor/system-extension execution. Neither an Octet pass nor
Structural+Bytes verification is an execution or host-authority receipt.

The checked neutral source is `fixtures/wasm-component-effects/`. Its selected
`wit/runtime.wit` is an exact copy of the Molten-owned WIT; the preparation
program compares it byte-for-byte with the caller's explicit authoritative WIT
input. The namespace/name manifest string does not omit version authority:
version `1.0.0` is bound by those exact versioned WIT bytes and their bundle
member identity.

## Runtime validation policy

`ComponentPipelineRequest.runtime_validation` defaults to
`{"mode":"standalone-smoke"}`. Existing standalone requests still execute real
`wasmtime run` and require exact declared stdout.

Host-import requests explicitly use:

```json
{
  "mode": "consumer-host-required",
  "wit_package": "molten:component-effects",
  "wit_world": "actor-effects",
  "required_imports": ["effect-request"]
}
```

This must match manifest WIT and remaining-import declarations exactly, and
requires `runtime_invoke: null` and `expected_runtime_stdout: ""`. No fallback
host or mock CLI execution is admitted. Static stages remain mandatory. A
`consumer-host-runtime-required` graph node has `not-run` status, exact final
artifact/runtime-profile bindings, no tool, and no execution claims.

`MaterializationBundle.runtime_validation` exposes the same public typed
`RuntimeValidation` value. Consumer mode is serialized and participates in the
bundle identity. Standalone mode is omitted from canonical serialization so old
bundle identities remain valid. The existing bundle fields separately bind
selected WIT bytes, final artifact, Octet/runtime profiles, and cohort. Consumers
must fulfill the requirement with their real host before activation or execution
receipts; a static materialization success does not fulfill it.

Non-WASI outputs are copied unchanged only after import inspection exactly
matches the deny-all virtualization plan's declared remaining host imports.
Artifacts with WASI imports still enter the existing WASI-Virt path. No WASI
allowance or baseline profile limit is broadened.

## Neutral deterministic fixture

The input is one canonical packed Preserves ByteString containing command bytes,
not a raw prefix outside the canonical value. The fixture does not interpret
application operations or canonical payload authority.
Framing is tag `b2`, then the minimal unsigned little-endian base-128 length,
then exactly that many bytes. Nonminimal lengths, overflow, trailing bytes,
and commands larger than 65,536 bytes are rejected. This is the closed
ByteString framing from Preserves 5.0.0-rc.7 `packed/writer.rs` (`varint` and
`write_atom`), not a second general canonicalization implementation.

| Command bytes inside ByteString | Behavior |
| --- | --- |
| `00` followed by canonical payload bytes | Return the payload bytes unchanged |
| `01` followed by canonical request bytes | Call actual `effect-request` with those exact bytes; return its exact success/error |
| `02` | Actual guest trap |
| `03` | Return deliberately malformed packed Preserves byte `ff` |
| `04` | Infinite volatile computation for real fuel/deadline interruption |
| `05` followed by exactly eight little-endian count bytes | Perform that many deterministic recurrence iterations; return a canonical ByteString of eight little-endian state bytes |

The recurrence begins at state/index zero and updates
`state = state * 6364136223846793005 + (index | 1)` modulo 2^64. Finite output is
`b2 08` followed by eight little-endian state bytes. Pure-true whole input is
`b2 02 00 81`; trap is `b2 01 02`; malformed is `b2 01 03`; infinite computation
is `b2 01 04`; zero-iteration finite input is
`b2 09 05 00 00 00 00 00 00 00 00`. Loop mode has no hostcalls or clocks, so
fuel and deadline observations come from the actual runtime. Empty/unknown
commands and invalid finite-workload width return a guest
error. Guest allocation is fixed at 256 KiB and reclaims canonical input/output
buffers between invocations.

System-extension callback envelopes/outcomes are product-specific. Supply a
Molten-owned genuine guest source directory using the optional preparation
argument below; do not synthesize a host callback success or put callback codecs
in Mantle. The source must include Cargo.toml/Cargo.lock, a cdylib root,
`src/lib.rs` using generated `actor_effects`, `wit/runtime.wit`, and exact
`wkg.lock`. Any Rust dependencies must be inside the source closure: the compiler
runs locked/offline with no ambient registry or sibling paths.

Even an empty package closure uses the pinned `wkg` owner's exact serialized
lock: version 1, its generated-file preamble, and `packages = []`. Regenerate a
source lock through the actual tool before preparing the source closure. A
hand-written version-only lock changes during `wkg wit fetch` and must be
rejected as byte drift; never repair that by rewriting a produced bundle.

The neutral guest's checked release profile uses size optimization and LTO.
Its checked `.cargo/config.toml` disables the unsupported `sign-ext` target
feature for `wasm32-wasip2` and passes the admitted linker's
`--initial-memory=2097152` and `--max-memory=2097152` options. The resulting
linear memory has exactly 32 fixed 64-KiB pages. Molten's unchanged hosted
profile requires equal initial and maximum memory/table bounds; an absent
maximum is not a fixed-memory declaration even when the guest never grows it.
Generated adapter tables must independently retain equal initial and maximum
bounds. Product-owned callback sources declare their own equivalent compiler
cohort. These genuine compiler inputs are part of the remeasured source closure.
They do not relax the closed checker/runtime profiles or rewrite generated
bindings or produced Wasm to manufacture admission.

## Immutable checker cohort

The producer pins published Octet revision
`abb6082062251944a40093a274df3d35331fa168`. Its full catalog BLAKE3 is
`09f1fbb7ce4d9da085b3f1325dbd5e8ae1c17bde2755caf79c1d2850e67b116a`;
the effects-profile identity is
`a3ad131b214390c9e0ab515e0a63698067dcdd15f2fd4abbc1fd2d120726292e`.
The portable and Molten baseline identities remain unchanged. The realized
toolchain cohort is
`703be19cf0bab382e10a65790c39540a45c5ab89262070ef9036c1ee35f5f306`.
This identifies the producer/checker tools, not downstream host execution.

## Production commands

Run long toolchain builds/proofs detached through Pueue, following
`.pi/skills/pueue-long-builds/SKILL.md`. The following are command bodies for that
single integration verification phase, not claims that they have run.

First publish/review the checker and update the actual immutable pin coherently:
`flake.nix` input/source revision and `flake.lock`; Rust
`OCTET_SOURCE_REVISION` and `OCTET_CONFIG_BLAKE3`; confirm the three exact profile
identity constants and wasm-tools cohort against the actual generated receipts.
Never install a temporary revision or sibling source checkout in the product.
The full generated catalog digest changes even though existing baseline profile
DTO identities remain unchanged.

```sh
TOOLCHAIN="$(nix build .#wasm-component-toolchain --no-link --print-out-paths)"
nix develop -c cargo build -p mantle --bin mantle
nix develop -c cargo run -p crunch-wasm-component --example prepare_effect_fixture -- \
  "$TOOLCHAIN" /absolute/run/fixture \
  /absolute/molten/wit/molten-component-effects/runtime.wit \
  EXACT_MOLTEN_RUNTIME_PROFILE_BLAKE3
```

The output directory must not already exist. For a real product-owned callback
guest, append `/absolute/molten/guest-source` as the fifth argument. Preparation
verifies the toolchain and exact WIT, runs the pinned generator without generated
text rewriting, rehashes the whole resulting source, and writes `request.ncl`,
`request.json`, and a binding-generation record. Its binding command is exactly:

```sh
"$TOOLCHAIN/bin/wit-bindgen" rust --world actor-effects --generate-all \
  --disable-run-ctors-once-workaround --disable-custom-section-link-helpers \
  --out-dir OUTPUT/source/src OUTPUT/source/wit
```

The selected request has no synthetic WIT package dependency, no ambient WASI,
no Wizer/AOT transform, and a root `invoke` export with exact effect-request host
surface. Execute the unchanged bounded build surface:

Use the actual compiled Mantle binary path (for ordinary local target layout,
`target/debug/mantle`) with:

```sh
target/debug/mantle --json --store /absolute/run/store --state-dir /absolute/run/state \
  wasm-component build /absolute/run/fixture/request.ncl --out /absolute/run/bundle
```

Verify the resulting report with
`crunch_wasm_component::verify_pipeline_execution_report_files` and the bundle
with the consumer verifier's Structural+Bytes layer / real Molten `VerifiedBundle`
loader, remeasuring the exact published source tree (including directories), WIT,
lock, final artifact, and every receipt. Inspect the typed host requirement, then
operate actual Molten actor and system-extension consumers over those exact
bytes. Retain real success/effect-denial/trap/malformed/fuel/deadline and lifecycle
receipts before cutover; never hand-author or rewrite bundle fields.

Focused source coverage for the final phase:

```sh
nix develop -c cargo test -p crunch-wasm-component-core
nix develop -c cargo test -p crunch-wasm-component
nix develop -c cargo test -p mantle-wasm-consumer-verifier
nix develop -c cargo test -p mantle --test wasm_component_cli -- --test-threads=1
nix develop -c cairn validate --root .
```

The final pipeline request and bundle, not these command examples, bind actual
source/artifact identities and absolute locators. Source preparation alone is
not a generated fixture, passing check, publication, or runtime proof.
