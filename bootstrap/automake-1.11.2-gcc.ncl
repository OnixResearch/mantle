let build_automake = import "automake-gcc-factory.ncl" in
build_automake {
  version = "1.11.2",
  source_url = "https://mirrors.kernel.org/gnu/automake/automake-1.11.2.tar.bz2",
  source_hash = "sha256-DZ1+dyXNhrs3Z6n82rd64Nfj1Zrzkct+chxhT8QhCFQ=",
  source_name = "automake-1.11.2-src",
  output_name = "automake-1.11.2-gcc-v1",
  autoconf_input_name = "autoconf-2.64-gcc-v1",
  autoconf_derivation_file = "autoconf-2.64-gcc.ncl",
}
