# Runtime-admitted full-source bootstrap provider candidate.
#
# Selection remains separate in seed.ncl. This expression normalizes the final
# GCC 10/musl 1.2.5/binutils 2.41 artifacts and executes the complete bounded
# tool/runtime smoke surface. External admission binds the resulting tree to a
# BLAKE3 output identity because an output cannot contain its own content hash.

let crunch = import "lib.ncl" in
let gcc10 = crunch.derivationFile "gcc-10-final.ncl" in
let musl = crunch.derivationFile "musl-full.ncl" in
let binutils = crunch.derivationFile "binutils-full.ncl" in

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
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      PROVIDER_NAME=%{seed_name}
      PUBLIC_TARGET=%{seed_target}
      DYNAMIC_LINKER=%{seed_dynamic_linker}
      GCC_TARGET=x86_64-unknown-linux-musl
      GCC_VERSION=10.5.0
      EXPECTED_EXIT_STATUS=42
      WORK=/tmp/full-source-seed-provider

      find_input() {
        input_suffix=$1
        for input_path in $NIX_STORE/*-"$input_suffix"; do
          if [ -e "$input_path" ]; then
            echo "$input_path"
            return 0
          fi
        done
        echo "ERROR: input $input_suffix not found" >&2
        exit 1
      }

      GCC=$(find_input gcc-10.5.0-musl-final-v1)
      MUSL=$(find_input musl-1.2.5-gcc10-v1)
      BINUTILS=$(find_input binutils-2.41-gcc10-v1)
      GCC_RUNTIME_DIR="$GCC/lib/gcc/$GCC_TARGET/$GCC_VERSION"
      GCC_SHARED_DIR="$GCC/lib64"
      SYSROOT="$out/$PUBLIC_TARGET"
      PROVIDER_LIB="$SYSROOT/lib"

      test -x "$GCC/bin/gcc"
      test -x "$GCC/bin/g++"
      test -x "$GCC/libexec/gcc/$GCC_TARGET/$GCC_VERSION/cc1"
      test -x "$GCC/libexec/gcc/$GCC_TARGET/$GCC_VERSION/cc1plus"
      test -s "$GCC_RUNTIME_DIR/libgcc.a"
      test -s "$GCC_RUNTIME_DIR/libgcc_eh.a"
      test -s "$GCC_SHARED_DIR/libgcc_s.so.1"
      test -s "$GCC_SHARED_DIR/libstdc++.so.6.0.28"
      test -s "$MUSL/lib/libc.a"
      test -s "$MUSL/lib/libc.so"
      test -s "$MUSL/lib/ld-musl-x86_64.so.1"

      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK" "$out/bin" "$out/lib" "$out/lib64" "$out/libexec" "$out/share" "$SYSROOT/include" "$PROVIDER_LIB"

      # Preserve the final compiler's own internals and headers inside the
      # normalized output. Target-prefixed wrappers below select only these
      # copied paths and the candidate sysroot.
      $BB cp -R "$GCC/lib/gcc" "$out/lib/"
      $BB cp -R "$GCC/libexec/gcc" "$out/libexec/"
      if [ -d "$GCC/include" ]; then
        $BB cp -R "$GCC/include" "$out/"
      fi
      if [ -d "$GCC/include-fixed" ]; then
        $BB cp -R "$GCC/include-fixed" "$out/"
      fi
      for compiler_binary in gcc g++ cpp; do
        $BB cp "$GCC/bin/$compiler_binary" "$out/bin/$compiler_binary.real"
        $BB chmod 755 "$out/bin/$compiler_binary.real"
      done
      $BB cp "$GCC_SHARED_DIR/libgcc_s.so.1" "$out/lib64/"
      $BB cp "$GCC_SHARED_DIR/libstdc++.so.6.0.28" "$out/lib64/"
      $BB ln -sf libgcc_s.so.1 "$out/lib64/libgcc_s.so"
      $BB ln -sf libstdc++.so.6.0.28 "$out/lib64/libstdc++.so.6"
      $BB ln -sf libstdc++.so.6 "$out/lib64/libstdc++.so"

      $BB cp -R "$MUSL/include"/* "$SYSROOT/include/"
      for runtime_file in libc.a libc.so libm.a libpthread.a librt.a libdl.a libutil.a crt1.o Scrt1.o rcrt1.o crti.o crtn.o; do
        test -s "$MUSL/lib/$runtime_file"
        $BB cp "$MUSL/lib/$runtime_file" "$PROVIDER_LIB/$runtime_file"
      done
      $BB cp "$MUSL/lib/ld-musl-x86_64.so.1" "$PROVIDER_LIB/ld-musl-x86_64.so.1"
      $BB cp "$GCC_RUNTIME_DIR/libgcc.a" "$PROVIDER_LIB/libgcc.a"
      $BB cp "$GCC_RUNTIME_DIR/libgcc_eh.a" "$PROVIDER_LIB/libgcc_eh.a"
      $BB cp "$GCC_SHARED_DIR/libgcc_s.so.1" "$PROVIDER_LIB/libgcc_s.so.1"
      $BB cp "$GCC_SHARED_DIR/libstdc++.a" "$PROVIDER_LIB/libstdc++.a"
      $BB cp "$GCC_SHARED_DIR/libsupc++.a" "$PROVIDER_LIB/libsupc++.a"
      $BB cp "$GCC_SHARED_DIR/libstdc++.so.6.0.28" "$PROVIDER_LIB/libstdc++.so.6.0.28"
      $BB ln -sf libgcc_s.so.1 "$PROVIDER_LIB/libgcc_s.so"
      $BB ln -sf libstdc++.so.6.0.28 "$PROVIDER_LIB/libstdc++.so.6"
      $BB ln -sf libstdc++.so.6 "$PROVIDER_LIB/libstdc++.so"

      for tool in ar as ld nm objcopy objdump ranlib readelf size strings strip; do
        test -x "$BINUTILS/bin/$tool"
        $BB cp "$BINUTILS/bin/$tool" "$out/bin/$tool"
        $BB cp "$BINUTILS/bin/$tool" "$out/bin/$PUBLIC_TARGET-$tool"
        $BB chmod 755 "$out/bin/$tool" "$out/bin/$PUBLIC_TARGET-$tool"
      done
      for tool_pair in 'gcc-ar:ar' 'gcc-nm:nm' 'gcc-ranlib:ranlib'; do
        public_tool=${tool_pair%%:*}
        binutils_tool=${tool_pair#*:}
        $BB cp "$BINUTILS/bin/$binutils_tool" "$out/bin/$PUBLIC_TARGET-$public_tool"
        $BB chmod 755 "$out/bin/$PUBLIC_TARGET-$public_tool"
      done

      write_compiler_wrapper() {
        wrapper_path=$1
        real_driver=$2
        compiler_mode=$3
        $BB cat > "$wrapper_path" <<EOF
#!/bin/sh
provider_bin=\${0%/*}
provider_root=\$(CDPATH= cd -- "\$provider_bin/.." && pwd)
public_target="$PUBLIC_TARGET"
gcc_target="$GCC_TARGET"
gcc_version="$GCC_VERSION"
sysroot="\$provider_root/\$public_target"
runtime_dir="\$provider_root/lib/gcc/\$gcc_target/\$gcc_version"
link_mode=dynamic
for mode_argument in "\$@"; do
  case "\$mode_argument" in
    -static) link_mode=static ;;
  esac
done
if [ "\$link_mode" = static ]; then
  libc_runtime="\$sysroot/lib/libc.a"
  libm_runtime="\$sysroot/lib/libm.a"
else
  libc_runtime="\$sysroot/lib/libc.so"
  libm_runtime="\$sysroot/lib/libc.so"
fi
export MANTLE_GCC_ASSEMBLER="\$provider_root/bin/as"
export MANTLE_GCC_LINKER="\$provider_root/bin/ld"
export MANTLE_GCC_LINK_LIBC="\$libc_runtime"
export MANTLE_GCC_LINK_LIBM="\$libm_runtime"
export MANTLE_GCC_LINK_LIBGCC="\$sysroot/lib/libgcc.a"
export MANTLE_GCC_LINK_LIBGCC_EH="\$sysroot/lib/libgcc_eh.a"
export MANTLE_GCC_LINK_LIBGCC_S="\$sysroot/lib/libgcc_s.so.1"
export MANTLE_GCC_LINK_CRT="\$sysroot/lib"
exec "\$provider_root/bin/$real_driver.real" \
  -B"\$provider_root/bin/" \
  -B"\$runtime_dir/" \
  --sysroot="\$sysroot" \
  -Wl,-dynamic-linker,"\$sysroot/lib/$DYNAMIC_LINKER" \
  -Wl,-rpath,"\$sysroot/lib" \
  "\$@"
EOF
        $BB chmod 755 "$wrapper_path"
      }
      write_compiler_wrapper "$out/bin/$PUBLIC_TARGET-gcc" gcc c
      write_compiler_wrapper "$out/bin/$PUBLIC_TARGET-g++" g++ cxx
      $BB ln -sf "$PUBLIC_TARGET-g++" "$out/bin/$PUBLIC_TARGET-c++"
      $BB cat > "$out/bin/$PUBLIC_TARGET-cpp" <<EOF
#!/bin/sh
exec "$out/bin/$PUBLIC_TARGET-gcc" -E "\$@"
EOF
      $BB chmod 755 "$out/bin/$PUBLIC_TARGET-cpp"

      for retained_tool in %{std.string.join " " provider_retained_tools}; do
        test -x "$out/bin/$retained_tool"
      done
      test -x "$out/libexec/gcc/$GCC_TARGET/$GCC_VERSION/cc1"
      test -x "$out/libexec/gcc/$GCC_TARGET/$GCC_VERSION/cc1plus"
      for required_runtime in crt1.o crti.o crtn.o libc.a libc.so libgcc.a libgcc_eh.a libgcc_s.so.1 libstdc++.a libstdc++.so.6.0.28; do
        test -s "$PROVIDER_LIB/$required_runtime"
      done
      for shared_runtime in "$PROVIDER_LIB/libgcc_s.so.1" "$PROVIDER_LIB/libstdc++.so.6.0.28"; do
        "$out/bin/readelf" -d "$shared_runtime" | $BB grep -q 'NEEDED.*\[libc.so\]'
        if "$out/bin/readelf" -d "$shared_runtime" | $BB grep -q 'NEEDED.*\[/'; then
          echo "ERROR: normalized provider shared runtime retains an absolute libc dependency: $shared_runtime" >&2
          exit 1
        fi
      done

      $BB cat > "$WORK/positive.c" <<'EOF'
#include <stdio.h>
int main(void) { return puts("provider-c-runtime") < 0; }
EOF
      $BB cat > "$WORK/positive.cpp" <<'EOF'
#include <cstdio>
struct base { virtual ~base() {} };
struct derived final : base { int value = 42; };
int main() {
  base *item = new derived();
  derived *typed = dynamic_cast<derived *>(item);
  int result = 1;
  try { if (typed != nullptr) throw typed->value; }
  catch (int value) { result = value == 42 ? 0 : 1; }
  delete item;
  return result;
}
EOF
      "$out/bin/$PUBLIC_TARGET-gcc" -static "$WORK/positive.c" -o "$WORK/c-static"
      "$WORK/c-static" | $BB grep -q 'provider-c-runtime'
      "$out/bin/$PUBLIC_TARGET-gcc" "$WORK/positive.c" -o "$WORK/c-dynamic"
      "$out/bin/readelf" -l "$WORK/c-dynamic" | $BB grep -q "$out/$PUBLIC_TARGET/lib/$DYNAMIC_LINKER"
      "$WORK/c-dynamic" | $BB grep -q 'provider-c-runtime'
      "$out/bin/$PUBLIC_TARGET-g++" -std=c++17 -static "$WORK/positive.cpp" -o "$WORK/cxx-static"
      "$WORK/cxx-static"
      "$out/bin/$PUBLIC_TARGET-g++" -std=c++17 "$WORK/positive.cpp" -o "$WORK/cxx-dynamic"
      "$WORK/cxx-dynamic"
      "$out/bin/readelf" -d "$WORK/cxx-dynamic" | $BB grep -q 'libstdc++.so.6'
      "$out/bin/readelf" -d "$WORK/cxx-dynamic" | $BB grep -q 'libgcc_s.so.1'

      $BB cat > "$WORK/positive.s" <<'EOF'
.equ mantle_exit_syscall, 60
.equ mantle_expected_status, 42
.global _start
.text
_start:
  mov $mantle_exit_syscall, %rax
  mov $mantle_expected_status, %rdi
  syscall
EOF
      "$out/bin/$PUBLIC_TARGET-as" -o "$WORK/positive.o" "$WORK/positive.s"
      "$out/bin/$PUBLIC_TARGET-ld" -o "$WORK/positive-bin" "$WORK/positive.o"
      $BB chmod 755 "$WORK/positive-bin"
      set +e
      "$WORK/positive-bin"
      assembly_status=$?
      set -e
      test "$assembly_status" -eq "$EXPECTED_EXIT_STATUS"
      "$out/bin/$PUBLIC_TARGET-ar" rc "$WORK/libpositive.a" "$WORK/positive.o"
      "$out/bin/$PUBLIC_TARGET-ranlib" "$WORK/libpositive.a"
      "$out/bin/$PUBLIC_TARGET-ar" t "$WORK/libpositive.a" | $BB grep -q positive.o
      "$out/bin/$PUBLIC_TARGET-readelf" -h "$WORK/positive.o" | $BB grep -q ELF64
      "$out/bin/$PUBLIC_TARGET-nm" "$WORK/positive.o" | $BB grep -q _start
      "$out/bin/$PUBLIC_TARGET-objdump" -d "$WORK/positive.o" | $BB grep -q syscall

      $BB printf '%s\n' 'int main(' > "$WORK/rejected.c"
      $BB rm -f "$WORK/rejected.o"
      if "$out/bin/$PUBLIC_TARGET-gcc" -c "$WORK/rejected.c" -o "$WORK/rejected.o"; then
        echo 'ERROR: normalized provider accepted malformed C' >&2
        exit 1
      fi
      test ! -e "$WORK/rejected.o"
      $BB printf '%s\n' 'template <typename T> int main(' > "$WORK/rejected.cpp"
      $BB rm -f "$WORK/rejected-cxx.o"
      if "$out/bin/$PUBLIC_TARGET-g++" -c "$WORK/rejected.cpp" -o "$WORK/rejected-cxx.o"; then
        echo 'ERROR: normalized provider accepted malformed C++' >&2
        exit 1
      fi
      test ! -e "$WORK/rejected-cxx.o"
      $BB printf '%s\n' '.mantle-invalid-directive' > "$WORK/rejected.s"
      $BB rm -f "$WORK/rejected-asm.o"
      if "$out/bin/$PUBLIC_TARGET-as" -o "$WORK/rejected-asm.o" "$WORK/rejected.s"; then
        echo 'ERROR: normalized provider accepted malformed assembly' >&2
        exit 1
      fi
      test ! -e "$WORK/rejected-asm.o"

      $BB mkdir -p "$out/share/crunch-bootstrap"
      $BB cat > "$out/share/crunch-bootstrap/provider.json" <<EOF
{
  "schema": "mantle-full-source-seed-provider-v1",
  "provider_id": "full-source-v1",
  "name": "$PROVIDER_NAME",
  "target": "$PUBLIC_TARGET",
  "compiler_target": "$GCC_TARGET",
  "dynamic_linker": "$DYNAMIC_LINKER",
  "source_authority": "declared-mantle-derivation-closure",
  "source_outputs": {
    "gcc": "$GCC",
    "musl": "$MUSL",
    "binutils": "$BINUTILS"
  },
  "output_identity": {
    "logical_store_path": "$out",
    "content_blake3": "external-admission-report"
  },
  "runtime_admission": {
    "status": "passed-in-builder",
    "surfaces": ["c-static", "c-dynamic", "cxx-static", "cxx-dynamic", "assembly-link", "archive-index", "object-inspection", "malformed-c", "malformed-cxx", "malformed-assembly"]
  },
  "closure_admission": {
    "status": "derivational-closure-passed",
    "state_pinned_inputs": false,
    "legacy_members": [],
    "release_generated_sources": []
  },
  "non_claims": ["compiler-correctness", "bootstrap-seed-correctness", "independent-rebuild-agreement", "release-reproducibility", "deployment-success", "full-cargo-compatibility"]
}
EOF
      test -s "$out/share/crunch-bootstrap/provider.json"
      echo "full-source provider normalization and runtime admission passed; external BLAKE3/closure admission still required"
    "%
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
    mode = "source-built",
    summary = "Runtime-admitted final GCC 10.5.0, musl 1.2.5, and binutils 2.41 normalized provider candidate",
    retained_tools = provider_retained_tools,
    dropped_components = [
      "fortran",
      "go",
      "ada",
      "d",
      "gdb",
      "gold",
      "gprofng",
    ],
    notes = [
      "Construction remains separate from bootstrap/seed.ncl selection until external BLAKE3 and closure admission pass.",
      "The runtime builder validates C, C++, assembly/link, archive, object inspection, static/dynamic libc, shared libgcc, malformed input, and compiler internals.",
      "Provider claims remain bounded by the explicit non_claims in provider.json.",
    ],
  },
}