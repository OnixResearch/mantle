# Diagnostic-only toolchain inputs for the Picolibc StageX comparison.
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
#
# Trust posture: binary trust inputs signed by the cache.nixos.org signer set.
# Meson and Ninja observations stay diagnostic-only and MUST NOT satisfy
# protected StageX execution evidence.

{
  meson = "/nix/store/7hhzs4aal2v8ydl7fnads45fcblc36xx-meson-1.10.2",
  ninja = "/nix/store/vqnp9w33c613xl7ism88wx69rqaabamn-ninja-1.13.2",
  gcc = "/nix/store/qrn789mrv652j7f5bcjp367npzpcyvdi-gcc-15.3.0",
  binutils = "/nix/store/advlbxfc5cg0jswl7zv40h2lzlxiksr6-binutils-2.46",
  coreutils = "/nix/store/di26b1kkbammy0sj70nq5qzvfrh78wxl-coreutils-9.11",
}
