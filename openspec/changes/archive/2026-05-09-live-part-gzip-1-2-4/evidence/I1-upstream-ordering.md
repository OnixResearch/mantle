# I1 upstream ordering evidence

Task-ID: I1
Covers: bootstrap.part.gzip.1.2.4

`bootstrap/gzip-tcc.ncl` is the early TinyCC gzip bridge used after `patch-2.5.9` and before later tar/source archive consumers.

Upstream mismatch recorded: `parts.rst` currently labels this section `gzip 1.2.5`, but the implemented live-bootstrap step directory is `steps/gzip-1.2.4`, with source pin `gzip-1.2.4.tar.gz` and checksum `1ca41818a23c9c59ef1d5e1d00c0d5eaa2285d931c0fb059637d7c0cc02ad967`.

Crunch therefore renamed the active part change from `live-part-gzip-1-2-5` to `live-part-gzip-1-2-4` and tracks the implemented derivation/source, not the stale heading.
