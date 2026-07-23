let build_autoconf = import "autoconf-gcc-factory.ncl" in
build_autoconf {
  version = "2.57",
  source_url = "https://mirrors.kernel.org/gnu/autoconf/autoconf-2.57.tar.bz2",
  source_hash = "sha256-Pz9baKCHcs+UMxLv7aN4dIHhlrHdnpxjTgTCDU85ThI=",
  source_name = "autoconf-2.57-src",
  output_name = "autoconf-2.57-gcc-v1",
  previous_input_name = "autoconf-2.55-gcc-v2",
  previous_derivation_file = "autoconf-2.55-gcc.ncl",
}
