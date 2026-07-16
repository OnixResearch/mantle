# Build a tiny cowsay-compatible shell script using only the sandbox shell and busybox.
#
# Usage:
#   mantle build examples/cowsay.ncl --import-path lib --store /tmp/mantle-store

let mantle = import "lib.ncl" in
{
  name = "cowsay",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox

      $BB mkdir -p "$out/bin"
      $BB cat > "$out/bin/cowsay" << 'EOF'
#!/bin/sh
set -eu

if [ "$#" = "0" ]; then
  message="Moo from Mantle!"
else
  message="$*"
fi

printf '< %s >\n' "$message"
printf ' \\   ^__^\n'
printf '  \\  (oo)\\_______\n'
printf '     (__)\\       )\\/\\\n'
printf '         ||----w |\n'
printf '         ||     ||\n'
EOF
      $BB chmod +x "$out/bin/cowsay"
    "%,
  ],
} | mantle.Derivation
