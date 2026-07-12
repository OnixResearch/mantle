# Official KernelScript v0.1.2 source archive as a flat fixed-output input.
#
# The official release publishes the archive SHA-256. This derivation does not
# unpack or compile it: fetchTarball would require a recursive/NAR identity that
# upstream does not publish, while the compiler dependency closure is not yet
# authoritatively locked.

let mantle = import "../../lib/lib.ncl" in

mantle.fetchurl {
  name = "kernelscript-0.1.2-source.tar.gz",
  url = "https://github.com/multikernel/kernelscript/releases/download/v0.1.2/kernelscript-0.1.2-source.tar.gz",
  hash = "sha256-mgC5bh8SfUgGwosHbycKzcS/SoxVjKY2v9n0kmi0ecE=",
}
