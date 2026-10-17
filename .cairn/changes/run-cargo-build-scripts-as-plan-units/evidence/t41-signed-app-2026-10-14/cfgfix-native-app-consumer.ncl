let cargo_unit_plan = import "lib/cargo_unit_plan.ncl" in
let mantle = import "lib/lib.ncl" in
let plan_producer = cargo_unit_plan {
  goal_name = "cargo-build-script-units-native-cfgfix-87lq",
  producer = "/nix/store/gnqz2kr75zgs3qgm4731zq81lhhz622s-mantle-cargo-unit-producer-current-87lq",
  config = "/nix/store/1cxzrjd337sm2im67xg9jmdgpyls1j05-mantle-unit-plan-config-cfgfix-87lq",
  workspace = "/nix/store/n9bqvcqx55qqigwkn6vnjrjkl7xv10z9-mantle-unit-plan-fixture-source",
  planner = "/nix/store/6df7id3a8zd7mxn7b4hhc3b6ljgn1fzr-mantle-unit-plan-planner-repaired-87lq",
  toolchain = "/nix/store/bhn71wnpahwj10irc2373w1nqw6353pw-mantle-unit-plan-nightly-nodoc-87lq",
  linker = "/nix/store/hb2bs5fg5wkm04x565737qd5nh2hy5nk-gcc-wrapper-15.2.0",
  helper = "/nix/store/ain8qbb840v775dhz1528lvz0syys7sj-mantle-cargo-unit-helper-cfgfix-87lq",
  vendor_sources = ["/nix/store/mblpg9zxh86ds777vc7zhrl4dpj4057v-mantle-unit-plan-locked-vendor"],
  runtime_inputs = [
    "/nix/store/8kvxvr3pmsypxiypq4g8zy13glnfr7nx-glibc-2.42-67",
    "/nix/store/n51dhmdbik1kfrsm62j5knavmigwrl1a-glibc-2.42-84",
    "/nix/store/ssvq1r0xd8f7paf6zqgpfql1a4drwhy2-xgcc-15.3.0-libgcc",
    "/nix/store/0vqb1mcas5j8dv6bhbrshinlgsg6bvgi-gcc-15.3.0-lib",
    "/nix/store/dbz6pb9g67kpgpl95k8d85kzpxm1c32p-zlib-1.3.2",
    "/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0",
  ],
  native_inputs = {
    "path+file:///build/cargo-unit-plan-workspace/workspace/native-answer-sys#0.1.0" = {
      inputs = { answer = "/nix/store/77giqf2dbsvv1sjcmydx9x366s5614fc-native-answer-0.1" },
      env = { NATIVE_ANSWER_ROOT = "{{role:answer}}" },
      path = [],
    },
  },
} in
{
  producer = plan_producer,
  app = {
    name = "cargo-build-script-units-app-check-cfgfix-87lq",
    builder = "/nix/store/pv1yr8fbzs88rpyc61xycwjlx9dakmp2-mantle-unit-plan-app-check",
    addressing_mode = 'input-addressed,
    sandbox = 'native,
    inputs = [
      "/nix/store/pv1yr8fbzs88rpyc61xycwjlx9dakmp2-mantle-unit-plan-app-check",
      "/nix/store/sfvyavxai6qvzmv9p9x6mp4wwdz4v41m-bash-interactive-5.3p9",
      "/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0",
      {
        name = "app",
        producer = plan_producer,
        plan_output = "plan",
        root = "u.99f32f5ae94f82205ce9acd4a184e1741597862139ddb9246b18f3686f406982",
        unit_output = "out",
      },
    ],
    env = {
      MANTLE_UNIT_APP = "{{mantle-plan-output:app}}",
      PATH = "/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin",
    },
  } | mantle.Derivation,
}
