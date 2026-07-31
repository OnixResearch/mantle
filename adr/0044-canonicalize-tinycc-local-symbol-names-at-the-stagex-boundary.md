# ADR 0044: Canonicalize TinyCC local symbol names at the StageX boundary

## Status

Accepted (2026-07-30)

## Context

TinyCC 0.9.27 creates anonymous local ELF symbols with names such as `L.521018500`. The numeric suffix comes from process-address state. Two compilations of the same source can therefore produce different object bytes.

This variation changed archive and executable BLAKE3 identities. Exact protected path-and-digest authorization cannot accept unknown output variants.

The affected symbols are local entries in an ELF symbol table. Their names do not define linker-visible authority.

A later fresh fixed-point run found the same variation in the retained TinyCC self-host objects. The already-linked compiler stayed stable, but the retained main object, `libtcc.a`, and object-tree evidence did not. Two proof contexts also produced different decimal suffix widths because the process-derived values had different magnitudes. Padding within the existing name width did not remove that variation.

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

## Alternatives Considered

### Accept many exact digests for each generated path

Rejected because compiler address state can create a much larger unknown output set than the bounded configure variants.

### Update expected digests after each build

Rejected because this records nondeterminism instead of removing it.

### Remove all ELF symbol tables

Rejected because relocatable objects need symbol tables for later archive and link steps.

### Patch TinyCC source

Deferred because the current StageX boundary already has a bounded wrapper. A compiler patch would change the protected predecessor and require a wider self-hosting review.

### Canonicalize whole archives after Make

Rejected because executable probes and intermediate links also require stable identities. Object-level canonicalization closes the earliest affected boundary.

## Consequences

- Repeated TinyCC object outputs can have stable BLAKE3 identities.
- Protected execution can keep exact path-and-digest authorization.
- Retained TinyCC self-host objects and `libtcc.a` have repeatable evidence identities without changing the compiler binary.
- Binutils archive and tool identities change once to their canonical values.
- Local anonymous symbol names no longer preserve TinyCC's process-derived numeric suffixes or their variable decimal widths.
- This decision does not prove compiler correctness, source semantics, provider admission, or debug-symbol equivalence.
