# Clone a git repository at a specific revision.
#
# Usage: mantle build examples/fetch-git.ncl
#
# Clones the repo, checks out the specified rev, strips .git/,
# and stores the working tree. Hash is the NAR of the checkout.
#
# Does not require host git in PATH.
#
# To get the hash for a new rev:
#   1. Use a dummy hash and run with --fix
#   2. Or: nix-hash --type sha256 --base32 --sri <path>

let mantle = import "lib.ncl" in

mantle.fetchGit {
  url = "https://github.com/andir/npins.git",
  rev = "refs/tags/0.3.0",
  hash = "sha256-nTm6IqCHNFQLU7WR7dJRP7ktBctpE/O2LHbUV25roJA=",
  name = "npins-src",
}
