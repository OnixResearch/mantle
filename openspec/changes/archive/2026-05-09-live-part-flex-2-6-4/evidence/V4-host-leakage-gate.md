# V4 host-leakage gate evidence

Task-ID: V4
Covers: bootstrap.part.flex.2.6.4

No leakage-clean runtime transcript is claimed because no successful trusted build output is claimed in this closeout.

The derivation remains constrained to declared bootstrap inputs (`stage0`, `tcc-musl-v2`, `musl-1.1.24-tcc-musl`, `make-3.82-tcc`, `sed-4.0.9-musl`, `m4-1.4.7-musl`, `flex-2.5.11-musl`, and the fixed-output Flex source). Host flex, Nix-provided flex, host GCC, and legacy compiler outputs remain forbidden as proof substitutes.
