# GCC 4.0 c-parse function include matrix validation

- Target: `bootstrap/diag-gcc40-c-parse-boundary.ncl`
- Status: `captured-runtime-boundary`
- Build exit code: `1`
- Saved derivation log: `/home/brittonr/git/crunch/crunch/.crunch-drain/gcc40-runtime-v4-20260507-state/logs/04vz24dn6p583icmxbjfgabhk57j2ai1-diag-gcc40-c-parse-boundary.drv.log`

## Matrix

- `cparse_func_after_system`: rc `139`
- `cparse_func_after_coretypes`: rc `139`
- `cparse_func_after_tm`: rc `139`
- `cparse_func_after_tree`: rc `139`
- `cparse_func_after_langhooks`: rc `139`
- `cparse_func_after_input`: rc `139`
- `cparse_func_after_cpplib`: rc `139`
- `cparse_func_after_c_pragma`: rc `139`
- `cparse_func_after_c_tree`: rc `139`
- `cparse_func_after_flags`: rc `139`
- `cparse_func_after_c_common`: rc `139`

## Conclusion

A trivial function definition after only `config-undef6.h` + `system.h` already segfaults (`rc=139`). All longer GCC c-parse include chains through `c-common.h` also return `rc=139`. Declaration-only controls from earlier evidence still pass, so the active reduced seam is function-definition parsing after the basic GCC system include setup, not a later `c-tree.h`/`flags.h`/`c-common.h` include side effect.
