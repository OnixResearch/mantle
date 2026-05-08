# V3 host-leakage status

Task-ID: V3
Covers: bootstrap.gcc47.runtime-validation
Captured: 2026-05-08T21:09:49Z

## Result

A gcc-4.7 host-leakage scan cannot be meaningfully run until V2 produces a gcc-4.7 build transcript. The current negative evidence preserves the host-leakage invariant instead:

- no host compiler, host libc, Nix compiler, or legacy provider may be used to replace the missing `gcc-4.0.4` provider;
- the build gate remains fail-closed at `find_input gcc-4.0.4` / `$GCC4/bin/gcc`;
- future positive gcc-4.7 evidence must scan the actual build transcript and provider paths before promotion.

No no-host-leakage success over a gcc-4.7 build transcript is claimed here, because no such transcript exists yet.
