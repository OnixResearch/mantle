# Build a tiny static website with no generated seed or network access.
#
# Usage:
#   mantle build examples/static-site.ncl --no-substitute

let mantle = import "lib.ncl" in
{
  name = "static-site",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox

      $BB mkdir -p "$out/assets"
      $BB cat > "$out/index.html" << 'EOF'
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>Built by Mantle</title>
    <link rel="stylesheet" href="assets/site.css">
  </head>
  <body>
    <main>
      <h1>Hello from a Mantle build</h1>
      <p>This directory is a content-addressed static site artifact.</p>
    </main>
  </body>
</html>
EOF
      $BB cat > "$out/assets/site.css" << 'EOF'
body { font-family: sans-serif; margin: auto; }
main { padding: 1em; }
EOF
    "%
  ],
} | mantle.Derivation
