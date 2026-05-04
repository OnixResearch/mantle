# V2 tcc-musl Mes-host attempt notes

Exploratory `bootstrap/tcc-musl.ncl` patch ported `tcc-musl-prep` TinyCC 0.9.27 source normalizations, used `tinycc-0.9.26` rather than the self-built `tcc-musl-prep` compiler for the host compile, and split object compile from final link.

Result: focused validation still failed in `tcc-0.9.27-musl.drv`, but the boundary advanced: TinyCC 0.9.26 compiled `tcc.c` to `tcc-musl.o`; the attempted Mes-runtime static link reached `<- tcc-musl` and then failed through the known broken library diagnostic path (`library '%s' not found`).

The implementation patch was reverted after evidence capture. Next boundary is final host-link library search/runtime handling for the Mes-hosted bridge, not `tcc.c` object compilation.
