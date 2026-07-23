let build_automake = import "automake-gcc-factory.ncl" in
build_automake {
  version = "1.15.1",
  source_url = "https://mirrors.kernel.org/gnu/automake/automake-1.15.1.tar.xz",
  source_hash = "sha256-/KHwBUWoEgUxjLuLFOpqM4c0UXUDWiyiBV4HBTwOcZU=",
  source_name = "automake-1.15.1-src",
  output_name = "automake-1.15.1-gcc-v1",
  autoconf_input_name = "autoconf-2.69-gcc-v1",
  autoconf_derivation_file = "autoconf-2.69-gcc.ncl",
}
