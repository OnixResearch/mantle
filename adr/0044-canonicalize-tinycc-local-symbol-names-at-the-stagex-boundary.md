# ADR 0044: Canonicalize TinyCC local symbol names at the StageX boundary

## Status

Accepted (2026-07-30)

## Context

TinyCC 0.9.27 creates anonymous local ELF symbols with names such as `L.521018500`. The numeric suffix comes from process-address state. Two compilations of the same source can therefore produce different object bytes.

This variation changed archive and executable BLAKE3 identities. Exact protected path-and-digest authorization cannot accept unknown output variants.

The affected symbols are local entries in an ELF symbol table. Their names do not define linker-visible authority.

A later fresh fixed-point run found the same variation in the retained TinyCC self-host objects. The already-linked compiler stayed stable, but the retained main object, `libtcc.a`, and object-tree evidence did not. Two proof contexts also produced different decimal suffix widths because the process-derived values had different magnitudes. Padding within the existing name width did not remove that variation.

After the self-host evidence was stable, the hydrated proof exposed the same variation in the native-musl objects and `libc.a`. Archive metadata normalization cannot repair nondeterministic member bytes.

A later full-source attempt exposed a remaining binutils boundary. The protected C helper replaced process-derived values but preserved each original decimal width. Retained archive members used names such as `L.0000012`; fresh members used `L.000000012`. Their code, symbol facts, and member order matched, but their string-table sizes and archive identities did not.

## Decision Drivers

- Keep exact protected executable digests.
- Do not accept an unbounded set of observed compiler outputs.
- Do not change the authenticated binutils source recipe.
- Preserve section layout, symbol order, bindings, values, sizes, and generated code.
- Keep the repair bounded, deterministic, and fail-closed.
- Publish changed files through create-new staging.

## Decision

Mantle builds a small static ELF canonicalizer from authenticated source. Mantle strips this helper at link time so that the helper has a stable identity before its first execution.

The canonicalizer accepts one absolute, bounded, regular ELF64 little-endian file. It rejects symlinks, path escapes, malformed section tables, oversized files, and unsupported symbol-table layouts.

For each `SHT_SYMTAB`, the protected binutils canonicalizer changes only local symbol names that exactly match `L.[0-9]+`. It replaces the decimal suffix with the symbol-table index, padded to the existing byte length. It does not change the file size or any symbol fact other than the matched local name bytes.

The canonicalizer writes a create-new sibling file, flushes it, and renames it over the compiler output. The bounded TinyCC wrapper runs it after each object compilation and executable link.

Mantle checks the canonicalizer source and binary by exact BLAKE3 identity. A positive smoke compiles the same anonymous-symbol fixture twice and requires equal canonical outputs. A negative smoke requires malformed input rejection.

The TinyCC self-host evidence path uses a pure Rust core with one stricter canonical form. It accepts a bounded relocatable ELF64 file with no program headers and exactly one `SHT_SYMTAB`. It rebuilds the linked string table so each matched local name uses a 10-digit symbol index. It updates every symbol-name offset, later section offsets, the string-table size, and the section-table offset. It also replaces the old alignment gap before the next file-backed section with the unique zero-filled canonical gap. It rejects overlaps, shared conflicting names, non-canonical input padding, unsupported layouts, and malformed bounds.

The shell reads each declared retained object, calls the core, publishes the result through a create-new sibling, and rebuilds `libtcc.a` with the declared predecessor. This path does not modify the already-linked compiler. It fails if no matching local symbol exists, if an input is not a bounded regular ELF64 file, or if the archive rebuild fails.

The native-musl stage applies the same shell to every compiled object before CRT installation and `libc.a` assembly. It requires at least one matched name across the bounded source closure. The protected compiler execution plan does not change because canonicalization is an in-process pure transformation followed by the existing archive command.

The binutils stage keeps the protected C helper after every TinyCC compile and link. Immediately after each of the four closed archive Make targets, Mantle applies a second pure Rust core to the exact archive format produced by `stagex-binutils-ar-runner.sh`. The core accepts only deterministic metadata, bounded short or GNU long names, and ELF object members. It applies the same 10-digit local-symbol canonical form to each member, rebuilds the archive with the original member names and order, and publishes it through a create-new sibling before any later component consumes it. Malformed metadata, symbol indexes, names, padding, non-ELF members, unsupported special members, and limit violations fail closed.

## Alternatives Considered

### Accept many exact digests for each generated path

Rejected because compiler address state can create a much larger unknown output set than the bounded configure variants.

### Update expected digests after each build

Rejected because this records nondeterminism instead of removing it.

### Remove all ELF symbol tables

Rejected because relocatable objects need symbol tables for later archive and link steps.

### Patch TinyCC source

Deferred because the current StageX boundary already has a bounded wrapper. A compiler patch would change the protected predecessor and require a wider self-hosting review.

### Canonicalize the complete binutils tree after all Make targets

Rejected because later binutils components consume earlier archives. Mantle canonicalizes each closed archive immediately after its own Make target while retaining object-level protected canonicalization at the earliest compiler boundary.

## Consequences

- Repeated TinyCC object outputs can have stable BLAKE3 identities.
- Protected execution can keep exact path-and-digest authorization.
- Retained TinyCC self-host objects and `libtcc.a` have repeatable evidence identities without changing the compiler binary.
- Native-musl CRT and archive inputs use the same canonical ELF boundary.
- Binutils archives use one fixed 10-digit member identity across retained and fresh decimal widths.
- The four archive identities change once; protected binutils executable and installed-tool identities remain unchanged.
- Local anonymous symbol names no longer preserve TinyCC's process-derived numeric suffixes or their variable decimal widths.
- This decision does not prove compiler correctness, source semantics, provider admission, or debug-symbol equivalence.
