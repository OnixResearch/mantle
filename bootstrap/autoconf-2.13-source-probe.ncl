let crunch = import "lib.ncl" in
crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/autoconf/autoconf-2.13.tar.gz",
  hash = "sha256-Gl7VQbtxjjyFy8rCJelqLhdl4TbO3L0FPumoBQb2+28=",
  name = "autoconf-2.13-source-probe",
}
