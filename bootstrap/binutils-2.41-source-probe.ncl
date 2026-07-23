let crunch = import "../lib/lib.ncl" in
crunch.fetchTarball {
  name = "binutils-2.41-src",
  url = "https://ftpmirror.gnu.org/binutils/binutils-2.41.tar.xz",
  hash = "sha256-8dGjwSZTk28XKXwkODNezqFVLXDA6Ycf/dLF+RoYGWo=",
}
