# V2 sed 4.0.9 musl bridge gate evidence

Task-ID: V2
Covers: bootstrap.part.sed.4.0.9.musl

No successful musl source build of GNU sed 4.0.9 is claimed in this closeout.

The derivation still bridges from the earlier `sed-tcc` runtime because the TinyCC/musl handoff currently segfaults while compiling the GNU sed 4.0.9 getline replacement. The bridge is made explicit and fail-closed, but it must not be counted as source-built musl sed proof.
