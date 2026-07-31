# Linux UAPI headers for the full-source Rust host-tool closure.
#
# The authenticated kernel source is normalized with the source-built GNU Make
# and admitted native provider rather than the compatibility bootstrap chain.

let crunch = import "lib.ncl" in
let native_provider = crunch.derivationFile "seed-full-toolchain.ncl" in
let gnumake = crunch.derivationFile "make-4.4.1-gcc10.ncl" in
let linux_src = crunch.derivationFile "linux-6.6-source.ncl" in

{
  name = "linux-headers-6.6-full-source-gcc10-v3",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      WORK=/tmp/linux-headers-6.6-full-source
      TARGET=x86_64-linux-musl

      find_input() {
        input_suffix=$1
        matched_input=
        match_count=0
        for input_path in $NIX_STORE/*-"$input_suffix"; do
          if [ -e "$input_path" ]; then
            matched_input=$input_path
            match_count=$((match_count + 1))
          fi
        done
        if [ "$match_count" -ne 1 ]; then
          echo "ERROR: expected exactly one declared input $input_suffix, got $match_count" >&2
          exit 1
        fi
        echo "$matched_input"
      }

      PROVIDER=$(find_input full-source-seed-toolchain)
      MAKE=$(find_input make-4.4.1-full-source-gcc10-v1)
      SRC=$(find_input linux-6.6-src)
      SOURCE_ROOT="$SRC"
      if [ -s "$SRC/linux-6.6/Makefile" ]; then SOURCE_ROOT="$SRC/linux-6.6"; fi
      test -x "$PROVIDER/bin/$TARGET-gcc"
      test -x "$MAKE/bin/make"
      test -s "$SOURCE_ROOT/Makefile"

      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK/tools"
      for applet in awk basename cat chmod cmp cp cut dirname echo env expr false find grep head install ln ls mkdir mv printf pwd readlink rm sed sh sort tail test touch tr true uname uniq wc xargs; do
        $BB ln -sf "$BB" "$WORK/tools/$applet"
      done
      $BB cat > "$WORK/tools/rsync" <<'EOF'
#!/bin/sh
set -eu
position_count=0
source_path=
destination_path=
for argument in "$@"; do
  case "$argument" in -*) continue ;; esac
  position_count=$((position_count + 1))
  if [ "$position_count" -eq 1 ]; then
    source_path=$argument
  elif [ "$position_count" -eq 2 ]; then
    destination_path=$argument
  else
    echo "ERROR: bounded headers rsync received extra positional argument" >&2
    exit 1
  fi
done
[ "$position_count" -eq 2 ]
[ "$source_path" = usr/include ]
[ -n "$destination_path" ]
/bin/busybox mkdir -p "$destination_path"
/bin/busybox cp -R "$source_path" "$destination_path"
EOF
      $BB chmod 755 "$WORK/tools/rsync"
      export PATH="$WORK/tools:$MAKE/bin:$PROVIDER/bin"
      export HOSTCC="$PROVIDER/bin/$TARGET-gcc"
      export HOSTCXX="$PROVIDER/bin/$TARGET-g++"
      export SOURCE_DATE_EPOCH=1

      $BB cp -R "$SOURCE_ROOT" "$WORK/source"
      $BB chmod -R u+w "$WORK/source"
      cd "$WORK/source"
      "$MAKE/bin/make" -j1 ARCH=x86 HOSTCC="$HOSTCC" HOSTCXX="$HOSTCXX" \
        INSTALL_HDR_PATH="$out" headers_install

      test -s "$out/include/linux/capability.h"
      test -s "$out/include/linux/loop.h"
      test -s "$out/include/linux/netlink.h"
      test -s "$out/include/linux/rtnetlink.h"
      ! "$BB" grep -R -n '/nix/store\|/mantle/store\|/crunch/store' "$out/include"

      $BB mkdir -p "$out/share/mantle-bootstrap"
      $BB cat > "$out/share/mantle-bootstrap/full-source-rust-host-linux-headers.txt" <<'EOF'
claim: bounded source-built Linux UAPI header installation passed
positive: required capability, loop, netlink, and rtnetlink headers installed
negative: installed headers contain no Nix, Mantle, or Crunch store references
source_authority: authenticated Linux 6.6 source
native_authority: admitted full-source provider plus independently built GNU Make
non_claims: kernel correctness, Rust provider completion, whole-bootstrap correctness
EOF
      echo "source-built Linux 6.6 UAPI header installation passed"
    "%
  ],
  inputs = [native_provider, gnumake, linux_src],
} | crunch.Derivation
