# crunch stdlib: seed toolchain template.
#
# Copy this file and fill in store paths from your Nix installation.
# Generate paths with: nix-build '<nixpkgs>' -A <pkg> --no-out-link
#
# The StorePath contract catches typos and malformed paths at eval time.

let { StorePath, .. } = import "contracts.ncl" in

{
  bash | StorePath
    | doc "Path to bash. Generate: nix-build '<nixpkgs>' -A bash --no-out-link",

  coreutils | StorePath
    | doc "Path to coreutils. Generate: nix-build '<nixpkgs>' -A coreutils --no-out-link",

  gcc | StorePath
    | doc "Path to gcc wrapper. Generate: nix-build '<nixpkgs>' -A gcc --no-out-link",

  gnumake | StorePath
    | doc "Path to GNU Make. Generate: nix-build '<nixpkgs>' -A gnumake --no-out-link",

  binutils | StorePath
    | doc "Path to binutils wrapper. Generate: nix-build '<nixpkgs>' -A binutils --no-out-link",
}
