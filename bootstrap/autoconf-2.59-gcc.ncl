let build_autoconf = import "autoconf-gcc-factory.ncl" in
build_autoconf {
  version = "2.59",
  source_url = "https://mirrors.kernel.org/gnu/autoconf/autoconf-2.59.tar.bz2",
  source_hash = "sha256-RV4aXRVd3FM7K/jvEmg2SH77YnO8MW6OwHizdeczmjU=",
  source_name = "autoconf-2.59-src",
  output_name = "autoconf-2.59-gcc-v2",
  previous_input_name = "autoconf-2.57-gcc-v1",
  previous_derivation_file = "autoconf-2.57-gcc.ncl",
}
