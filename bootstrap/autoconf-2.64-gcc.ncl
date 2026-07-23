let build_autoconf = import "autoconf-gcc-factory.ncl" in
build_autoconf {
  version = "2.64",
  source_url = "https://mirrors.kernel.org/gnu/autoconf/autoconf-2.64.tar.xz",
  source_hash = "sha256-4yjN0lzGMVILd/gdx9bPCYbEIgDyD1NX5TUnNBuAmiQ=",
  source_name = "autoconf-2.64-src",
  output_name = "autoconf-2.64-gcc-v1",
  previous_input_name = "autoconf-2.61-gcc-v1",
  previous_derivation_file = "autoconf-2.61-gcc.ncl",
}
