Task-ID: V3
Covers: bootstrap.part.tinycc.0.9.26.selfcompile

# Produced compiler smoke test

Command:

```sh
OUT="$PWD/target/fix-tinycc-mes-bufferedfile-codegen/run-current-nofuse/store/miymhdmqink0c81drn4y1f1chc73vdf9-tinycc-0.9.26"
LOGICAL="/crunch/store/miymhdmqink0c81drn4y1f1chc73vdf9-tinycc-0.9.26"
BB=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox
BWRAP=/nix/store/dk9qhjgg469lv6mriys7v4c59igarmvx-bubblewrap-0.11.1/bin/bwrap

test -x "$OUT/bin/tcc"
test -x "$OUT/bin/tcc-0.9.26"
test -s "$OUT/lib/mes/libc.a"
test -s "$OUT/lib/mes/tcc/libtcc1.a"

"$BWRAP" --unshare-user --uid 1000 --gid 100 --dev /dev --proc /proc \
  --tmpfs /tmp --dir /crunch --dir /crunch/store \
  --ro-bind "$BB" /bin/sh --ro-bind "$BB" /bin/busybox \
  --ro-bind "$OUT" "$LOGICAL" --chdir /tmp /bin/sh -c '
set -eu
/crunch/store/miymhdmqink0c81drn4y1f1chc73vdf9-tinycc-0.9.26/bin/tcc -version
/crunch/store/miymhdmqink0c81drn4y1f1chc73vdf9-tinycc-0.9.26/bin/tcc-0.9.26 -version
/bin/busybox cat > hello.c <<EOF
int main(){return 0;}
EOF
/crunch/store/miymhdmqink0c81drn4y1f1chc73vdf9-tinycc-0.9.26/bin/tcc -static -o hello hello.c
./hello
/crunch/store/miymhdmqink0c81drn4y1f1chc73vdf9-tinycc-0.9.26/bin/tcc-0.9.26 -static -o hello2 hello.c
./hello2
echo V3-smoke-pass
'
```

Result: PASS.

Transcript:

```text
tcc version 0.9.26 (x86_64 Linux)
tcc version 0.9.26 (x86_64 Linux)
V3-smoke-pass
```

This smoke test uses produced `bin/tcc` and `bin/tcc-0.9.26`; `tcc-mes -version` alone is not used as acceptance evidence.
