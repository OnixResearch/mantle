# First-stage musl libgcc_eh unwind frontier (2026-06-16)

Task-ID: V1
Covers: r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_libgcc_eh_unwind]

## Real rerun

Pueue task: `82`
Run root: `/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-rerun-2026-06-16`
Status: `1`

The generated wrapper contained both previous fixes (`PATH="$target_alias_dir:$PATH"` and `-static-pie) mapped_arg="-static"`), but the `run_rustc` hello-world link failed with unresolved unwind symbols:

```text
9832:error: linking with `cc` failed: exit status: 1
9834:  = note:  "cc" "-m64" "rcrt1.o" "crti.o" "crtbeginS.o" ... "-lunwind" ... "-lc" ... "-static-pie" ... "crtendS.o" "crtn.o"
9836:  = note: .../x86_64-linux-musl-ld: .../libstd.rlib(...): undefined reference to `_Unwind_Resume'
9848:          ... undefined reference to `_Unwind_Backtrace'
9851:          ... undefined reference to `_Unwind_GetIP'
9863:          ... undefined reference to `_Unwind_GetDataRelBase'
9865:          ... undefined reference to `_Unwind_GetTextRelBase'
9888:          ... undefined reference to `_Unwind_DeleteException'
9890:          ... undefined reference to `_Unwind_RaiseException'
9901:          collect2: error: ld returned 1 exit status
9906:make: *** [Makefile:172: output/prefix-s/bin/hello_world] Aborted (core dumped)
```

## Source-root archive probe

Pueue task `17` checked the selected source-root GCC archives with `x86_64-linux-musl-nm`:

```text
libgcc.a count: 0
libgcc_eh.a count: 4
```

The full symbol probe showed `libgcc_eh.a` defines `_Unwind_Backtrace`, `_Unwind_DeleteException`, `_Unwind_GetIP`, `_Unwind_GetIPInfo`, `_Unwind_GetTextRelBase`, `_Unwind_RaiseException`, and `_Unwind_Resume`.

## Decision

The private first-stage wrapper must copy `libgcc_eh.a` to `$target_runtime_dir/libunwind.a`, because Rust's target link requests `-lunwind`. It should still keep `libgcc.a` available for libgcc-style aliases.
