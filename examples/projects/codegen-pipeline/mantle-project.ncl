# Project-shaped schema -> generated source -> runnable package pipeline.

let mantle = import "lib.ncl" in

let model_derivation = {
  name = "codegen-model",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      $BB mkdir -p "$out"
      $BB printf '%s\n' 'message=Hello from generated project code' > "$out/model.txt"
    "%
  ],
} | mantle.Derivation in

let generated_derivation = {
  name = "codegen-generated-source",
  builder = "/bin/sh",
  inputs = [model_derivation],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      model_input=""
      for candidate in "$NIX_STORE"/*-codegen-model; do
        if [ -f "$candidate/model.txt" ]; then
          model_input="$candidate/model.txt"
          break
        fi
      done
      if [ -z "$model_input" ]; then
        echo 'code generation model was not mounted' >&2
        exit 1
      fi
      message=$($BB cut -d= -f2- "$model_input")
      $BB mkdir -p "$out/generated"
      $BB printf "MESSAGE='%s'\n" "$message" > "$out/generated/message.sh"
    "%
  ],
} | mantle.Derivation in

let app_package = {
  name = "codegen-demo",
  builder = "/bin/sh",
  inputs = [generated_derivation],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      generated_input=""
      for candidate in "$NIX_STORE"/*-codegen-generated-source; do
        if [ -f "$candidate/generated/message.sh" ]; then
          generated_input="$candidate/generated/message.sh"
          break
        fi
      done
      if [ -z "$generated_input" ]; then
        echo 'generated source was not mounted' >&2
        exit 1
      fi
      . "$generated_input"
      $BB mkdir -p "$out/bin" "$out/share/codegen-demo"
      $BB cp "$generated_input" "$out/share/codegen-demo/message.sh"
      $BB cat > "$out/bin/codegen-demo" << EOF
#!/bin/sh
USAGE_EXIT_CODE=64
if [ "\$#" -ne 0 ]; then
  echo 'usage: codegen-demo' >&2
  exit "\$USAGE_EXIT_CODE"
fi
printf '%s\n' '$MESSAGE'
EOF
      $BB chmod +x "$out/bin/codegen-demo"
    "%
  ],
} | mantle.Derivation in

let app_check = {
  name = "codegen-demo-check",
  builder = "/bin/sh",
  inputs = [app_package],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      app_input=""
      for candidate in "$NIX_STORE"/*-codegen-demo; do
        if [ -x "$candidate/bin/codegen-demo" ]; then
          app_input="$candidate"
          break
        fi
      done
      if [ -z "$app_input" ]; then
        echo 'generated application was not mounted' >&2
        exit 1
      fi
      output=$($app_input/bin/codegen-demo)
      if [ "$output" != 'Hello from generated project code' ]; then
        echo "unexpected generated output: $output" >&2
        exit 1
      fi
      if "$app_input/bin/codegen-demo" unexpected; then
        echo 'generated application accepted an invalid argument' >&2
        exit 1
      fi
      $BB mkdir -p "$out"
      $BB printf '%s\n' 'codegen-demo-check: ok' > "$out/result.txt"
    "%
  ],
} | mantle.Derivation in

{
  packages = {
    generated-source = generated_derivation,
    app = app_package,
  },
  checks = { app = app_check },
  default = { package = "app" },
} | mantle.Project
