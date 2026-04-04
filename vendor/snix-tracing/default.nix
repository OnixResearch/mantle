{ depot, lib, ... }:

(depot.snix.crates.workspaceMembers.snix-tracing.build.override {
  runTests = true;
}).overrideAttrs
  (old: rec {
    meta.ci.targets = lib.filter (x: lib.hasPrefix "with-features" x || x == "no-features") (
      lib.attrNames passthru
    );
    passthru =
      old.passthru
      // (depot.snix.utils.mkFeaturePowerset {
        inherit (old) crateName;
        features = [
          "axum"
          "clap"
          "otlp"
          "tracy"
          "tonic"
          "reqwest"
        ];
      });
  })
