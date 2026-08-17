# rust-topology-custom-build-alias-binding

## Why

After proc-macro target-name normalization, native topology reaches `aws-lc-sys@0.39.1` and fails before a JSON topology receipt. The target lib unit carries a same-package build-script dependency named `build_script_main`, while the native host artifact is normalized as `build_script_build`. Host artifact binding only rewrites exact crate-name matches, so the placeholder remains and execution looks for a same-package target artifact that will never be produced.

## Change

Teach same-package custom-build host artifact binding to recognize Cargo's build-script extern aliases (`build_script_main` and `build_script_build`) as names for the selected build-script host artifact. Keep proc-macro binding unchanged.

## Success

- Focused tests prove custom-build host artifact placeholders with `build_script_main` are rewritten to the produced host executable.
- Clean self-probe advances beyond the `aws-lc-sys` self-dependency internal error or records the next frontier.
