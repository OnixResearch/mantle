# No-seed native closure proof frontier (2026-06-16)

Task-ID: V2
Covers: r[rust_package_planning.source_built_toolchain_closure.explicit_native_promotion]

## Summary

The current source-built Rust provider is real, but it is not a complete native toolchain closure by itself. A host Rust executable link with an empty PATH fails because `rustc` still needs a host linker driver named `cc`. Pointing `rustc` directly at the provider `rust-lld` also fails because GNU host libc/startup/runtime inputs are not present in the source-built provider closure. Therefore this change must not retire `not-source-built-toolchain-closure` for the current native stack.

## Inputs

- Provider: `/home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider`
- rustc: `/home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider/bin/rustc`
- rust-lld: `/home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld`
- Work dir: `/home/brittonr/git/mantle/target/source-built-native-toolchain-closure/frontier`

## Command 1: source-built provider with empty PATH

```sh
env -i HOME=/home/brittonr/git/mantle/target/source-built-native-toolchain-closure/frontier TMPDIR=/home/brittonr/git/mantle/target/source-built-native-toolchain-closure/frontier PATH= /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider/bin/rustc --sysroot /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider /home/brittonr/git/mantle/target/source-built-native-toolchain-closure/frontier/hello.rs -o /home/brittonr/git/mantle/target/source-built-native-toolchain-closure/frontier/hello-empty-path
status=1
```

### stdout

```text
```

### stderr

```text
error: linker `cc` not found
  |
  = note: No such file or directory (os error 2)

error: aborting due to 1 previous error

```

## Command 2: explicit provider rust-lld as linker

```sh
env -i HOME=/home/brittonr/git/mantle/target/source-built-native-toolchain-closure/frontier TMPDIR=/home/brittonr/git/mantle/target/source-built-native-toolchain-closure/frontier PATH= /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider/bin/rustc --sysroot /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider -C linker=/home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld /home/brittonr/git/mantle/target/source-built-native-toolchain-closure/frontier/hello.rs -o /home/brittonr/git/mantle/target/source-built-native-toolchain-closure/frontier/hello-rust-lld
status=1
```

### stdout

```text
```

### stderr

```text
error: linking with `/home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld` failed: exit status: 1
  |
  = note:  "/home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider/lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld" "-flavor" "gnu" "/home/brittonr/git/mantle/target/source-built-native-toolchain-closure/frontier/rustcwEDGJk/symbols.o" "<2 object files omitted>" "--as-needed" "-Bstatic" "<sysroot>/lib/rustlib/x86_64-unknown-linux-gnu/lib/{libstd-*,libpanic_unwind-*,libobject-*,libmemchr-*,libaddr2line-*,libgimli-*,libcfg_if-*,librustc_demangle-*,libstd_detect-*,libhashbrown-*,librustc_std_workspace_alloc-*,libminiz_oxide-*,libadler2-*,libunwind-*,liblibc-*,librustc_std_workspace_core-*,liballoc-*,libcore-*,libcompiler_builtins-*}.rlib" "-Bdynamic" "-lgcc_s" "-lutil" "-lrt" "-lpthread" "-lm" "-ldl" "-lc" "-L" "/home/brittonr/git/mantle/target/source-built-native-toolchain-closure/frontier/rustcwEDGJk/raw-dylibs" "--eh-frame-hdr" "-z" "noexecstack" "-L" "<sysroot>/lib/rustlib/x86_64-unknown-linux-gnu/lib" "-o" "/home/brittonr/git/mantle/target/source-built-native-toolchain-closure/frontier/hello-rust-lld" "--gc-sections" "-pie" "-z" "relro" "-z" "now"
  = note: some arguments are omitted. use `--verbose` to show all linker arguments
  = note: rust-lld: error: unable to find library -lgcc_s
          rust-lld: error: unable to find library -lutil
          rust-lld: error: unable to find library -lrt
          rust-lld: error: unable to find library -lpthread
          rust-lld: error: unable to find library -lm
          rust-lld: error: unable to find library -ldl
          rust-lld: error: unable to find library -lc
          

error: aborting due to 1 previous error

```

## Decision

Mantle must keep the native closure non-claim until the explicit `--toolchain-closure` manifest can bind source-built host-compatible C/linker/libc/sysroot inputs (or a different Rust provider host topology removes that GNU host link requirement).
