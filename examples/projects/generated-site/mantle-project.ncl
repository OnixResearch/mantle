# Project-shaped generated static site with a package and content check.

let mantle = import "lib.ncl" in

let site_package = {
  name = "project-generated-site",
  builder = "/bin/sh",
  env = {
    SITE_TITLE = "Mantle Project Gallery",
    SITE_TAGLINE = "Generated as a complete project package.",
  },
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      $BB mkdir -p "$out/assets"
      $BB cat > "$out/index.html" << EOF
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>$SITE_TITLE</title>
    <link rel="stylesheet" href="assets/site.css">
  </head>
  <body>
    <main>
      <h1>$SITE_TITLE</h1>
      <p>$SITE_TAGLINE</p>
    </main>
  </body>
</html>
EOF
      $BB cat > "$out/assets/site.css" << 'EOF'
body { font-family: sans-serif; margin: auto; }
main { padding: 1em; }
EOF
      $BB printf '%s\n' 'index.html' 'assets/site.css' > "$out/manifest.txt"
    "%
  ],
} | mantle.Derivation in

let site_check = {
  name = "project-generated-site-check",
  builder = "/bin/sh",
  inputs = [site_package],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      site_input=""
      for candidate in "$NIX_STORE"/*-project-generated-site; do
        if [ -f "$candidate/index.html" ]; then
          site_input="$candidate"
          break
        fi
      done
      if [ -z "$site_input" ]; then
        echo 'generated site input was not mounted' >&2
        exit 1
      fi
      $BB grep -q 'Mantle Project Gallery' "$site_input/index.html"
      $BB grep -q 'font-family: sans-serif' "$site_input/assets/site.css"
      $BB grep -q 'assets/site.css' "$site_input/manifest.txt"
      $BB mkdir -p "$out"
      $BB printf '%s\n' 'generated-site-check: ok' > "$out/result.txt"
    "%
  ],
} | mantle.Derivation in

{
  packages = { site = site_package },
  checks = { site-content = site_check },
  default = { package = "site" },
} | mantle.Project
