# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.autoconf.2.52

- Source pin: `https://mirrors.kernel.org/gnu/autoconf/autoconf-2.52.tar.bz2` with fixed hash `sha256-vCx4E2/YyJCkwgInpyyrGhJTYDyl70WVSkVHLeYr7Fg=`.
- Intentional Crunch deviations: explicit declared-input lookup via `$NIX_STORE`, bootstrapped PATH composition, `CONFIG_SHELL` bound to bootstrapped Bash, TinyCC static C compiler settings, and local `/tmp/autoconf252` work directory.
- The pre-drain derivation suppressed configure/install errors and copied partial outputs as fallback; that is not acceptable evidence for a required bootstrap part.
- This slice removes suppressed failures and fallback-copy behavior, making required configure/build/install/output checks fail closed.
