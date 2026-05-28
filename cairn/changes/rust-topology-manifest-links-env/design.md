## Design

Keep the change in the pure native package-env core. `native_cargo_package_env(...)` already owns bounded Cargo-style package metadata for build scripts. Add one key, `CARGO_MANIFEST_LINKS`, populated from `NativeManifestPackage.links` or `""` when absent.

No shell behavior changes. Build-script execution already appends the package env into child process env; adding the key lets linked packages such as `ring` observe the Cargo-compatible value without opening a broader ambient environment surface.
