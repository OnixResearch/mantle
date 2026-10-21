# crunch stdlib: seed toolchain template.
#
# Copy this file and fill in store paths from your Nix installation.
# Generate paths with: nix-build '<nixpkgs>' -A <pkg> --no-out-link
#
# The StorePath contract catches typos and malformed paths at eval time.
#
# Nickel record contracts may carry at most one field without a definition, so
# the template keeps its field documentation in an open contract and states
# the required fields in the shape check.

let { StorePath, Shape, .. } = import "contracts.ncl" in

let SeedContract = {
  bash | StorePath
    | doc "Path to bash. Generate: nix-build '<nixpkgs>' -A bash --no-out-link"
    | optional,

  coreutils | StorePath
    | doc "Path to coreutils. Generate: nix-build '<nixpkgs>' -A coreutils --no-out-link"
    | optional,

  gcc | StorePath
    | doc "Path to gcc wrapper. Generate: nix-build '<nixpkgs>' -A gcc --no-out-link"
    | optional,

  gnumake | StorePath
    | doc "Path to GNU Make. Generate: nix-build '<nixpkgs>' -A gnumake --no-out-link"
    | optional,

  binutils | StorePath
    | doc "Path to binutils wrapper. Generate: nix-build '<nixpkgs>' -A binutils --no-out-link"
    | optional,
  ..
} in
let SeedRequired = {
  bash : StorePath,
  coreutils : StorePath,
  gcc : StorePath,
  gnumake : StorePath,
  binutils : StorePath,
} in

std.contract.Sequence [
  SeedContract,
  Shape [] SeedRequired,
]
