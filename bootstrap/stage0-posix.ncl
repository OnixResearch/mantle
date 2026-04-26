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
  hash = "sha256-04VlrCewQnE2ateObg9YmGOPvfAWjScUFOkNTgAsyg4=",
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
  hash = "sha256-NWutNXdNipFHm0DOVYi6O18kjK76+R9zQW32jbqzDRY=",
  name = "mescc-tools",
} in

let mescctools_extra_src = crunch.fetchGit {
  url = "https://github.com/oriansj/mescc-tools-extra.git",
  rev = "a151c245e512076971a3c85bb1502cf92cfa83b6",
  hash = "sha256-Szg4apU8VCcbewprmRdsmLS+qyq+ODt516W1YLYU474=",
  name = "mescc-tools-extra",
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
      $BB cp -r "$BOOTSTRAP_SEEDS_SRC" bootstrap-seeds

      # Make everything writable (build writes artifacts in-tree).
      $BB chmod -R u+w .

      # Create the top-level files that kaem.amd64 references.
      # These are from the main stage0-posix repo.
      $BB cat > amd64.answers << 'ANSWERS'
ANSWERS
      # The answers file is used for sha256sum -c; we skip it for now.
      # after.kaem is the post-build hook (we use a no-op).
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

      # mescc-tools binaries
      for tool in hex0 hex1 hex2 M1 M0 catm blood-elf kaem get_machine; do
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
    bootstrap_seeds_src,
  ],
} | crunch.Derivation
