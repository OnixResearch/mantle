# crunch builders package.
#
# Provides mkStdenv, mkDerivation, mkShell, callPackage, and
# MkDerivationArgs. This is separate from the core stdlib — the
# stdlib ships the schema, this package ships the opinions.
#
# Usage:
#   let crunch = import "lib.ncl" in
#   let builders = import "builders/lib.ncl" in
#   let stdenv = builders.mkStdenv seed in
#   stdenv.mkDerivation { pname = "hello", ... }

let pkg = import "mk_derivation.ncl" in

{
  mkStdenv = pkg.mkStdenv,
  mkShell = pkg.mkShell,
  callPackage = pkg.callPackage,
  MkDerivationArgs = pkg.MkDerivationArgs,
}
