# V3 binutils BFD/libbfd boundary

Focused validation for `bootstrap/binutils-tcc.ncl` advanced past the prior `bfd.lo` TinyCC/musl compile segfault after simplifying BFD error-formatting code and disabling plugin support.

Current boundary:

```text
tcc: ar: can't open file Segmentation fault (core dumped)
make[2]: *** [libbfd.la] Error 1
ERROR: build failed in bfd
```

Saved build log: `/home/brittonr/.local/state/crunch/logs/xbssjk5nj2mkcf0wfn2zknad1m3iknqp-binutils-2.30-tcc.drv.log`

This is evidence of progress only; `gas/as` is not yet produced and V3 remains incomplete.
