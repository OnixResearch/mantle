# Install bounded Linux UAPI headers from a pinned kernel source release.
#
# bwrap and BusyBox consume raw kernel interfaces that are intentionally not
# part of musl's libc headers. Keep that source boundary explicit instead of
# inheriting headers from the retired binary seed provider.

let crunch = import "lib.ncl" in
let seed = import "seed.ncl" in
let toolchain = seed.toolchain in
let seed_name = seed.name in
let seed_target = seed.target in
let make = crunch.derivationFile "make.ncl" in
let linux_src = crunch.derivationFile "linux-6.6-source.ncl" in

{
  name = "linux-headers-6.6",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      WORK=/tmp/linux-headers-6.6
      TOOLBIN="$WORK/tools"

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

      SRC=$(find_input linux-6.6-src)
      MAKE=$(find_input gnumake)
      TOOLCHAIN=$(find_input %{seed_name})
      SOURCE_ROOT="$SRC"
      if [ -s "$SRC/linux-6.6/Makefile" ]; then
        SOURCE_ROOT="$SRC/linux-6.6"
      fi
      test -s "$SOURCE_ROOT/Makefile"

      $BB rm -rf "$WORK"
      $BB mkdir -p "$TOOLBIN"
      for applet in awk basename cat chmod cmp cp cut dirname echo env expr false find grep head install ln ls mkdir mv printf pwd readlink rm sed sh sort tail test touch tr true uname uniq wc xargs; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done
      $BB cat > "$TOOLBIN/rsync" <<'EOF'
#!/bin/sh
set -eu
POSITIONAL_COUNT_REQUIRED=2
position_count=0
source_path=
destination_path=
for argument in "$@"; do
  case "$argument" in
    -*) continue ;;
  esac
  position_count=$((position_count + 1))
  if [ "$position_count" -eq 1 ]; then
    source_path=$argument
  elif [ "$position_count" -eq "$POSITIONAL_COUNT_REQUIRED" ]; then
    destination_path=$argument
  else
    echo "ERROR: bounded headers rsync received extra positional argument" >&2
    exit 1
  fi
done
[ "$position_count" -eq "$POSITIONAL_COUNT_REQUIRED" ]
[ "$source_path" = usr/include ]
[ -n "$destination_path" ]
/bin/busybox mkdir -p "$destination_path"
/bin/busybox cp -R "$source_path" "$destination_path"
EOF
      $BB chmod 755 "$TOOLBIN/rsync"
      export PATH="$MAKE/bin:$TOOLCHAIN/bin:$TOOLBIN"
      export HOSTCC="$TOOLCHAIN/bin/%{seed_target}-gcc"
      export HOSTCXX="$TOOLCHAIN/bin/%{seed_target}-g++"

      $BB cp -R "$SOURCE_ROOT" "$WORK/source"
      $BB chmod -R u+w "$WORK/source"
      cd "$WORK/source"

      "$MAKE/bin/make" -j1 ARCH=x86 HOSTCC="$HOSTCC" HOSTCXX="$HOSTCXX" \
        INSTALL_HDR_PATH="$out" headers_install

      test -s "$out/include/linux/capability.h"
      test -s "$out/include/linux/loop.h"
      test -s "$out/include/linux/netlink.h"
      test -s "$out/include/linux/rtnetlink.h"
      ! grep -R -n '/nix/store\|/mantle/store\|/crunch/store' "$out/include"
      echo "Linux 6.6 UAPI header installation passed"
    "%
  ],
  inputs = [toolchain, make, linux_src],
} | crunch.Derivation
