# Fixed-output source fetch with reproducible positive and negative patch fixtures.

let mantle = import "lib.ncl" in

let upstream_source = mantle.fetchTarball {
  url = "https://static.crates.io/crates/crc64/crc64-2.0.0.crate",
  hash = "sha256-G2t3VkrTsVRXQrfE4NYeYPnQUgnzxpGz2eI3QlVE/rA=",
  name = "crc64-upstream",
} in
let invalid_hash_source = mantle.fetchTarball {
  url = "https://static.crates.io/crates/crc64/crc64-2.0.0.crate",
  hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
  name = "crc64-invalid-hash",
} in

let local_patch = fun relative_path input_name digest => mantle.fetchurl {
  url = "file://./%{relative_path}",
  hash = digest,
  name = input_name,
} in

let readme_patch = local_patch "patches/readme.patch" "crc64-readme-patch" "blake3-rJ78KgFIs9yiXysyACURCmm4HBvR0RXd2brSN/kqOGs=" in
let invalid_patch = local_patch "patches/invalid-context.patch" "crc64-invalid-patch" "blake3-8HZEn3mB2aajINDuNcFFIi++FNSKDeacJILfgTQm+C8=" in

let apply_patch = fun output_name patch_input patch_input_name => {
  name = output_name,
  builder = "/bin/sh",
  inputs = [upstream_source, patch_input],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      upstream_input=""
      patch_path=""

      for candidate in "$NIX_STORE"/*-crc64-upstream; do
        if [ -f "$candidate/Cargo.toml" ]; then
          upstream_input="$candidate"
          break
        fi
      done
      for candidate in "$NIX_STORE"/*-%{patch_input_name}; do
        if [ -f "$candidate" ]; then
          patch_path="$candidate"
          break
        fi
      done
      if [ -z "$upstream_input" ] || [ -z "$patch_path" ]; then
        echo 'upstream source or patch input was not mounted' >&2
        exit 1
      fi

      $BB mkdir -p "$out"
      $BB cp -r "$upstream_input/." "$out/"
      $BB chmod -R u+w "$out"
      cd "$out"
      "$BB" patch -p1 < "$patch_path"
    "%
  ],
} | mantle.Derivation in

let patched_source = apply_patch "crc64-patched" readme_patch "crc64-readme-patch" in
let invalid = apply_patch "crc64-invalid-patch-result" invalid_patch "crc64-invalid-patch" in

let patch_check = {
  name = "crc64-patch-check",
  builder = "/bin/sh",
  inputs = [patched_source],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      patched_input=""
      for candidate in "$NIX_STORE"/*-crc64-patched; do
        if [ -f "$candidate/README.md" ]; then
          patched_input="$candidate"
          break
        fi
      done
      if [ -z "$patched_input" ]; then
        echo 'patched source was not mounted' >&2
        exit 1
      fi
      $BB grep -F 'patched reproducibly by the Mantle example project' "$patched_input/README.md" >/dev/null
      $BB grep -F 'version = "2.0.0"' "$patched_input/Cargo.toml" >/dev/null
      if $BB grep -F 'THIS HEADING DOES NOT EXIST' "$patched_input/README.md" >/dev/null; then
        echo 'invalid patch content leaked into the patched source' >&2
        exit 1
      fi
      $BB mkdir -p "$out"
      $BB printf '%s\n' 'fetched-and-patched-check: ok' > "$out/result.txt"
    "%
  ],
} | mantle.Derivation in

{
  packages = {
    upstream = upstream_source,
    patched = patched_source,
    invalid-patch = invalid,
    invalid-hash = invalid_hash_source,
  },
  checks = { patch = patch_check },
  default = { package = "patched" },
} | mantle.Project
