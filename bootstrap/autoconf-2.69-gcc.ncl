let build_autoconf = import "autoconf-gcc-factory.ncl" in
build_autoconf {
  version = "2.69",
  source_url = "https://mirrors.kernel.org/gnu/autoconf/autoconf-2.69.tar.xz",
  source_hash = "sha256-tpVLeuvm1/He9UvOAGzup4fc7YTQYp1xWTowDs1rR4I=",
  source_name = "autoconf-2.69-src",
  output_name = "autoconf-2.69-gcc-v1",
  previous_input_name = "autoconf-2.64-gcc-v1",
  previous_derivation_file = "autoconf-2.64-gcc.ncl",
}
