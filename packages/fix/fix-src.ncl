# Pin the psyclyx/fix Nix evaluator source tree.
#
# `fix` is a from-scratch parallel evaluator for the Nix language, written in
# Zig. Mantle uses it as a producer frontend: it evaluates Nix expressions and
# instantiates .drv closures that feed the existing foreign-derivation-import
# pipeline. This file pins only the source. It makes no claim about evaluator
# correctness beyond the recorded, pin-bound differential evidence.
#
# Upstream: https://github.com/psyclyx/fix
# Pinned rev: fd675c2e938da6c9f444a00893b6716d66c89927 (0.3.0, 2026-08-03)
#
# To update the pin: set the new rev, use a dummy hash, run
# `mantle build packages/fix/fix-src.ncl --fix`, and commit the result.

let crunch = import "../../lib/lib.ncl" in

{
  fix-src = crunch.fetchGit {
    url = "https://github.com/psyclyx/fix.git",
    rev = "fd675c2e938da6c9f444a00893b6716d66c89927",
    hash = "sha256-qLTYqSdaxPNMsI59PVgOCMVZjBn655xujFeraqxyK/M=",
    name = "fix-0.3.0-src",
  },
}
