probe: target/mantle-self-rust-plan-probe-after-58d9073d-clean/receipt.json
head: 58d9073dd82a43d804efafb46b63e74eeeacbb79
git_status_short_bytes=0

probe_status=0
topology_execution=blocked
topology_unit_executions=1
metadata_runs=0

blocker classes:
      2 rustc-failed

topology blocker:
- rustc-failed: error: linking with `/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin/cc` failed: exit status: 1
  |
  = note:  "/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin/cc" "-m64" "/home/brittonr/git/mantle/target/mantle-self-rust-plan-probe-after-58d9073d-clean/execution/516_path_file____home_brittonr_git_mantle_vendor_fuse-backend-rs_0.12.0_build-script-build_custom-build_build/rustcjc5zf6/symbols.o" "<2 object files omitted>" "-Wl,--as-needed" "-Wl,-Bstatic" "<sysroot>/lib/rustlib/x86_64-unknown-linux-gnu/lib/{libstd-*,libpanic_unwind-*,libobject-*,libmemchr-*,libaddr2line-*,libgimli-*,libcfg_if-*,librustc_demangle-*,libstd_detect-*,libhashbrown-*,librustc_std_workspace_alloc-*,libminiz_oxide-*,libadler2-*,libunwind-*,liblibc-*,librustc_std_workspace_core-*,liballoc-*,libcore-*,libcompiler_builtins-*}.rlib" "-Wl,-Bdynamic" "-lgcc_s" "-lutil" "-lrt" "-lpthread" "-lm" "-ldl" "-lc" "-L" "/home/brittonr/git/mantle/target/mantle-self-rust-plan-probe-after-58d9073d-clean/execution/516_path_file____home_brittonr_git_mantle_vendor_fuse-backend-rs_0.12.0_build-script-build_custom-build_build/rustcjc5zf6/raw-dylibs" "-B<sysroot>/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld" "-fuse-ld=lld" "-Wl,--eh-frame-hdr" "-Wl,-z,noexecstack" "-L" "<sysroot>/lib/rustlib/x86_64-unknown-linux-gnu/lib" "-o" "target/mantle-self-rust-plan-probe-after-58d9073d-clean/execution/516_path_file____home_brittonr_git_mantle_vendor_fuse-backend-rs_0.12.0_build-script-build_custom-build_build/build_script_build" "-Wl,--gc-sections" "-pie" "-Wl,-z,relro,-z,now" "-nodefaultlibs"
  = note: some arguments are omitted. use `--verbose` to show all linker arguments
  = note: /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld/ld.lld: line 5: /nix/store/3jmhpvfwqag9jcxi21l2lrv79ha7h4p5-rustup-1.29.0/nix-support/ld-wrapper.sh: No such file or directory
          clang: error: unable to execute command: No such file or directory
          clang: error: linker command failed due to signal (use -v to see invocation)


error: aborting due to 1 previous error
