# Musl-host provider frontier (2026-06-16)

Task-ID: V2
Covers: r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_linker_wrapper_path]

## Command

```sh
RUN_ROOT=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-2026-06-16
SOURCE_ROOT=/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain
PATH="$SOURCE_ROOT/bin:$PATH" \
cargo run -p mantle --bin mantle -- bootstrap rust-source-provider \
  --recipe bootstrap/rust-source.ncl \
  --route-plan bootstrap/rust-source-musl-host-plan.ncl \
  --output-dir "$RUN_ROOT/provider-out" \
  --verbose
```

Pueue task: `56`
Result: failed with exit code 1 after reaching first-stage `run_rustc` musl-host link.

## Evidence

The first-stage script selected the source-root musl target linker wrapper:

```text
using target linker wrapper: /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-2026-06-16/tmp/mantle-rust-source-provider-GULew5/build/target-linker-bin/cc -> /home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain/bin/x86_64-linux-musl-gcc (x86_64-linux-musl); runtime CRT/unwind dir: /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-2026-06-16/tmp/mantle-rust-source-provider-GULew5/build/target-linker-runtime
```

The child Rust compiler still resolved plain `cc` outside that private wrapper directory and failed to find the musl CRT/runtime inputs:

```text
9832:error: linking with `cc` failed: exit status: 1
9834:  = note:  "cc" "-m64" "rcrt1.o" "crti.o" "crtbeginS.o" ... "-lunwind" ... "-lc" ... "crtendS.o" "crtn.o"
9836:  = note: /nix/store/wfbypjb5ama7gslwr6kv4fss6b9h272m-binutils-patchelfed-ld-2.44/bin/ld: cannot find rcrt1.o: No such file or directory
9837:          /nix/store/wfbypjb5ama7gslwr6kv4fss6b9h272m-binutils-patchelfed-ld-2.44/bin/ld: cannot find crti.o: No such file or directory
9838:          /nix/store/wfbypjb5ama7gslwr6kv4fss6b9h272m-binutils-patchelfed-ld-2.44/bin/ld: cannot find crtbeginS.o: No such file or directory
9839:          /nix/store/wfbypjb5ama7gslwr6kv4fss6b9h272m-binutils-patchelfed-ld-2.44/bin/ld: cannot find -lunwind: No such file or directory
9841:          /nix/store/wfbypjb5ama7gslwr6kv4fss6b9h272m-binutils-patchelfed-ld-2.44/bin/ld: cannot find -lc: No such file or directory
9843:          /nix/store/wfbypjb5ama7gslwr6kv4fss6b9h272m-binutils-patchelfed-ld-2.44/bin/ld: cannot find crtendS.o: No such file or directory
9844:          /nix/store/wfbypjb5ama7gslwr6kv4fss6b9h272m-binutils-patchelfed-ld-2.44/bin/ld: cannot find crtn.o: No such file or directory
9845:          collect2: error: ld returned 1 exit status
```

## Decision

The next repair is to prepend the generated `$BUILD_DIR/target-linker-bin` directory to PATH inside the musl target wrapper branch, so child `rustc` links that ask for plain `cc` resolve to the generated source-root musl wrapper.
