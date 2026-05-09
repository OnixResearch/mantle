# V3 gawk 3.0.4 smoke gate evidence

Task-ID: V3
Covers: bootstrap.part.gawk.3.0.4

No produced `gawk-3.0.4-musl` output path is claimed in this closeout, so no runtime smoke success is claimed.

The derivation now contains fail-closed smokes that must pass once prerequisites produce a real output: local and installed `gawk` must process a tiny program and print `awk-ok`.
