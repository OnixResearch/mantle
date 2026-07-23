let build_automake = import "automake-gcc-factory.ncl" in
build_automake {
  version = "1.6.3",
  source_url = "https://mirrors.kernel.org/gnu/automake/automake-1.6.3.tar.bz2",
  source_hash = "sha256-5gX3QsW95k/3hksMMINEYthaHm5o7iv9NXLHAWRA7fc=",
  source_name = "automake-1.6.3-src",
  output_name = "automake-1.6.3-gcc-v4",
  autoconf_input_name = "autoconf-2.52-gcc-v6",
  autoconf_derivation_file = "autoconf-2.52-gcc.ncl",
}
