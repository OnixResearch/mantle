# crunch stdlib: Project outputs contract.
#
# Validates the structure of crunch.ncl project root files.
# A project declares packages, dev shells, and checks.
#
# Usage:
#   let crunch = import "lib.ncl" in
#   {
#     packages.hello = stdenv.mkDerivation { ... },
#     devShells.default = builders.mkShell { ... },
#     default.package = "hello",
#   } | crunch.Project

let Derivation = import "derivation.ncl" in

let DerivationSet = { _ : Derivation } in

{
  packages | DerivationSet
    | doc "Buildable packages. `crunch build .#name` resolves here first."
    | default = {},

  devShells | DerivationSet
    | doc "Development shells. `crunch develop` resolves from here."
    | default = {},

  checks | DerivationSet
    | doc "CI checks. `crunch check` builds all of these."
    | default = {},

  default | {
    package | String
      | doc "Name of the default package for bare `crunch build`."
      | optional,
    shell | String
      | doc "Name of the default shell for `crunch develop`."
      | optional,
  }
    | doc "Default targets for commands with no explicit selector."
    | default = {},
}
