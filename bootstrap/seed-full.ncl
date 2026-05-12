# Full-source bootstrap seed provider.
#
# Exposes the same normalized seed contract as seed-legacy.ncl but
# backed by the from-source chain: hex0 -> stage0-posix -> mes -> tcc ->
# gcc version ladder -> musl + binutils.

let crunch = import "lib.ncl" in

let gcc10 = import "gcc-10.ncl" in
let musl = import "musl-full.ncl" in
let binutils = import "binutils-full.ncl" in

let seed_name = "full-source-seed-toolchain" in
let seed_target = "x86_64-linux-musl" in
let seed_dynamic_linker = "ld-musl-x86_64.so.1" in

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

let seed_toolchain = {
  name = seed_name,
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox

      find_input() {
        for d in $NIX_STORE/*-"$1"; do
          if [ -d "$d" ]; then echo "$d"; return; fi
        done
        echo "ERROR: input $1 not found" >&2; exit 1
      }

      GCC=$(find_input gcc-10.5.0)
      MUSL=$(find_input musl-1.2.5-full)
      BINUTILS=$(find_input binutils-2.41-full)

      TGT="%{seed_target}"
      $BB mkdir -p "$out/bin" "$out/lib" "$out/libexec"
      $BB mkdir -p "$out/$TGT/include" "$out/$TGT/lib"
      $BB mkdir -p "$out/share/crunch-bootstrap"

      # --- Compiler binaries ---
      # Copy GCC compiler drivers with target-prefixed names
      for tool in gcc g++ cpp gcc-ar gcc-nm gcc-ranlib; do
        if [ -x "$GCC/bin/$tool" ]; then
          $BB cp "$GCC/bin/$tool" "$out/bin/$TGT-$tool"
        elif [ -x "$GCC/bin/$TGT-$tool" ]; then
          $BB cp "$GCC/bin/$TGT-$tool" "$out/bin/$TGT-$tool"
        fi
      done
      # c++ is a symlink to g++
      if [ -x "$out/bin/$TGT-g++" ]; then
        $BB ln -sf "$TGT-g++" "$out/bin/$TGT-c++"
      fi

      # --- GCC libexec (cc1, cc1plus, collect2, lto*) ---
      GCC_LIBEXEC="$GCC/libexec/gcc/$TGT"
      if [ -d "$GCC_LIBEXEC" ]; then
        $BB mkdir -p "$out/libexec/gcc/$TGT"
        $BB cp -r "$GCC_LIBEXEC"/* "$out/libexec/gcc/$TGT/" 2>/dev/null || true
      fi

      # --- Binutils ---
      for tool in ar as ld nm objcopy objdump ranlib readelf size strings strip; do
        if [ -x "$BINUTILS/bin/$tool" ]; then
          $BB cp "$BINUTILS/bin/$tool" "$out/bin/$TGT-$tool"
        elif [ -x "$BINUTILS/bin/$TGT-$tool" ]; then
          $BB cp "$BINUTILS/bin/$TGT-$tool" "$out/bin/$TGT-$tool"
        fi
      done

      # --- Musl headers and libraries ---
      $BB cp -r "$MUSL/include"/* "$out/$TGT/include/" 2>/dev/null || true
      $BB cp "$MUSL/lib"/*.a "$out/$TGT/lib/" 2>/dev/null || true
      $BB cp "$MUSL/lib"/*.o "$out/$TGT/lib/" 2>/dev/null || true
      if [ -f "$MUSL/lib/libc.so" ]; then
        $BB cp "$MUSL/lib/libc.so" "$out/$TGT/lib/"
        $BB ln -sf libc.so "$out/$TGT/lib/ld-musl-x86_64.so.1" 2>/dev/null || true
      fi

      # --- libgcc ---
      for gcclib in "$GCC/lib/gcc/$TGT"/*; do
        if [ -d "$gcclib" ]; then
          $BB cp "$gcclib"/*.a "$out/$TGT/lib/" 2>/dev/null || true
          $BB cp "$gcclib"/*.o "$out/$TGT/lib/" 2>/dev/null || true
          if [ -f "$gcclib/libgcc_s.so.1" ]; then
            $BB cp "$gcclib/libgcc_s.so.1" "$out/$TGT/lib/"
            $BB ln -sf libgcc_s.so.1 "$out/$TGT/lib/libgcc_s.so" 2>/dev/null || true
          fi
        fi
      done

      # --- Provider metadata ---
      $BB cat > "$out/share/crunch-bootstrap/provider.json" << 'PROVIDER'
{
  "provider_id": "full-source-v1",
  "name": "full-source-seed-toolchain",
  "summary": "Full-source seed: hex0 -> stage0-posix -> mes -> tcc -> gcc ladder -> musl + binutils",
  "target": "x86_64-linux-musl",
  "dynamic_linker": "ld-musl-x86_64.so.1",
  "source_root": {
    "manifest_digest": "pending-full-source-manifest-digest",
    "source_chain": "live-bootstrap commit 9a268c4c39cae952b268bc86da342be2175f03d4"
  },
  "gcc_version": "10.5.0",
  "musl_version": "1.2.5",
  "binutils_version": "2.41",
  "reduction": {
    "retained_tools": [
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
      "x86_64-linux-musl-include"
    ],
    "dropped_components": [],
    "notes": "Normalization contract only; full trusted runtime proof remains gated by archived predecessor evidence until the source-built chain is rebuilt end-to-end."
  }
}
PROVIDER

      # --- Verify normalized seed contract ---
      for tool in %{std.array.join " " provider_retained_tools}; do
        test -x "$out/bin/$tool" || { echo "ERROR: retained tool $tool not found" >&2; exit 1; }
      done
      test -x "$out/bin/$TGT-gcc" || { echo "ERROR: $TGT-gcc not found" >&2; exit 1; }
      test -x "$out/bin/$TGT-as" || { echo "ERROR: $TGT-as not found" >&2; exit 1; }
      test -x "$out/bin/$TGT-ld" || { echo "ERROR: $TGT-ld not found" >&2; exit 1; }
      test -f "$out/$TGT/lib/libc.a" || { echo "ERROR: libc.a not found" >&2; exit 1; }
      test -s "$out/$TGT/lib/crt1.o" || { echo "ERROR: crt1.o not found" >&2; exit 1; }
      test -s "$out/$TGT/lib/crti.o" || { echo "ERROR: crti.o not found" >&2; exit 1; }
      test -s "$out/$TGT/lib/crtn.o" || { echo "ERROR: crtn.o not found" >&2; exit 1; }
      test -d "$out/$TGT/include" || { echo "ERROR: headers not found" >&2; exit 1; }
      test -s "$out/share/crunch-bootstrap/provider.json" || { echo "ERROR: provider metadata missing" >&2; exit 1; }

      echo "full-source-seed-toolchain normalization complete"
    "%,
  ],
  inputs = [gcc10, musl, binutils],
} | crunch.Derivation in

{
  name = seed_name,
  target = seed_target,
  dynamic_linker = seed_dynamic_linker,
  toolchain = seed_toolchain,
  provider = {
    id = "full-source-v1",
    summary = "Full-source seed normalization contract for the GCC/musl/binutils ladder",
    retained_tools = provider_retained_tools,
    dropped_components = [],
    notes = [
      "Intended source chain starts from a 229-byte hex0 seed; current promotion remains gated by predecessor runtime evidence.",
      "Chain: hex0 -> stage0-posix -> mes -> tcc -> gcc-4.0 -> gcc-4.7 -> gcc-10 -> musl + binutils.",
      "Source chain reference: live-bootstrap commit 9a268c4c39cae952b268bc86da342be2175f03d4.",
    ],
  },
}
