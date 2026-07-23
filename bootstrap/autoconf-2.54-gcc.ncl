let build_autoconf = import "autoconf-gcc-factory.ncl" in
build_autoconf {
  version = "2.54",
  source_url = "https://mirrors.kernel.org/gnu/autoconf/autoconf-2.54.tar.bz2",
  source_hash = "sha256-d49GJOBXB8ms66PximT8+s0YjgXn14x5l8xWg2W1Cu8=",
  source_name = "autoconf-2.54-src",
  output_name = "autoconf-2.54-gcc-v1",
  previous_input_name = "autoconf-2.53-gcc-v7",
  previous_derivation_file = "autoconf-2.53-gcc.ncl",
}
