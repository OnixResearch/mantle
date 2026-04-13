# Bootstrap seed provider.
#
# The public seed contract is a normalized toolchain derivation with a
# stable layout: target-prefixed binutils in `bin/`, a target sysroot at
# `<target>/`, and libgcc available under `<target>/lib/`.
#
# Today that normalized toolchain is built by reducing the fetched musl.cc
# native tarball down to the C/C++ bootstrap surface crunch actually uses.
# Later changes can swap the raw provider without touching the rest of the
# bootstrap chain.

let crunch = import "lib.ncl" in

let provider_id = "musl.cc-native-reduced-v1" in
let raw_name = "musl-gcc-raw" in
let seed_name = "musl-seed-toolchain" in
let seed_target = "x86_64-linux-musl" in
let seed_dynamic_linker = "ld-musl-x86_64.so.1" in
let raw_url = "https://musl.cc/x86_64-linux-musl-native.tgz" in
let raw_hash = "sha256-XpcI34j9YwAQj7qw4DpvXqT1CX00vHcUQbAk/do46jw=" in

let provider_retained_tools = [
  "x86_64-linux-musl-gcc",
  "x86_64-linux-musl-g++",
  "x86_64-linux-musl-c++",
  "x86_64-linux-musl-cpp",
  "x86_64-linux-musl-gcc-ar",
  "x86_64-linux-musl-gcc-nm",
  "x86_64-linux-musl-gcc-ranlib",
  "x86_64-linux-musl-ar",
  "x86_64-linux-musl-as",
  "x86_64-linux-musl-ld",
  "x86_64-linux-musl-nm",
  "x86_64-linux-musl-objcopy",
  "x86_64-linux-musl-objdump",
  "x86_64-linux-musl-ranlib",
  "x86_64-linux-musl-readelf",
  "x86_64-linux-musl-size",
  "x86_64-linux-musl-strings",
  "x86_64-linux-musl-strip",
] in

let provider_dropped_components = [
  "share/locale and other translated message catalogs",
  "Fortran frontend and runtime (f951, x86_64-linux-musl-gfortran, libgfortran*)",
  "coverage helpers (gcov, gcov-dump, gcov-tool)",
  "LTO helpers (lto1, lto-wrapper, lto-dump)",
  "gold and profiling extras (ld.gold, dwp, gprof)",
] in

let provider_notes_list = [
  "Raw provenance stays explicit: one pinned musl.cc tarball with a recursive hash.",
  "Reduced output keeps only the C/C++ compiler, required binutils, headers, sysroot, and runtime libraries used by the current bootstrap chain.",
  "The reduced provider writes provider.json into the store output so operators can inspect provenance and the dropped surface area after fetch bootstrap.",
] in

let raw_toolchain = crunch.fetchTarball {
  url = raw_url,
  hash = raw_hash,
  name = raw_name,
} in

let seed_toolchain = {
  name = seed_name,
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox

      RAW=""
      for d in $NIX_STORE/*-%{raw_name}; do
        if [ -d "$d/bin" ]; then RAW="$d"; break; fi
      done
      if [ -z "$RAW" ]; then
        echo "ERROR: raw seed toolchain not found" >&2
        exit 1
      fi

      $BB mkdir -p "$out/bin" "$out/lib" "$out/libexec" "$out/%{seed_target}" "$out/%{seed_target}/lib"

      copy_tree() {
        src="$1"
        dst="$2"
        if [ -e "$src" ]; then
          $BB rm -rf "$dst" 2>/dev/null || true
          $BB cp -r "$src" "$dst"
          $BB chmod -R u+w "$dst"
        fi
      }

      copy_file() {
        src="$1"
        dst="$2"
        if [ -f "$src" ]; then
          $BB cp -f "$src" "$dst"
          $BB chmod 0555 "$dst"
        fi
      }

      # Keep only the trees the bootstrap chain actually consumes.
      copy_tree "$RAW/include" "$out/include"
      copy_tree "$RAW/lib" "$out/lib"
      copy_tree "$RAW/libexec" "$out/libexec"
      copy_tree "$RAW/%{seed_target}" "$out/%{seed_target}"

      # Keep only compiler drivers the bootstrap chain uses.
      for src_tool in \
        "$RAW/bin/%{seed_target}-gcc" \
        "$RAW/bin/%{seed_target}-gcc-"* \
        "$RAW/bin/%{seed_target}-g++" \
        "$RAW/bin/%{seed_target}-c++" \
        "$RAW/bin/%{seed_target}-cpp" \
        "$RAW/bin/%{seed_target}-gcc-ar" \
        "$RAW/bin/%{seed_target}-gcc-nm" \
        "$RAW/bin/%{seed_target}-gcc-ranlib"; do
        if [ -f "$src_tool" ]; then
          copy_file "$src_tool" "$out/bin/${src_tool##*/}"
        fi
      done

      # Materialize the normalized target-prefixed binutils contract in bin/.
      for tool in addr2line ar as c++filt elfedit ld ld.bfd nm objcopy objdump ranlib readelf size strings strip; do
        src_tool=""
        if [ -x "$RAW/bin/$tool" ]; then
          src_tool="$RAW/bin/$tool"
        elif [ -x "$RAW/%{seed_target}/bin/$tool" ]; then
          src_tool="$RAW/%{seed_target}/bin/$tool"
        fi

        if [ -n "$src_tool" ]; then
          copy_file "$src_tool" "$out/bin/$tool"
          copy_file "$src_tool" "$out/bin/%{seed_target}-$tool"
        fi
      done

      # Normalize the target sysroot view.
      if [ -d "$out/include" ]; then
        $BB rm -rf "$out/%{seed_target}/include"
        copy_tree "$out/include" "$out/%{seed_target}/include"
      elif [ -d "$out/%{seed_target}/sys-include" ]; then
        $BB rm -rf "$out/%{seed_target}/include"
        copy_tree "$out/%{seed_target}/sys-include" "$out/%{seed_target}/include"
      fi

      for lib in libgcc_s.so libgcc_s.so.1 libc.so libstdc++.a libstdc++.so libstdc++.so.6 libsupc++.a; do
        if [ -f "$out/lib/$lib" ]; then
          copy_file "$out/lib/$lib" "$out/%{seed_target}/lib/$lib"
        fi
      done

      # Drop payload the current bootstrap chain does not execute.
      $BB rm -rf "$out/share" 2>/dev/null || true
      $BB rm -rf "$out/libexec/gcc/%{seed_target}"/*/install-tools 2>/dev/null || true
      $BB rm -f \
        "$out/bin/lto-dump" \
        "$out/bin/gcov" \
        "$out/bin/gcov-dump" \
        "$out/bin/gcov-tool" \
        "$out/bin/x86_64-linux-musl-gfortran" \
        "$out/bin/gfortran" \
        "$out/bin/ld.gold" \
        "$out/bin/%{seed_target}-ld.gold" \
        "$out/bin/dwp" \
        "$out/bin/%{seed_target}-dwp" \
        "$out/bin/gprof" \
        "$out/bin/%{seed_target}-gprof" \
        "$out/lib/libgfortran.a" \
        "$out/lib/libgfortran.so" \
        "$out/lib/libgfortran.so.5" \
        "$out/lib/libgfortran.so.5.0.0" \
        "$out/libexec/gcc/%{seed_target}"/*/f951 \
        "$out/libexec/gcc/%{seed_target}"/*/lto1 \
        "$out/libexec/gcc/%{seed_target}"/*/lto-wrapper \
        "$out/libexec/gcc/%{seed_target}"/*/g++-mapper-server \
        2>/dev/null || true

      $BB mkdir -p "$out/share/crunch-bootstrap"
      set -- $($BB du -sm "$RAW")
      RAW_SIZE_MB="$1"
      set -- $($BB du -sm "$out")
      REDUCED_SIZE_MB="$1"

      test "$RAW_SIZE_MB" -gt 0
      test "$REDUCED_SIZE_MB" -gt 0
      test "$REDUCED_SIZE_MB" -lt "$RAW_SIZE_MB"

      $BB cat > "$out/share/crunch-bootstrap/provider.json" << EOF
{
  "provider_id": "%{provider_id}",
  "name": "%{seed_name}",
  "target": "%{seed_target}",
  "dynamic_linker": "%{seed_dynamic_linker}",
  "raw": {
    "name": "%{raw_name}",
    "url": "%{raw_url}",
    "hash": "%{raw_hash}"
  },
  "reduction": {
    "raw_size_mb": $RAW_SIZE_MB,
    "reduced_size_mb": $REDUCED_SIZE_MB,
    "dropped_components": [
      "share/locale and translated catalogs",
      "Fortran frontend and runtime",
      "coverage helpers",
      "LTO helpers",
      "gold and profiling extras"
    ]
  }
}
EOF
      $BB chmod 0444 "$out/share/crunch-bootstrap/provider.json"

      test -x "$out/bin/%{seed_target}-gcc"
      test -x "$out/bin/%{seed_target}-g++"
      test -x "$out/bin/%{seed_target}-ar"
      test -x "$out/bin/%{seed_target}-ld"
      test -f "$out/%{seed_target}/lib/libgcc_s.so.1"
      test -d "$out/%{seed_target}/include"
      test -f "$out/share/crunch-bootstrap/provider.json"
    "%,
  ],
  inputs = [raw_toolchain],
} | crunch.Derivation in

{
  name = seed_name,
  target = seed_target,
  dynamic_linker = seed_dynamic_linker,
  toolchain = seed_toolchain,
  provider = {
    id = provider_id,
    summary = "Reduced C/C++ bootstrap seed carved from the pinned musl.cc native tarball",
    raw = {
      name = raw_name,
      url = raw_url,
      hash = raw_hash,
    },
    retained_tools = provider_retained_tools,
    dropped_components = provider_dropped_components,
    notes = provider_notes_list,
  },
}
