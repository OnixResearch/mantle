# ADR 0043: Stage the StageX binutils install under its logical prefix

## Status

Accepted (2026-07-30)

## Context

The authenticated component build uses the logical prefix `/mantle/stagex/binutils-probe-output`. This fixed value removes scratch paths from `ld-new`.

A direct install to that prefix is not permitted on the host. Replacing the prefix with a scratch path changes embedded linker paths and output identity.

The first install probe changed only `prefix`. Zlib still used its configured `libdir` and tried to create `/mantle`. The install failed closed.

## Decision Drivers

- Preserve the fixed logical prefix in compiled outputs.
- Do not write to an ambient host root.
- Run authenticated Make install targets.
- Keep install files in one bounded scratch tree.
- Require exact native tool identities and no predecessor delegation.
- Preserve the recipe’s positive and negative runtime smokes.

## Decision

Mantle runs each authenticated component install with the configured logical prefix. It sets `DESTDIR` to a new private staging directory.

The effective install root is:

```text
<scratch>/install-destdir/mantle/stagex/binutils-probe-output
```

Mantle requires 11 regular executable ELF files: `as`, `ld`, `ar`, `ranlib`, `nm`, `objcopy`, `objdump`, `readelf`, `size`, `strings`, and `strip`.

Each file must match its checked BLAKE3 identity. No file can contain the protected TinyCC path.

Mantle creates one target-prefixed link for each required tool. Each link points to the corresponding absolute path under the logical prefix.

The bounded install accepts 4,891 or 4,892 total sed calls. Configure can make one optional empty sed call. The build-only boundary remains 4,771 or 4,772 calls.

Mantle then runs these authenticated smokes:

- assemble and link a static `_start` program;
- execute that program and require status 42;
- create, index, and list an archive;
- inspect the object with `nm`, `objdump`, and `readelf`;
- copy the object with `objcopy`;
- reject malformed assembly, object, and archive inputs;
- reject any partial negative output.

## Alternatives Considered

### Install directly to the logical prefix

Rejected because the host does not grant that write authority.

### Replace every configured directory with a scratch path

Rejected because it changes embedded paths and output identities.

### Copy selected build-tree binaries without Make install

Rejected because it bypasses authenticated install behavior and can omit required files.

### Accept tools by name and mode only

Rejected because it does not bind installed bytes to the authenticated build.

## Consequences

- The installed tool set has stable logical paths and exact identities.
- The staged tree can become the input to later provider publication.
- The runtime smokes cover the recipe’s declared positive and negative boundary.
- The source-built Make diagnostic about `-l` remains in install evidence.
- Generated child authorization, provider receipt publication, native TinyCC consumption, compiler correctness, and provider admission remain separate work.
