{ depot, ... }:

depot.snix.crates.workspaceMembers.nix-compat-derive.build.override {
  runTests = true;
}
