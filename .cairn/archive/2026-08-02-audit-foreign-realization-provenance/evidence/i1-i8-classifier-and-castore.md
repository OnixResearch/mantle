# I1-I8: Pure classification and castore observation

## Implemented core

`crates/crunch-store/src/provenance.rs` defines the bounded policy, payload
classes, reference observations, findings, path facts, and scan result.

The pure core classifies these payloads:

- regular data;
- ELF executables;
- executable scripts;
- symbolic links;
- uncompressed tar archives;
- newc and gzip-newc initrds;
- malformed payloads;
- unsupported containers; and
- unsupported executable bytes.

The resolver checks shebangs, links, and recognized store references. Foreign
store paths, unknown targets, undeclared references, suffix escapes, and link
loops produce typed findings.

## Imperative shell

The shell first loads every requested PathInfo. It verifies the logical path,
trusted signature, receipt-bound NAR facts, admitted closure, and references.
It retains those checked PathInfo values for scanning. Thus, mutable service
order cannot replace a checked root between preflight and traversal.

The scanner then reads directory and blob content from castore. It does not read
exported output paths. Directory digest and size, blob digest and size, path
length, traversal depth, duplicates, and all byte limits are checked.

Tar and initrd readers use in-process bounded parsers. They do not execute shell,
archive, or decompression commands. Unsupported entry types and compressed
container formats fail closed.

## Tests

Positive and negative tests cover arbitrary bytes, deterministic classification,
ELF and script cases, non-UTF-8 input, exact references, suffixes, foreign paths,
unknown targets, links, tar traversal, hidden executable entries, cpio, gzip,
malformed containers, signed PathInfo, inconsistent NAR facts, missing blobs,
duplicate nodes, symlink loops, and named limit findings.
