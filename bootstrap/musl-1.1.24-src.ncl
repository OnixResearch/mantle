# Canonical musl 1.1.24 source used by the native bootstrap candidate.
let crunch = import "lib.ncl" in
crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/musl-1.1.24/sources
  # first-consumer: bootstrap/musl-1.1.24-native.ncl
  url = "https://musl.libc.org/releases/musl-1.1.24.tar.gz",
  hash = "sha256-mSl3003jmxfRju1tcJlRGT75yWw13QJ89diR6M6prkE=",
  name = "musl-1.1.24-src",
}
