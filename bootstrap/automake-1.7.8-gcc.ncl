let build_automake = import "automake-gcc-factory.ncl" in
build_automake {
  version = "1.7.8",
  source_url = "https://mirrors.kernel.org/gnu/automake/automake-1.7.8.tar.bz2",
  source_hash = "sha256-VNz8BxyM7EtrGg/MVCcXPO3wZcaVim27+Gt6j8Vtqrs=",
  source_name = "automake-1.7.8-src",
  output_name = "automake-1.7.8-gcc-v1",
  autoconf_input_name = "autoconf-2.55-gcc-v2",
  autoconf_derivation_file = "autoconf-2.55-gcc.ncl",
}
