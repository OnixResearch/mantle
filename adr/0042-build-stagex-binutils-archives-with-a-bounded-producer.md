# ADR 0042: Build StageX binutils archives with a bounded producer

## Status

Accepted (2026-07-30)

## Context

Authenticated binutils Make needs GNU `ar` behavior for four static archives. The protected StageX lineage does not yet contain a native `ar`.

Configure previously used `AR=true`. BFD Make then reported success without creating `.libs/libbfd.a`. The next component failed when it tried to link this missing archive. Make status alone did not detect the false-green build.

An ambient host `ar` would add undeclared authority. A later native `ar` is not available at this stage.

## Decision Drivers

- Keep authenticated configure and Make in control of the build.
- Do not authorize a host archive tool.
- Produce GNU-compatible archives that the authenticated linker can use.
- Normalize archive metadata and preserve member order.
- Bound paths, members, file sizes, archive size, and invocation count.
- Reject partial, substituted, and pre-existing outputs.
- Validate semantic outputs and exact BLAKE3 identities after Make.

## Decision

Mantle uses `bootstrap/stagex-binutils-ar-runner.sh` as the canonical archive producer for this bounded build.

Mantle binds the script by BLAKE3 and runs it with the exact protected full Bash. The script accepts only `rc` or `cru` mode and relative regular-file members. It rejects path traversal, symlinks, duplicate names, empty member sets, existing outputs, oversized inputs, and unsupported modes.

The producer supports GNU short names and the GNU long-name table. It writes deterministic member headers with zero time, user, and group values. It writes mode `100644`, preserves member order, and pads odd payloads.

The producer creates a new file in a private scratch directory. It publishes the file only after all size and structure checks pass. The invocation budget is 64. The selected build must contain exactly four successful archive operations:

1. `libiberty.a` with 66 members;
2. `libz.a` with 15 members;
3. `.libs/libbfd.a` with 59 members;
4. `.libs/libopcodes.a` with five members.

The TinyCC wrapper sets compiled object modes to `0644` before Make can archive them.

Mantle passes the archive producer to configure and Make. This is necessary because generated `libtool` records the configure-time `AR` value.

Mantle uses the fixed logical `tooldir` `/mantle/stagex/binutils-probe-output`. This prevents scratch paths from changing `ld-new` bytes.

The full build accepts only the three observed sed counts: 4,771, 4,772, and 4,773. Authenticated configure can run up to two optional empty sed probes. All three forms produce the same required component identities.

## Alternatives Considered

### Use ambient `ar`

Rejected because it adds undeclared host execution authority.

### Keep `AR=true`

Rejected because BFD Make can report success without `libbfd.a`.

### Port all binutils archive behavior

Rejected because this stage needs only one closed GNU archive subset. A broad implementation would add unused authority.

### Build native `ar` first

Deferred. The native binutils handoff is the result of this build, not an available predecessor.

## Consequences

- Authenticated Make builds `libiberty`, `zlib`, BFD, opcodes, binutils, gas, gprof, and ld.
- Mantle rejects Make success when a required archive or executable is absent.
- Mantle validates the exact BLAKE3 identity of eight required component outputs.
- Archive publication is bounded and does not overwrite an existing target.
- Generated `libtool`, `config.status`, Make children, install output, runtime smokes, and protected-transition audit integration still need separate authority and evidence.
- This decision does not prove provider admission, compiler correctness, or general binutils behavior.
