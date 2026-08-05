# Pin the Picolibc 1.8.12 release source for the StageX comparison diagnostic.
#
# Research-only input for the `evaluate-picolibc-stagex-runtime` change. This
# pin does not admit Picolibc into the StageX lineage, the normalized
# provider graph, or any parity row.
#
# Upstream: https://github.com/picolibc/picolibc/releases/tag/1.8.12
# Release asset: picolibc-1.8.12.tar.xz
# Upstream flat sha256 (informational, not the pin):
#   64e8c412e1c40fa6eb1a72d2b5cdbcbfe6ceca4cbea454edbad54557ffc747fa
#
# To update the pin: set the new URL, use a dummy hash, run
# `mantle build bootstrap/picolibc-1.8.12-src.ncl --fix`, and commit.

let crunch = import "../lib/lib.ncl" in

{
  picolibc-src = crunch.fetchTarball {
    url = "https://github.com/picolibc/picolibc/releases/download/1.8.12/picolibc-1.8.12.tar.xz",
    hash = "sha256-a2XLUN2U49Lpsyizzb2cQpMbww0cmOUUKdgIj6OHhpQ=",
    name = "picolibc-1.8.12-src",
  },
}
