# I1 upstream ordering evidence

Task-ID: I1
Covers: bootstrap.part.m4.1.4.7

`bootstrap/m4-1.4.7-musl.ncl` tracks GNU m4 1.4.7 from the live-bootstrap `steps/m4-1.4.7` stage. It sits after the TinyCC/musl/make/sed boundary and before autoconf/autotools consumers. The source pin now records provenance and first-consumer notes beside the fixed-output fetch.
