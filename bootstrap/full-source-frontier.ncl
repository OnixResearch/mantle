# Named diagnostic roots for the in-progress full-source provider frontier.
#
# Keeping independent roots in one evaluation lets Mantle deduplicate their
# shared compiler/libc/binutils construction without promoting any root to the
# selected seed contract.

{
  gcc4_cxx = import "gcc-4.0-musl-cxx.ncl",
  bison23 = import "bison-2.3-gcc.ncl",
}
