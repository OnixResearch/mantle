probe: target/mantle-self-rust-plan-probe-after-c3ccf629-clean/receipt.json
head: c3ccf629a3943d3e5427ca8cc8711579e1f29a21
git_status_short_bytes=0

git_status_short:

probe_status=0
topology_execution=blocked
source_closure=true
native_registry_source_planning=true
native_git_source_planning=true
native_package_target_planning=true
native_unit_graph_planning=true
native_host_unit_graph_planning=true
unit_derivation_graph=true
topology_unit_executions=1
metadata_runs=0

blocker classes:
      2 rustc-failed

topology blocker:
- rustc-failed: error: linking with `/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin/cc` failed: exit status: 1
  |
  = note:  "/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin/cc" "-m64" "/home/brittonr/git/mantle/target/mantle-self-rust-plan-probe-after-c3ccf629-clean/execution/516_path_file____home_brittonr_git_mantle_vendor_fuse-backend-rs_0.12.0_build-script-build_custom-build_build/rustcX4fSZx/symbols.o" "<2 object files omitted>" "-Wl,--as-needed" "-Wl,-Bstatic" "<sysroot>/lib/rustlib/x86_64-unknown-linux-gnu/lib/{libstd-*,libpanic_unwind-*,libobject-*,libmemchr-*,libaddr2line-*,libgimli-*,libcfg_if-*,librustc_demangle-*,libstd_detect-*,libhashbrown-*,librustc_std_workspace_alloc-*,libminiz_oxide-*,libadler2-*,libunwind-*,liblibc-*,librustc_std_workspace_core-*,liballoc-*,libcore-*,libcompiler_builtins-*}.rlib" "-Wl,-Bdynamic" "-lgcc_s" "-lutil" "-lrt" "-lpthread" "-lm" "-ldl" "-lc" "-L" "/home/brittonr/git/mantle/target/mantle-self-rust-plan-probe-after-c3ccf629-clean/execution/516_path_file____home_brittonr_git_mantle_vendor_fuse-backend-rs_0.12.0_build-script-build_custom-build_build/rustcX4fSZx/raw-dylibs" "-B<sysroot>/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld" "-fuse-ld=lld" "-Wl,--eh-frame-hdr" "-Wl,-z,noexecstack" "-L" "<sysroot>/lib/rustlib/x86_64-unknown-linux-gnu/lib" "-o" "target/mantle-self-rust-plan-probe-after-c3ccf629-clean/execution/516_path_file____home_brittonr_git_mantle_vendor_fuse-backend-rs_0.12.0_build-script-build_custom-build_build/build_script_build" "-Wl,--gc-sections" "-pie" "-Wl,-z,relro,-z,now" "-nodefaultlibs"
  = note: some arguments are omitted. use `--verbose` to show all linker arguments
  = note: env: 'bash': No such file or directory
          clang: error: unable to execute command: No such file or directory
          clang: error: linker command failed due to signal (use -v to see invocation)


error: aborting due to 1 previous error
