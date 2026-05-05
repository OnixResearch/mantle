# V3 binutils configure/BFD boundary

Focused validation of `bootstrap/binutils-tcc.ncl` now gets past the earlier opaque `ERROR: as not built` boundary far enough to configure top-level binutils, configure `libiberty`, `intl`, and `bfd`, and start compiling BFD with TinyCC/musl.

Key changes under validation:
- use `/bin/sh` for configure instead of the limited bootstrap Bash;
- add BusyBox-backed configure utility wrappers;
- wrap TinyCC preprocessing so autoconf sanity probes do not die on the known invalid-syntax probe;
- suppress doc subdir recursion in generated Makefile inputs;
- expose config logs and manual-build markers when the full build falls back.

Current concrete blocker:

```text
make[2]: *** [regex.o] Segmentation fault (core dumped)
make[2]: *** [bfd.lo] Error 1
ERROR: as not built
```

The parent target still fails, but the boundary is now narrowed to TinyCC/musl compiling libiberty `regex.c` and BFD `bfd.c`, rather than hidden configure/gas suppression.

See:
- `V3-binutils-configure-bfd-boundary-validation-summary.json`
- `V3-binutils-configure-bfd-boundary-build.stdout.log`
- `V3-binutils-configure-bfd-boundary-build.stderr.log`

## Extracts

```text
 compiler works... yes
checking whether we are cross compiling... yes
checking for suffix of executables...
checking for suffix of object files... o
checking whether we are using the GNU C compiler... no
checking whether tcc accepts -g... no
checking for tcc option to accept ISO C89... unsupported
checking how to run the C preprocessor... /tmp/binutils-build/tools/cpp-tcc
checking for grep that handles long lines and -e... /crunch/store/mnd29ba2jam4hwgfmrxg0k3ckxhqn2kl-grep-2.4-musl/bin/grep
checking for egrep... /crunch/store/mnd29ba2jam4hwgfmrxg0k3ckxhqn2kl-grep-2.4-musl/bin/egrep
checking for ANSI C header files... no
checking for sys/types.h... yes
checking for sys/stat.h... yes
checking for stdlib.h... yes
checking for string.h... yes
checking for memory.h... yes
checking for strings.h... yes
checking for inttypes.h... yes
checking for stdint.h... yes
checking for unistd.h... yes
checking minix/config.h usability... no
checking minix/config.h presence... no
checking for minix/config.h... no
checking whether it is safe to define __EXTENSIONS__... yes
checking for special C compi
---
 for aclocal... no
checking for autoconf... no
checking for autoheader... no
configure: updating cache ./config.cache
configure: creating ./config.status
config.status: creating Makefile
config.status: creating config.intl
config.status: creating config.h
config.status: executing default-1 commands
Configuring in ./bfd
configure: creating cache ./config.cache
checking build system type... x86_64-unknown-linux-gnu
checking host system type... x86_64-unknown-linux-musl
checking target system type... x86_64-unknown-linux-musl
checking for x86_64-unknown-linux-musl-gcc... tcc
checking for C compiler default output file name... a.out
checking whether the C compiler works... yes
checking whether we are cross compiling... yes
checking for suffix of executables...
checking for suffix of object files... o
checking whether we are using the GNU C compiler... no
checking whether tcc accepts -g... no
checking for tcc option to accept ISO C89... unsupported
checking for library containing strerror... none required
checking for a BSD-compatible install... /tmp/binutils-build/binutils-2.30/install-
---
or!
configure: WARNING: sys/resource.h: proceeding with the compiler's result
configure: WARNING: sys/procfs.h: accepted by the compiler, rejected by the preprocessor!
configure: WARNING: sys/procfs.h: proceeding with the compiler's result
make: the `-l' option requires a positive integral argument
make[2]: *** [regex.o] Segmentation fault (core dumped)
make[1]: *** [all-libiberty] Error 2
make: *** [all] Error 2
Full configure/make failed, trying manual build...
===== libiberty/config.log tail =====
ac_cv_prog_cc_c89='no'
ac_cv_prog_cc_g='no'
ac_cv_prog_cc_tcc_c_o='yes'
ac_cv_safe_to_define___extensions__='yes'
ac_cv_search_strerror='none required'
ac_cv_sizeof_int='4'
ac_cv_sizeof_long='8'
ac_cv_sizeof_long_long='8'
ac_cv_sizeof_size_t='8'
ac_cv_sys_file_offset_bits='no'
ac_cv_sys_largefile_CC='no'
ac_cv_type_intptr_t='yes'
ac_cv_type_long_long='yes'
ac_cv_type_pid_t='yes'
ac_cv_type_ssize_t='yes'
ac_cv_type_uintptr_t='yes'
acx_cv_prog_cc_warning__W='yes'
acx_cv_prog_cc_warning__Wall='yes'
acx_cv_prog_cc_warning__Wcpp_compat='yes'
acx_cv_prog_cc_warning__Wshadow_local='yes'
acx_cv_
---
uild: libiberty
make: *** [regex.o] Segmentation fault (core dumped)
manual build: bfd
make: the `-l' option requires a positive integral argument
make: the `-l' option requires a positive integral argument
make: the `-l' option requires a positive integral argument
Segmentation fault (core dumped)
make[2]: *** [bfd.lo] Error 1
make[1]: *** [all-recursive] Error 1
make: *** [all] Error 2
manual build: opcodes
make: *** No targets specified and no makefile found.  Stop.
manual build: gas
make: *** No targets specified and no makefile found.  Stop.
manual build: binutils
make: *** No targets specified and no makefile found.  Stop.
manual build: ld
make: *** No targets specified and no makefile found.  Stop.
make: the `-l' option requires a positive integral argument
make: the `-l' option requires a positive integral argument
make: the `-l' option requires a positive integral argument
make: the `-l' option requires a positive integral argument
Segmentation fault (core dumped)
make[4]: *** [bfd.lo] Error 1
make[3]: *** [install-recursive] Error 1
make[2]: *** [install] Error 2
make[1]: *
---
ion requires a positive integral argument
make: the `-l' option requires a positive integral argument
Segmentation fault (core dumped)
make[4]: *** [bfd.lo] Error 1
make[3]: *** [install-recursive] Error 1
make[2]: *** [install] Error 2
make[1]: *** [install-bfd] Error 2
make: *** [install] Error 2
ERROR: as not built

```
