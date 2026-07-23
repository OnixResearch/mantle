let build_automake = import "automake-gcc-factory.ncl" in
build_automake {
  version = "1.9.6",
  source_url = "https://mirrors.kernel.org/gnu/automake/automake-1.9.6.tar.bz2",
  source_hash = "sha256-UtTeZRL99v8NUx1CDFhJGjGErduX6zV4dvPXHHX43Bo=",
  source_name = "automake-1.9.6-src",
  output_name = "automake-1.9.6-gcc-v1",
  autoconf_input_name = "autoconf-2.61-gcc-v1",
  autoconf_derivation_file = "autoconf-2.61-gcc.ncl",
}
