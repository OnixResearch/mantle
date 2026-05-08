# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.autoconf.2.61

- `bootstrap/autoconf-2.61.ncl` fetches GNU Autoconf 2.61 from mirrors.kernel.org with fixed hash `sha256-L7gjFM87mxjMvwA1cg2g0uueg9bmerkYwLFjvnPBV8Q=`.
- Intentional Crunch deviations: explicit `$NIX_STORE` `find_input` predecessor lookup, fixed bootstrapped PATH composition, `CONFIG_SHELL` bound to bootstrapped Bash, and local `/tmp/ac2.61` build directory.
- Removed suppressed configure/install failures and fallback copy behavior; required build/install steps now fail closed.
- The output contract now checks named Autoconf entrypoints plus `share/autoconf` instead of accepting either `autoconf` or `autoreconf`.
