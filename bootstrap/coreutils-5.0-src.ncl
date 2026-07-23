# Canonical coreutils 5.0 source used by the musl bootstrap rebuild.
let crunch = import "lib.ncl" in
crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/coreutils-5.0/sources
  url = "https://mirrors.kernel.org/gnu/coreutils/coreutils-5.0.tar.bz2",
  hash = "sha256-VtfMrIYHZZ4YT21zYxctKfFEvGXDKerxOvCO7mmOPlA=",
  name = "coreutils-5.0-src",
}
