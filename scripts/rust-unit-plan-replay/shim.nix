# Linker-name adapter for mantle.offlineCargoPackage on the gnu target: the lane hard-codes
# `linker = "x86_64-linux-musl-gcc"` from its seed toolchain; this exposes the same pinned
# gcc-wrapper cc the unit-plan lane uses under that name (store reference retained).
let
  bb = builtins.storePath /nix/store/h75l4bwd606k94l22ajqdb98vfar5x66-busybox-static-x86_64-unknown-linux-musl-1.37.0;
  cc = builtins.storePath /nix/store/hb2bs5fg5wkm04x565737qd5nh2hy5nk-gcc-wrapper-15.2.0;
in derivation {
  name = "gnu-linker-shim";
  system = "x86_64-linux";
  builder = "${bb}/bin/busybox";
  args = [ "sh" "-c" "${bb}/bin/busybox mkdir -p $out/bin && ${bb}/bin/busybox ln -s ${cc}/bin/cc $out/bin/x86_64-linux-musl-gcc" ];
}
