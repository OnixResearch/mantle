# Clone a git repository at a specific revision.
#
# Usage: crunch build examples/fetch-git.ncl
#
# Clones the repo, checks out the specified rev, strips .git/,
# and stores the working tree. Hash is the NAR of the checkout.
#
# Requires git in PATH.
#
# To get the hash for a new rev:
#   1. Use a dummy hash and run with --fix
#   2. Or: nix-hash --type sha256 --base32 --sri <path>

let crunch = import "lib.ncl" in

crunch.fetchGit {
  url = "https://github.com/nickel-lang/nickel.git",
  rev = "6f2c7ab1e3e4e37a0e11e5eb82deba56cb4e23fe",
  hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
  name = "nickel-src",
}
