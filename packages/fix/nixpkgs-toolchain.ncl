# nixpkgs toolchain inputs for the fix backend build.
#
# These logical /nix/store paths are admitted through signed cache
# substitution, never read from the ambient host store:
#
#   mantle --nix-compat --store <store> --state-dir <state> store pull \
#     --from https://cache.nixos.org --closure \
#     --trusted-public-keys cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY= \
#     <path>
#
# nixpkgs pin: github:NixOS/nixpkgs/dfd9566f82a6e1d55c30f861879186440614696e
# (narHash sha256-/m3yyS/EnXqoPGBJYVy4jTOsirdgsEZ3JdN2gGkBr14=)
#
# Trust posture: binary trust input signed by the cache.nixos.org signer set.
# These are not source-built compilers or libraries.

{
  zig = "/nix/store/j0yhl5hnz61i7g8plp0nx0hradbqd80a-zig-0.16.0",
  libgit2 = "/nix/store/zfhvy22z7fn800j66xf1mp0k7va03rrq-libgit2-1.9.4",
  libgit2_lib = "/nix/store/y0br2rqsm58691rmyrwj49gskqld5glv-libgit2-1.9.4-lib",
  libgit2_dev = "/nix/store/p9vp6qppska5i42x7084njqmmkxvkc9b-libgit2-1.9.4-dev",
  curl = "/nix/store/9gg2kw74rag1sjl2jl6qaddaisllpdip-curl-8.21.0",
  curl_dev = "/nix/store/d0a0g5q9spi04dpz89n4wqnzv8synh73-curl-8.21.0-dev",
  openssl = "/nix/store/39jzpf9ray7mxjnlmqbrxlyz0dsz0448-openssl-3.6.3",
  openssl_dev = "/nix/store/dbvxz51s7m6401ycyp3l38407y11hq6p-openssl-3.6.3-dev",
  zlib = "/nix/store/fkcbg2c1w29jr5yp9awai9w3v1wvxdk9-zlib-1.3.2",
  zlib_dev = "/nix/store/h4si9iyvqy1aaqy4piqw0pj2c6snmz69-zlib-1.3.2-dev",
  libssh2 = "/nix/store/zzq7pa15jb9bip5yhl7cdjr3wx5xhra2-libssh2-1.11.1",
  pkg_config = "/nix/store/2l3a3fhc1nyp69mlq5ifb4mkb9mryb98-pkg-config-wrapper-0.29.2",
}
