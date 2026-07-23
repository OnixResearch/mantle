let build_autoconf = import "autoconf-gcc-factory.ncl" in
build_autoconf {
  version = "2.55",
  source_url = "https://mirrors.kernel.org/gnu/autoconf/autoconf-2.55.tar.bz2",
  source_hash = "sha256-n7fIRRdwsUC78qbTxmLbrnIFim+0jqBoe+bFcMwl7kA=",
  source_name = "autoconf-2.55-src",
  output_name = "autoconf-2.55-gcc-v2",
  previous_input_name = "autoconf-2.54-gcc-v1",
  previous_derivation_file = "autoconf-2.54-gcc.ncl",
}
