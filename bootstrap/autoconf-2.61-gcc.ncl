let build_autoconf = import "autoconf-gcc-factory.ncl" in
build_autoconf {
  version = "2.61",
  source_url = "https://mirrors.kernel.org/gnu/autoconf/autoconf-2.61.tar.bz2",
  source_hash = "sha256-L7gjFM87mxjMvwA1cg2g0uueg9bmerkYwLFjvnPBV8Q=",
  source_name = "autoconf-2.61-src",
  output_name = "autoconf-2.61-gcc-v1",
  previous_input_name = "autoconf-2.59-gcc-v2",
  previous_derivation_file = "autoconf-2.59-gcc.ncl",
}
