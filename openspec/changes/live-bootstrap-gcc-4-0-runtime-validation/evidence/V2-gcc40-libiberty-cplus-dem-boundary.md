# GCC 4.0 libiberty cplus-dem boundary

Focused validation command:

```sh
timeout 520 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-cplus-store" \
  bootstrap validate bootstrap/gcc-4.0.ncl \
  --evidence-dir "$PWD/.crunch-drain/gcc40-cplus-evidence" \
  --resume
```

Result: validation exits non-zero, but it advances beyond the prior `libiberty` / CPP sanity boundary.
The focused build now completes `libiberty`, `libcpp`, and `gcc` configure, enters `make -C build/libiberty`,
compiles the shimmed `regex.o`, and stops deterministically at `libiberty/cplus-dem.c` under TinyCC:

```text
checking whether ldgetname is declared... no
checking whether times is declared... no
checking for struct tms... no
checking for clock_t... no
checking for .preinit_array/.init_array/.fini_array support... yes
checking if mkdir takes one argument... yes
Using `/tmp/gcc-build/gcc-4.0.4/gcc/config/i386/i386.c' for machine-specific logic.
Using `/tmp/gcc-build/gcc-4.0.4/gcc/config/i386/i386.md' as machine description file.
Using the following target machine macro files:
	/tmp/gcc-build/gcc-4.0.4/gcc/config/i386/biarch64.h
	/tmp/gcc-build/gcc-4.0.4/gcc/config/i386/i386.h
	/tmp/gcc-build/gcc-4.0.4/gcc/config/i386/unix.h
	/tmp/gcc-build/gcc-4.0.4/gcc/config/i386/att.h
	/tmp/gcc-build/gcc-4.0.4/gcc/config/dbxelf.h
	/tmp/gcc-build/gcc-4.0.4/gcc/config/elfos.h
	/tmp/gcc-build/gcc-4.0.4/gcc/config/svr4.h
	/tmp/gcc-build/gcc-4.0.4/gcc/config/linux.h
	/tmp/gcc-build/gcc-4.0.4/gcc/config/i386/x86-64.h
	/tmp/gcc-build/gcc-4.0.4/gcc/config/i386/linux64.h
Using host-linux.o for host machine hooks.
checking whether NLS is requested... no
checking what assembler to use... /crunch/store/rzzs05bbvjbn8d7ng6vd9hjw8n6azq6r-binutils-2.30-tcc/bin/as
checking what linker to use... /crunch/store/rzzs05bbvjbn8d7ng6vd9hjw8n6azq6r-binutils-2.30-tcc/bin/ld
checking what nm to use... nm
checking what objdump to use... objdump
checking assembler for .balign and .p2align... yes
checking assembler for .p2align with maximum skip... no
checking assembler for working .subsection -1... no
checking assembler for .weak... yes
checking assembler for .nsubspa comdat... no
checking assembler for .hidden... yes
checking linker for .hidden support... no
checking assembler for .sleb128 and .uleb128... no
checking assembler for eh_frame optimization... no
checking assembler for section merging support... no
checking assembler for section merging support... (cached) no
checking assembler for COMDAT group support... no
checking assembler for COMDAT group support... no
checking assembler for thread-local storage support... no
checking linker -Bstatic/-Bdynamic option... no
checking assembler for filds and fists mnemonics... no
checking assembler for cmov syntax... no
checking assembler for GOTOFF in data... no
checking assembler for dwarf2 debug_line support... no
checking assembler for buggy dwarf2 .file directive... no
checking assembler for --gdwarf2 option... yes
checking assembler for --gstabs option... yes
checking linker read-only and read-write section mixing... unknown
checking linker PT_GNU_EH_FRAME support... no
checking linker position independent executable support... no
checking linker --as-needed support... no
Using ggc-page for garbage collection.
checking whether to enable maintainer-specific portions of Makefiles... no
Links are now set up to build a native compiler for x86_64-unknown-linux-gnu.
configure: creating ./config.status
config.status: creating Makefile
config.status: creating gccbug
config.status: creating mklibgcc
config.status: creating libada-mk
config.status: creating auto-host.h
config.status: executing default commands
make: Entering directory `/tmp/gcc-build/build/gcc'
make: `/tmp/gcc-build/gcc-4.0.4/gcc/gengtype-yacc.c' is up to date.
make: Leaving directory `/tmp/gcc-build/build/gcc'
building libiberty
make: Entering directory `/tmp/gcc-build/build/libiberty'
if [ x"" != x ] && [ ! -d pic ]; then \
  mkdir pic; \
else true; fi
touch stamp-picdir
if [ x"" != x ]; then \
  gcc40-cc -c -DHAVE_CONFIG_H -I/crunch/store/l9srrg1y7ipfmz62n3pf6n7f7i8r23sd-musl-1.1.24-tcc-musl/include -D_GNU_SOURCE -DHAVE_ALLOCA_H -I. -I/tmp/gcc-build/gcc-4.0.4/libiberty/../include    /tmp/gcc-build/gcc-4.0.4/libiberty/regex.c -o pic/regex.o; \
else true; fi
gcc40-cc -c -DHAVE_CONFIG_H -I/crunch/store/l9srrg1y7ipfmz62n3pf6n7f7i8r23sd-musl-1.1.24-tcc-musl/include -D_GNU_SOURCE -DHAVE_ALLOCA_H -I. -I/tmp/gcc-build/gcc-4.0.4/libiberty/../include   /tmp/gcc-build/gcc-4.0.4/libiberty/regex.c -o regex.o
if [ x"" != x ]; then \
  gcc40-cc -c -DHAVE_CONFIG_H -I/crunch/store/l9srrg1y7ipfmz62n3pf6n7f7i8r23sd-musl-1.1.24-tcc-musl/include -D_GNU_SOURCE -DHAVE_ALLOCA_H -I. -I/tmp/gcc-build/gcc-4.0.4/libiberty/../include    /tmp/gcc-build/gcc-4.0.4/libiberty/cplus-dem.c -o pic/cplus-dem.o; \
else true; fi
gcc40-cc -c -DHAVE_CONFIG_H -I/crunch/store/l9srrg1y7ipfmz62n3pf6n7f7i8r23sd-musl-1.1.24-tcc-musl/include -D_GNU_SOURCE -DHAVE_ALLOCA_H -I. -I/tmp/gcc-build/gcc-4.0.4/libiberty/../include   /tmp/gcc-build/gcc-4.0.4/libiberty/cplus-dem.c -o cplus-dem.o
make: *** [cplus-dem.o] Segmentation fault (core dumped)
make: Leaving directory `/tmp/gcc-build/build/libiberty'
```

The slice also records explicit configure cache values and configure-only bridges (`gcc40-cpp`, `gcc40-cc` conftest executable stub) used to keep legacy Autoconf from falling back to unbounded link probes.
