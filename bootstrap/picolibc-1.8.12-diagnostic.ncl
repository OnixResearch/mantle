# Research-only Picolibc x86_64 Linux static diagnostic.
#
# This derivation builds the pinned Picolibc 1.8.12 x86_64-linux-gnu static
# profile with the upstream Meson cross file. It is a diagnostic for the
# `evaluate-picolibc-stagex-runtime` comparison only. It is not part of the
# StageX lineage, the normalized provider graph, or any parity row. Meson and
# Ninja roles stay diagnostic-only.
#
# Build with the same --nix-compat store and state that pulled the toolchain:
#
#   mantle --nix-compat --store <store> --state-dir <state> \
#     build bootstrap/picolibc-1.8.12-diagnostic.ncl
#
# The build records compiled sources, generated files, tool roles, output
# bytes, and BLAKE3 identities in $out/share/picolibc-diagnostic/manifest.txt.

let crunch = import "../lib/lib.ncl" in
let src = import "picolibc-1.8.12-src.ncl" in
let tc = import "picolibc-diagnostic-tools.ncl" in

{
  picolibc-diagnostic = {
    name = "picolibc-1.8.12-x86_64-linux-diagnostic",
    builder = "/bin/sh",
    system = 'x86_64-linux,
    args = [
      "-c",
      m%"
        set -eu
        BB=/bin/busybox

        SRC=$(echo $NIX_STORE/*-picolibc-1.8.12-src)
        $BB mkdir -p work shims
        $BB cp -r "$SRC" work/src
        $BB chmod -R u+w work/src

        # The upstream cross file names GNU-prefixed tools. Bind the declared
        # gcc and binutils inputs under those names without editing upstream.
        # collect2 also needs plain `ld` and friends on PATH.
        $BB mkdir -p tools
        for tool in gcc g++; do
          $BB ln -s %{tc.gcc}/bin/$tool shims/x86_64-linux-gnu-$tool
        done
        for tool in ar as nm strip objcopy ld ranlib; do
          $BB ln -s %{tc.binutils}/bin/$tool shims/x86_64-linux-gnu-$tool
          $BB ln -s %{tc.binutils}/bin/$tool tools/$tool
        done
        # Ninja spawns commands with execvp("sh", ...): bind the sandbox shell.
        $BB ln -s /bin/sh tools/sh
        # Meson bakes helper-program paths into build.ninja; give it real
        # coreutils so baked paths resolve inside the sandbox.
        export PATH="$PWD/shims:$PWD/tools:%{tc.coreutils}/bin:%{tc.meson}/bin:%{tc.ninja}/bin"
        export HOME=$PWD

        cd work
        meson setup build src \
          --cross-file src/scripts/cross-x86_64-linux-gnu.txt \
          -Dtests=false \
          -Dsemihost=false \
          -Dos-linux=true \
          -Dspecsdir=lib/picolibc \
          --prefix "$PWD/install" > meson-setup.log 2>&1 || {
            $BB cat meson-setup.log >&2
            exit 1
          }
        # Serialized build keeps the log order deterministic so isolated
        # rebuilds produce byte-identical outputs including transcripts.
        ninja -C build -j1 > ninja-build.log 2>&1 || {
          $BB tail -100 ninja-build.log >&2
          exit 1
        }
        ninja -C build install > ninja-install.log 2>&1

        # Reject host-libc dependence in every produced binary object.
        if $BB grep -rl 'GLIBC_' install/lib 2>/dev/null; then
          echo "ERROR: host-libc symbol version reference in diagnostic output" >&2
          exit 1
        fi

        $BB mkdir -p $out/share/picolibc-diagnostic
        $BB cp -r install/* $out/
        $BB cp meson-setup.log ninja-build.log ninja-install.log $out/share/picolibc-diagnostic/

        # Deterministic fact manifest for the comparison core.
        {
          echo "picolibc_version=1.8.12"
          echo "profile=x86_64-linux-gnu-static"
          echo "cross_file=scripts/cross-x86_64-linux-gnu.txt"
          echo "meson=$(%{tc.meson}/bin/meson --version)"
          echo "ninja=$(%{tc.ninja}/bin/ninja --version)"
          echo "gcc=$(x86_64-linux-gnu-gcc -dumpfullversion)"
          echo "binutils_as=$(x86_64-linux-gnu-as --version | $BB head -1)"
          echo "compiled_objects=$($BB find build -name '*.o' | $BB wc -l)"
          echo "compiled_sources:"
          $BB find build -name '*.o' | $BB sed 's|\.o$||' | $BB sort -u
          echo "generated_files:"
          $BB find build -name '*.h' -newer src/meson.build | $BB sort
          echo "output_files:"
          (cd $out && $BB find . -type f | $BB sort)
        } > $out/share/picolibc-diagnostic/manifest.txt

        (cd $out && $BB find . -type f -exec $BB sha256sum {} + | $BB sort -k2) \
          > $out/share/picolibc-diagnostic/output-sha256-manifest.txt
      "%,
    ],
    outputs = ["out"],
    env = {},
    inputs = [
      src.picolibc-src,
      tc.meson,
      tc.ninja,
      tc.gcc,
      tc.binutils,
      tc.coreutils,
    ],
    addressing_mode = 'input-addressed,
    sandbox = 'native,
  } | crunch.Derivation,
}
