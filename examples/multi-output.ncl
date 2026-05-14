# Multi-output derivation example.
#
# The builder creates separate directories for each output:
#   $out/bin/hello       — the binary
#   $dev/include/hello.h — the header
#   $man/share/man/man1  — the man page
#
# mantle build examples/multi-output.ncl
#
# The builder receives $outputs="out dev man" and each output path
# as $out, $dev, $man.

let mantle = import "lib.ncl" in
let seed = import "seed.ncl" in

{
  name = "hello-multi",
  builder = "%{seed.bash}/bin/bash",
  outputs = ["out", "dev", "man"],
  env = {
    PATH = "%{seed.coreutils}/bin",
  },
  args = ["-c", m%"
    # Create output directories
    for output in $outputs; do
      mkdir -p "${!output}"
    done

    # Install binary
    mkdir -p $out/bin
    echo '#!/bin/sh' > $out/bin/hello
    echo 'echo "Hello from multi-output!"' >> $out/bin/hello
    chmod +x $out/bin/hello

    # Install header
    mkdir -p $dev/include
    echo '#define HELLO_VERSION "1.0"' > $dev/include/hello.h

    # Install man page
    mkdir -p $man/share/man/man1
    echo '.TH HELLO 1' > $man/share/man/man1/hello.1
    echo '.SH NAME' >> $man/share/man/man1/hello.1
    echo 'hello - print greeting' >> $man/share/man/man1/hello.1
  "%],
  inputs = [seed.bash, seed.coreutils],
  addressing_mode = 'input-addressed,
} | mantle.Derivation
