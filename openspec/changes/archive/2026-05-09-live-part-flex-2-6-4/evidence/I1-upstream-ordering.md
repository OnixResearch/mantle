# I1 upstream ordering evidence

Task-ID: I1
Covers: bootstrap.part.flex.2.6.4

`bootstrap/flex-2.6.4-musl.ncl` is the refreshed musl-linked Flex part after `flex-2.5.11-musl`, using `tcc-musl-v2`, `musl-1.1.24-tcc-musl`, `make-3.82-tcc`, `sed-4.0.9-musl`, `m4-1.4.7-musl`, and the predecessor `flex-2.5.11-musl`.

The source pin is the upstream westes/flex `v2.6.4` release tarball used for this live-bootstrap part. The derivation now records provenance and first-consumer notes beside the fixed-output source pin.
