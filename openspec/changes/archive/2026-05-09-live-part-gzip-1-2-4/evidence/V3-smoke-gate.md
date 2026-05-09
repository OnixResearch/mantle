# V3 gzip 1.2.4 smoke gate evidence

Task-ID: V3
Covers: bootstrap.part.gzip.1.2.4

No produced `gzip-1.2.4-tcc` output path is claimed in this closeout, so no runtime smoke success is claimed.

The derivation now contains fail-closed installed `gzip`/`gunzip` checks and a compression/decompression roundtrip smoke that must pass once prerequisites produce a real output.
