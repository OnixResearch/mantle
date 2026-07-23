let build_automake = import "automake-gcc-factory.ncl" in
build_automake {
  version = "1.10.3",
  source_url = "https://mirrors.kernel.org/gnu/automake/automake-1.10.3.tar.bz2",
  source_hash = "sha256-TUdNfauySrAadTritfM7Cv61Fx5VQjR0ywzooEfqc9o=",
  source_name = "automake-1.10.3-src",
  output_name = "automake-1.10.3-gcc-v1",
  autoconf_input_name = "autoconf-2.61-gcc-v1",
  autoconf_derivation_file = "autoconf-2.61-gcc.ncl",
}
