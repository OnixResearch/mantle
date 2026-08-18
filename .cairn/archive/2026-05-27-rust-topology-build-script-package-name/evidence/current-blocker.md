# Current review finding

Same-family review warning:

```text
Build-script CARGO_PKG_NAME is still target-derived, not package-derived.
Evidence: src/rust_plan.rs build_script_child_env sets BUILD_SCRIPT_CARGO_PKG_NAME_ENV to rust_crate_name(&unit.target_name).
Evidence: test build_script_child_env_sets_tool_target_and_manifest_dir expects "build_script_build" for a build script target.
```

Decision: fix before continuing to the proc-macro/dependency frontier because the accepted build-script env spec says the environment is Cargo-like, and Cargo uses manifest package name for `CARGO_PKG_NAME`.
