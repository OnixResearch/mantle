# Canonical TinyCC 0.9.27 source for self-hosted bootstrap candidates.
let crunch = import "lib.ncl" in
crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/tcc-0.9.27/sources
  # first-consumer: bootstrap/tcc-musl-selfhost.ncl
  url = "https://download-mirror.savannah.gnu.org/releases/tinycc/tcc-0.9.27.tar.bz2",
  hash = "sha256-J1qVIv8ti91hOO6Gp2Wqho3t8C7jQHcXpTI85IPwZn0=",
  name = "tcc-0.9.27-src",
}
