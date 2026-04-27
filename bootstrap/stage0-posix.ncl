# Build stage0-posix: hex0 → mescc-tools + M2-Planet.
#
# Starts from only the hex0 seed binary (229 bytes) and kaem-optional-seed
# (618 bytes). Everything else is built from source inside the sandbox.
#
# Output: mescc-tools (hex0, hex1, hex2, M1, M0, catm, blood-elf, kaem),
#         M2-Planet, and mescc-tools-extra (cp, chmod, mkdir, untar, ungz,
#         unbz2, unxz, sha256sum).
#
# Adapted from stage0-posix Release_1.9.1.

let crunch = import "lib.ncl" in

# Source repositories (submodules of stage0-posix).
# Each pinned at the Release_1.9.1 commit.

let amd64_src = crunch.fetchGit {
  url = "https://github.com/oriansj/stage0-posix-amd64.git",
  rev = "82efa0d6be1c9bb993a7a62af1cccd8d2cda91f6",
  hash = "sha256-Yz9YjbGZuBPOZxiSnbflq225R+0engR+4f8FYL28zEw=",
  name = "stage0-posix-amd64",
} in

let m2planet_src = crunch.fetchGit {
  url = "https://github.com/oriansj/M2-Planet.git",
  rev = "bd2fe4b0659fd0ad3f476a5ad0ef801bd134665d",
  hash = "sha256-QEehlCukfqJXgd7P7HlXOq+9MWxraFiUey6B94bZ5pM=",
  name = "M2-Planet",
} in

let m2libc_src = crunch.fetchGit {
  url = "https://github.com/oriansj/M2libc.git",
  rev = "68a23cfd05d5a355ba7a30c770d684cbe86fcc4e",
  hash = "sha256-2x/v/snbRmwY68kSdeLDa7q8obhMHl7Z245WQUHnAEY=",
  name = "M2libc",
} in

let mescctools_src = crunch.fetchGit {
  url = "https://git.savannah.nongnu.org/git/mescc-tools.git",
  rev = "5adfbf3364261a77109878a56b100aeeb6ef9ac4",
  hash = "sha256-hUmxc6Njfg++YU8+AHv7eLjxNPKo7NzoL1Ox0GsUQHY=",
  name = "mescc-tools",
} in

let mescctools_extra_src = crunch.fetchGit {
  url = "https://github.com/oriansj/mescc-tools-extra.git",
  rev = "a151c245e512076971a3c85bb1502cf92cfa83b6",
  hash = "sha256-0C9QUYsJyYu/oEL+zEkkPjq1SpmBYYnal1Z40+TdMhQ=",
  name = "mescc-tools-extra",
} in

let m2mesoplanet_src = crunch.fetchGit {
  url = "https://github.com/oriansj/M2-Mesoplanet.git",
  rev = "4b011a85da73a7c97212468d41f17e806ba99547",
  hash = "sha256-ti/4V3naZT7kIR+ioqfQpcD1qZyxe8VzFNm7Bg+npzQ=",
  name = "M2-Mesoplanet",
} in

let bootstrap_seeds_src = crunch.fetchGit {
  url = "https://github.com/oriansj/bootstrap-seeds.git",
  rev = "cedec6b8066d1db229b6c77d42d120a23c6980ed",
  hash = "sha256-0RVjc5eTPD2AXFdQ4/rKyeiGrll7Fj62NY5RISvGNSg=",
  name = "bootstrap-seeds",
} in

{
  name = "stage0-posix",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox

      # Find our input store paths.
      find_input() {
        for d in $NIX_STORE/*-"$1"; do
          if [ -d "$d" ]; then echo "$d"; return; fi
        done
        echo "ERROR: input $1 not found" >&2
        exit 1
      }

      AMD64_SRC=$(find_input stage0-posix-amd64)
      M2PLANET_SRC=$(find_input M2-Planet)
      M2LIBC_SRC=$(find_input M2libc)
      MESCCTOOLS_SRC=$(find_input mescc-tools)
      MESCCTOOLS_EXTRA_SRC=$(find_input mescc-tools-extra)
      M2MESOPLANET_SRC=$(find_input M2-Mesoplanet)
      BOOTSTRAP_SEEDS_SRC=$(find_input bootstrap-seeds)

      # Set up the stage0-posix source tree layout.
      WORK=/tmp/stage0-posix
      $BB mkdir -p "$WORK"
      cd "$WORK"

      # Copy sources into the expected directory layout. The kaem scripts
      # use relative paths (./AMD64/..., ./M2-Planet/..., etc.).
      $BB cp -r "$AMD64_SRC" AMD64
      $BB cp -r "$M2PLANET_SRC" M2-Planet
      $BB cp -r "$M2LIBC_SRC" M2libc
      $BB cp -r "$MESCCTOOLS_SRC" mescc-tools
      $BB cp -r "$MESCCTOOLS_EXTRA_SRC" mescc-tools-extra
      $BB cp -r "$M2MESOPLANET_SRC" M2-Mesoplanet
      $BB cp -r "$BOOTSTRAP_SEEDS_SRC" bootstrap-seeds

      # Make everything writable (build writes artifacts in-tree).
      $BB chmod -R u+w .

      # Create the top-level files that kaem.amd64 references.
      # These are from the main stage0-posix repo.
      $BB cat > amd64.answers << 'ANSWERS'
6d1c534225ac7a3e74e29cb17579f5dd720547c19e7eea6bfe742326fb4a0bc7  AMD64/bin/blood-elf
c13daf05065008a1e7bc4852fcce016df24edb1c633961f462421b8b0ab176ca  AMD64/bin/catm
55e78986bd1a5641f6a665ecb8be51aff06ce805b700b246821a001e9afc60e8  AMD64/bin/chmod
aa8eb71e83570f84b3fe13c08f28e3861e59681b178e38b6fd8f0a4cafaebc27  AMD64/bin/cp
379cb35a87bd7a9c190d79d3141af897869aae5e4cb0ac9e03235c78b84e1be7  AMD64/bin/get_machine
f782df6d59f7ff930e67f34dde7ea62288b12d6d1a0b036ef465ad06b4a4b97a  AMD64/bin/hex2
6364de59b36f8575fd938779476a9a121878fc0e2d65cc01ae5372aad2d028b5  AMD64/bin/kaem
fc68c115384852b827a35b607de382592a475c324e1e11ca7cf5b9e840ce15a9  AMD64/bin/M1
f24855886c8a7e63015b8c8b6aeb4a03556540be16baab62d234deb4c57625b4  AMD64/bin/M2-Mesoplanet
7cf19de29a4ae63637898f6093d0526b73108040ab3634f11232030439602fc1  AMD64/bin/M2-Planet
962d44ad6cc75ee1f51fbb0d07c5e4fdf9479540d46ceae26d1bb34558feaa2f  AMD64/bin/match
c8bdd424b1e588714c654de9e1df3de690301cf2ff883e93da1f01b86806cc20  AMD64/bin/mkdir
88bfafe232428196a056b1ef517d4f797c68f0eaabc3dce84f66194899dbf864  AMD64/bin/replace
c7ac6f9ca26a0b771c130b43c8b536b300445f696e04b5db4f0f4eb949613459  AMD64/bin/rm
f7c0fcae688ec01050d836771392f4d81f5ad6788590278ec76ccd17cdd89e48  AMD64/bin/sha256sum
0769b2c0af733b62db727a73c04cbdadafa747ed15c0ade42c6c613c18e165ec  AMD64/bin/ungz
2e2ff2ae38b2d4e9ca2268b3265b0d82b79a931ce25aa91daf36c006537f2d45  AMD64/bin/unbz2
1960964e6d73cdc3b55f2bb3b2f2c33668025a802d86cef48f39bdaa9a5ea201  AMD64/bin/unxz
8100ef93f3f078fb825d97eb53773f9f3d6e420d5bd40f8b0e3285e405544e9c  AMD64/bin/untar
ANSWERS
      $BB cat > after.kaem << 'AFTERKAEM'
echo "stage0-posix build complete"
AFTERKAEM

      # Create artifact directory.
      $BB mkdir -p AMD64/artifact AMD64/bin

      #################################################################
      # Phase 0: hex0-seed builds hex0 from source                    #
      #################################################################
      # kaem-optional-seed runs mescc-tools-seed-kaem.kaem which:
      # 1. hex0-seed assembles hex0 from hex0_AMD64.hex0
      # 2. hex0 assembles kaem-0 from kaem-minimal.hex0
      $BB chmod +x bootstrap-seeds/POSIX/AMD64/hex0-seed
      $BB chmod +x bootstrap-seeds/POSIX/AMD64/kaem-optional-seed
      bootstrap-seeds/POSIX/AMD64/kaem-optional-seed AMD64/mescc-tools-seed-kaem.kaem

      #################################################################
      # Phase 1-11: kaem-0 builds the full mescc-tools chain          #
      #################################################################
      # kaem-0 runs mescc-tools-mini-kaem.kaem which builds:
      # hex1, hex2, M1, M0, catm, blood-elf, kaem (full)
      AMD64/artifact/kaem-0 AMD64/mescc-tools-mini-kaem.kaem

      #################################################################
      # Phase 12-23: full kaem rebuilds M2-Planet + mescc-tools-extra  #
      #################################################################
      AMD64/bin/kaem --verbose --strict --file AMD64/kaem.run

      #################################################################
      # Install outputs                                                #
      #################################################################
      $BB mkdir -p "$out/bin" "$out/lib"

      # hex0 lives in AMD64/artifact/, not AMD64/bin/
      $BB cp "AMD64/artifact/hex0" "$out/bin/hex0"

      # mescc-tools binaries (built into AMD64/bin/ by the kaem chain)
      for tool in hex1 hex2 M1 M0 catm blood-elf kaem get_machine; do
        if [ -x "AMD64/bin/$tool" ]; then
          $BB cp "AMD64/bin/$tool" "$out/bin/$tool"
        fi
      done

      # M2-Planet
      if [ -x "AMD64/bin/M2-Planet" ]; then
        $BB cp "AMD64/bin/M2-Planet" "$out/bin/M2-Planet"
      fi

      # mescc-tools-extra
      for tool in cp chmod mkdir untar ungz unbz2 unxz sha256sum match replace wrap; do
        if [ -x "AMD64/bin/$tool" ]; then
          $BB cp "AMD64/bin/$tool" "$out/bin/$tool"
        fi
      done

      # M2libc for later stages
      $BB cp -r M2libc "$out/lib/M2libc"

      # Verify critical outputs exist
      test -x "$out/bin/hex0"
      test -x "$out/bin/kaem"
      test -x "$out/bin/M2-Planet"
      test -x "$out/bin/M1"
      test -x "$out/bin/hex2"
      test -x "$out/bin/blood-elf"

      $BB chmod -R 0555 "$out/bin"
    "%,
  ],
  inputs = [
    amd64_src,
    m2planet_src,
    m2libc_src,
    mescctools_src,
    mescctools_extra_src,
    m2mesoplanet_src,
    bootstrap_seeds_src,
  ],
} | crunch.Derivation
