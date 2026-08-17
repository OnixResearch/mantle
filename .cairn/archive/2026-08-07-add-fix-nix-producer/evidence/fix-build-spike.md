# Fix build evidence

`packages/fix/fix-src.ncl` pins `https://github.com/psyclyx/fix.git` at revision `fd675c2e938da6c9f444a00893b6716d66c89927`. Its fixed-output hash is `sha256-qLTYqSdaxPNMsI59PVgOCMVZjBn655xujFeraqxyK/M=`.

`packages/fix/zig-toolchain.ncl` pins Zig 0.16.0 with hash `sha256-Bgb91FvN9FDW0bCoP3tdr+5XLZqd5LKoKlzv472t6s0=`. The build records signed nixpkgs cache inputs for libgit2, curl, OpenSSL, zlib, libssh2, and `pkg-config`.

`packages/fix/fix.ncl` runs `zig build --release=fast` in the Mantle sandbox. The output is `qjj0nm512hrcnivix5fd4wcfsyffkp4s-fix-0.3.0`.

The output has signed PathInfo and an artifact attestation with runtime-reference edges to the declared toolchain inputs. Smoke evaluation returned `3` and `42` for the recorded expressions.

Hash mismatch and floating-revision inputs fail before build. The sandbox mounts declared inputs only. `fix instantiate` uses the daemon for store-write transport; it does not run a build. This evidence does not claim evaluator correctness, nixpkgs-scale parity, or realization readiness.
