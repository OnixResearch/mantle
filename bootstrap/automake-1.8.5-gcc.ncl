let build_automake = import "automake-gcc-factory.ncl" in
build_automake {
  version = "1.8.5",
  source_url = "https://mirrors.kernel.org/gnu/automake/automake-1.8.5.tar.bz2",
  source_hash = "sha256-rDrGs5BMDfV5Oe5PfBsrlncUcQZfTaFPACrNP2vxoWo=",
  source_name = "automake-1.8.5-src",
  output_name = "automake-1.8.5-gcc-v1",
  autoconf_input_name = "autoconf-2.59-gcc-v2",
  autoconf_derivation_file = "autoconf-2.59-gcc.ncl",
}
