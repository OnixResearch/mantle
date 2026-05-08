# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.mpc.1.2.1

- `bootstrap/mpc-1.2.1.ncl` uses the upstream GNU MPC 1.2.1 tarball and fixed hash.
- Configure is driven by bootstrapped Bash and GCC 4.7.4 with static GMP/MPFR/Musl library/include paths.
- Intentional Crunch deviations: explicit `$NIX_STORE` `find_input` contract, fixed `--build/--host=x86_64-unknown-linux-musl`, `--disable-shared --enable-static`, and direct fail-closed output checks instead of a broad install trust claim.
- No unrelated bootstrap files are changed by this part.
