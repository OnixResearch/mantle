# Example crunch.ncl project file.
#
# This demonstrates the project outputs schema.
#
# Usage:
#   1. Generate seed: mantle bootstrap -o examples/project/seed.ncl
#   2. Build default: cd examples/project && mantle build
#   3. Build hello:   cd examples/project && mantle build .#hello
#   4. Run hello:     cd examples/project && mantle run .#hello
#   5. Run checks:    cd examples/project && mantle build .#checks.test-hello

let mantle = import "lib.ncl" in
let seed = import "seed.ncl" in

{
  packages = {
    hello = {
      name = "hello",
      builder = "%{seed.bash}/bin/bash",
      args = ["-c", m%"
        mkdir -p $out/bin
        echo '#!/bin/sh' > $out/bin/hello
        echo 'echo "Hello from mantle project!"' >> $out/bin/hello
        chmod +x $out/bin/hello
      "%],
      inputs = [seed.bash],
    } | mantle.Derivation,

    goodbye = {
      name = "goodbye",
      builder = "%{seed.bash}/bin/bash",
      args = ["-c", m%"
        mkdir -p $out/bin
        echo '#!/bin/sh' > $out/bin/goodbye
        echo 'echo "Goodbye from mantle project!"' >> $out/bin/goodbye
        chmod +x $out/bin/goodbye
      "%],
      inputs = [seed.bash],
    } | mantle.Derivation,
  },

  checks = {
    test-hello = {
      name = "test-hello",
      builder = "%{seed.bash}/bin/bash",
      args = ["-c", m%"
        echo "running test..."
        echo "test passed"
        mkdir -p $out
        echo "ok" > $out/result
      "%],
      inputs = [seed.bash],
    } | mantle.Derivation,
  },

  default = {
    package = "hello",
  },
} | mantle.Project
