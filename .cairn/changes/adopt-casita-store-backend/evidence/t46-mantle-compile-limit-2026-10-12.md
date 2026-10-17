# Mantle compiler-depth accommodation — 2026-10-12

`src/main.rs` sets `#![recursion_limit = "256"]` for the binary crate. Before editing, an isolated nightly `cargo rustc` build with `-Zcrate-attr='recursion_limit="256"'` completed successfully, establishing that the compiler's suggested query-depth increase resolves the original `cmd_attest_async` layout failure. The ordinary source-current build below then exited **0** in 50.57 seconds (Cargo compilation time 50.02 seconds). This is an offline patched-vendor build, not a successful default Nix-shell gate. No runtime behavior is attributed to the compiler attribute.

Exact build command, from repository root:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/etc/profiles/per-user/brittonr/bin:$PATH HOME=/home/brittonr/mantle-compile-agent-home CARGO_HOME=/home/brittonr/mantle-compile-agent-home/cargo TMPDIR=/home/brittonr/mantle-compile-agent-tmp CARGO_TARGET_DIR=/home/brittonr/mantle-compile-agent-target CARGO_NET_OFFLINE=true CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=gcc RUSTFLAGS='-C link-arg=-fuse-ld=mold -C link-arg=-Wl,--allow-multiple-definition' SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo --config .cairn/changes/adopt-casita-store-backend/evidence/t46-isolated-vendor-config-2026-10-12.txt build --locked --offline -p mantle --bin mantle
```

Complete combined tool output (compiler stderr; captured as `artifact://120` in the build session):

```text
warning: Cargo.toml: file `/home/brittonr/git/OnixResearch/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
warning: `mantle` (manifest) generated 1 warning
[kache] the cache index could not be opened after retries (opening index database /var/cache/kache-nix/user-brittonr/index.db: disk I/O error: Error code 4618: disk I/O error).
[kache] Caching is disabled for this build — compilation still succeeds,
[kache] just without cache hits or stores (everything builds uncached).
[kache] This is usually a storage issue: the cache directory is on a
[kache] filesystem that doesn't support reliable file locking, or it is
[kache] being accessed from more than one machine at the same time.
[kache] → set KACHE_CACHE_DIR to a fast, local, single-machine path
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/mantle)
warning: use of deprecated method `std::sync::atomic::Atomic::<u64>::fetch_update`: renamed to `try_update` for consistency
   --> src/build_cmd.rs:499:10
    |
499 |         .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| current.checked_add(1))
    |          ^^^^^^^^^^^^
    |
    = note: `#[warn(deprecated)]` on by default
help: replace the use of the deprecated method
    |
499 -         .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| current.checked_add(1))
499 +         .try_update(Ordering::Relaxed, Ordering::Relaxed, |current| current.checked_add(1))
    |

warning: use of deprecated method `std::sync::atomic::Atomic::<u8>::fetch_update`: renamed to `try_update` for consistency
   --> src/remote_build/live_state/producer.rs:407:20
    |
407 |     let _ = status.fetch_update(Ordering::AcqRel, Ordering::Acquire, |previous| {
    |                    ^^^^^^^^^^^^
    |
help: replace the use of the deprecated method
    |
407 -     let _ = status.fetch_update(Ordering::AcqRel, Ordering::Acquire, |previous| {
407 +     let _ = status.try_update(Ordering::AcqRel, Ordering::Acquire, |previous| {
    |

warning: `mantle` (bin "mantle") generated 2 warnings (run `cargo fix --bin "mantle" -p mantle` to apply 2 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 50.02s
warning: the following packages contain code that will be rejected by a future version of Rust: n0-future v0.3.2, nix v0.29.0, nix v0.31.3, proc-macro-error2 v2.0.1, turso_core v0.8.0-pre.7 (https://github.com/cachix/turso.git?rev=dca55133caa690f90dcdd58d3c4329fb0703659c#dca55133)
note: to see what the problems were, use the option `--future-incompat-report`, or run `cargo report future-incompatibilities --id 1`
```

The resulting `/home/brittonr/mantle-compile-agent-target/debug/mantle --store-backend casita --state-dir /home/brittonr/mantle-compile-agent-tmp/state attest --help` exited **0** and listed attestation subcommands and the `--store-backend` option. A non-mutating `doctor --profile build` invocation with the same backend reached diagnostics but exited **3** because `bwrap` was absent and `/nix/store` was not writable; no derivation-build claim follows.
