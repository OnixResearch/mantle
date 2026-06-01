# Fast local named-output layout.
#
# Usage:
#   mantle build examples/local-output-layout.ncl --no-substitute
#
# The builder creates three named outputs without generated seed material:
#   $out/bin/show-layout
#   $dev/include/local_output_layout.h
#   $doc/share/doc/local-output-layout/README

let mantle = import "lib.ncl" in

{
  name = "local-output-layout",
  builder = "/bin/sh",
  outputs = ["out", "dev", "doc"],
  args = ["-c", m%"
    BB=/bin/busybox
    $BB mkdir -p $out/bin $dev/include $doc/share/doc/local-output-layout
    $BB cat > $out/bin/show-layout << 'EOF'
#!/bin/sh
echo "runtime output uses local-output-layout"
EOF
    $BB chmod +x $out/bin/show-layout
    $BB echo '#define LOCAL_OUTPUT_LAYOUT 1' > $dev/include/local_output_layout.h
    $BB echo 'local output layout docs' > $doc/share/doc/local-output-layout/README
  "%],
} | mantle.Derivation
