let crunch = import "../lib/lib.ncl" in
crunch.fetchTarball {
  name = "gcc-10.5.0-src",
  url = "https://ftpmirror.gnu.org/gcc/gcc-10.5.0/gcc-10.5.0.tar.xz",
  hash = "sha256-t4vO308XvqCAgwrl2jS1llkIvHHfSV4wlkdLMkYfrkU=",
}
