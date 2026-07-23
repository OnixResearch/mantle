let crunch = import "lib.ncl" in

crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/gawk-3.0.4/sources
  url = "https://mirrors.kernel.org/gnu/gawk/gawk-3.0.4.tar.gz",
  hash = "sha256-Vvj7dAxXMJhOHjWRd6ikD4eyjIERORHR9UcWfBDjAkY=",
  name = "gawk-3.0.4-src",
}
