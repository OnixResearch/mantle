# Spike proof: native i386 static ELF execution in Crunch's sandbox.
#
# This is intentionally small: it proves the selected spike execution model
# (i386 userspace under amd64 Linux/bwrap) before attempting the full
# tcc-0.9.27 -> make-3.82 i386 chain.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in

{
  name = "spike-i386-native-smoke",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox

      WORK=/tmp/spike-i386-native-smoke
      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK"
      cd "$WORK"

      # 32-bit little-endian i386 ET_EXEC. Program body is:
      #   mov eax, 1      ; sys_exit
      #   mov ebx, 42
      #   int 0x80
      $BB printf '\177\105\114\106\001\001\001\000\000\000\000\000\000\000\000\000\002\000\003\000\001\000\000\000\124\200\004\010\064\000\000\000\000\000\000\000\000\000\000\000\064\000\040\000\001\000\000\000\000\000\000\000\001\000\000\000\000\000\000\000\000\200\004\010\000\200\004\010\140\000\000\000\140\000\000\000\005\000\000\000\000\020\000\000\270\001\000\000\000\273\052\000\000\000\315\200' > i386-exit42
      $BB chmod 755 i386-exit42
      $BB ls -l i386-exit42

      set +e
      ./i386-exit42
      rc=$?
      set -e
      echo "i386-exit42 rc=$rc"
      test "$rc" = 42

      $BB mkdir -p "$out/bin" "$out/share/spike-i386-native-smoke"
      $BB cp i386-exit42 "$out/bin/i386-exit42"
      $BB printf 'native i386 static ELF executed successfully with rc=42\n' > "$out/share/spike-i386-native-smoke/result.txt"
      $BB chmod 755 "$out/bin/i386-exit42"
      test -x "$out/bin/i386-exit42"
    "%,
  ],
  inputs = [stage0],
} | crunch.Derivation
