# V4 mpc host-leakage gate

Task-ID: V4
Covers: bootstrap.part.mpc.1.2.1

A target-specific host-leakage scan cannot be completed until V2 produces an MPC build transcript. The current evidence preserves the invariant instead:

- `bootstrap/mpc-1.2.1.ncl` must continue to resolve `gcc-4.7.4`, GMP, MPFR, Musl, Make, Bash, Coreutils, Sed, and Grep through declared inputs;
- the missing `gcc-4.7.4` provider must not be replaced with host GCC, Nix GCC, or any undeclared compiler;
- future positive evidence must scan the actual MPC build transcript before promotion.

No no-host-leakage success over an MPC build transcript is claimed here, because no such transcript exists yet.
