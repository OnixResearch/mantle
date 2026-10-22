# Build-script unit graph baseline (fresh run).

The capture driver /home/brittonr/scratch/mantle-da00-t11-graphs.sh regenerates the
lockfile and the unit graph for the four recorded fixtures with the pinned
nightly cargo, offline and locked. All four lock and graph commands exited 0.

fixture=proc-macro2
lock_exit=0
graph_exit=0
9351cd36fd8fab46cd191b8d4e307951e96545e90157d996e6932b6a44cdb6f1  /home/brittonr/scratch/mantle-da00-t11-fixtures/proc-macro2/Cargo.toml
c20f3c8e351cb7a6a8020ba47a0bf21e05ff7f809fc129781fd979bcf5f879fe  /home/brittonr/scratch/mantle-da00-t11-fixtures/proc-macro2/Cargo.lock
26c4441eb92e7eb5476d38e005f64409f98bc7d82dd55673496310bdaf393715  /home/brittonr/scratch/mantle-da00-t11-fixtures/proc-macro2/src/main.rs
fixture=serde-derive
lock_exit=0
graph_exit=0
14fdac5310a721e58745d61d682f4a50778cb501bfdcc54cdb74881faa741eda  /home/brittonr/scratch/mantle-da00-t11-fixtures/serde-derive/Cargo.toml
5867dd1e47f8b4b15d1ee12748b52e9c34590d558c850918d4cfaab1185be5b2  /home/brittonr/scratch/mantle-da00-t11-fixtures/serde-derive/Cargo.lock
317d0882183bbac4d8028af5da9c3c1a6de99acb47301f5908e97d82a1feb978  /home/brittonr/scratch/mantle-da00-t11-fixtures/serde-derive/src/main.rs
fixture=libc
lock_exit=0
graph_exit=0
b79a2d1223a14da28d0dff59f700781cb8f31c03b86082190a6b1700d77196fa  /home/brittonr/scratch/mantle-da00-t11-fixtures/libc/Cargo.toml
a6387e11d4d76af158adb5a5ff5d77fe63e20ac04098f18344f15939cc8ac92c  /home/brittonr/scratch/mantle-da00-t11-fixtures/libc/Cargo.lock
7509dff48a4f2202ae7728173a22ff2031139b0cedbf625fefd6cb20a2e0ed67  /home/brittonr/scratch/mantle-da00-t11-fixtures/libc/src/main.rs
fixture=native-answer-sys
lock_exit=0
graph_exit=0
795cc737fbfb8d07c6dd137f48ef169e6a267443795b97399618042cf680a09d  /home/brittonr/scratch/mantle-da00-t11-fixtures/native-answer-sys/Cargo.toml
44b1f4809bb8109de59e75a7a929811ce1411f646b4046052a034643727f6508  /home/brittonr/scratch/mantle-da00-t11-fixtures/native-answer-sys/Cargo.lock
4f9b75dff8250f383cba8bb93c92ff67a00f0036e83b0c20990748b561a827af  /home/brittonr/scratch/mantle-da00-t11-fixtures/native-answer-sys/src/main.rs
12be295352d19e0e0337a4c88d909c50779f1b764a0f41f69970f98d8d47e392  /home/brittonr/scratch/mantle-da00-t11-fixtures/native-answer-sys/native-answer-sys/Cargo.toml
d74b38188dcc2cbc090076b4485add6f412f2335b039f6101e80a03242aafdec  /home/brittonr/scratch/mantle-da00-t11-fixtures/native-answer-sys/native-answer-sys/build.rs
63e64b8e902359b2a71a6834daeec727b5d5aeb2fc7a85cf85a96fd19f0a3ba1  /home/brittonr/scratch/mantle-da00-t11-fixtures/native-answer-sys/native-answer-sys/src/lib.rs

The fixture digests match the recorded historical baseline in
evidence/historical-da00-baseline-2026-10-14.md.
