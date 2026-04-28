# Live-bootstrap Part OpenSpec Index

Generated from `~/git/pi-repos/fosslinux--live-bootstrap/parts.rst` and current Crunch bootstrap derivations.

| Change | Upstream part | Crunch file | Phase |
| --- | --- | --- | --- |
| `live-part-stage0-posix` | `bootstrap-seeds through mescc-tools-extra` | `bootstrap/stage0-posix.ncl` | seed toolchain |
| `live-part-mes-0-27` | `mes 0.27` | `bootstrap/mes.ncl` | scheme and mes libc |
| `live-part-tinycc-0-9-26` | `tinycc 0.9.26` | `bootstrap/tinycc-mes.ncl` | first C compiler |
| `live-part-tinycc-0-9-27` | `tinycc 0.9.27` | `bootstrap/tinycc.ncl` | self-hosted tcc |
| `live-part-make-3-82` | `make 3.82` | `bootstrap/make-tcc.ncl` | first make |
| `live-part-patch-2-5-9` | `patch 2.5.9` | `bootstrap/patch-tcc.ncl` | early tcc tools |
| `live-part-gzip-1-2-5` | `gzip 1.2.5` | `bootstrap/gzip-tcc.ncl` | early tcc tools |
| `live-part-tar-1-12` | `tar 1.12` | `bootstrap/tar-tcc.ncl` | early tcc tools |
| `live-part-sed-4-0-9-tcc` | `sed 4.0.9` | `bootstrap/sed-tcc.ncl` | early tcc tools |
| `live-part-bzip2-1-0-8-tcc` | `bzip2 1.0.8` | `bootstrap/bzip2-tcc.ncl` | early tcc tools |
| `live-part-coreutils-5-0-tcc` | `coreutils 5.0` | `bootstrap/coreutils-5.0-tcc.ncl` | early tcc tools |
| `live-part-oyacc-6-6` | `oyacc 6.6` | `bootstrap/oyacc-tcc.ncl` | early tcc tools |
| `live-part-bash-2-05b` | `bash 2.05b` | `bootstrap/bash-2.05b-tcc.ncl` | early shell |
| `live-part-tcc-musl-prep` | `musl 1.1.24 and musl_target` | `bootstrap/tcc-musl-prep.ncl` | libc boundary |
| `live-part-musl-1-1-24-tcc` | `musl 1.1.24 and musl_target` | `bootstrap/musl-1.1.24-tcc.ncl` | libc boundary |
| `live-part-tcc-musl` | `musl 1.1.24 and musl_target` | `bootstrap/tcc-musl.ncl` | libc boundary |
| `live-part-musl-1-1-24-tcc-musl` | `musl 1.1.24 and musl_target` | `bootstrap/musl-1.1.24-tcc-musl.ncl` | libc boundary |
| `live-part-tcc-musl-v2` | `musl 1.1.24 and musl_target` | `bootstrap/tcc-musl-v2.ncl` | libc boundary |
| `live-part-grep-2-4` | `grep 2.4` | `bootstrap/grep-2.4-musl.ncl` | post-musl tools |
| `live-part-sed-4-0-9-musl` | `sed 4.0.9` | `bootstrap/sed-4.0.9-musl.ncl` | post-musl tools |
| `live-part-bzip2-1-0-8-musl` | `bzip2 1.0.8` | `bootstrap/bzip2-1.0.8-musl.ncl` | post-musl tools |
| `live-part-m4-1-4-7` | `m4 1.4.7` | `bootstrap/m4-1.4.7-musl.ncl` | post-musl tools |
| `live-part-heirloom-devtools` | `heirloom devtools` | `bootstrap/heirloom-devtools.ncl` | post-musl tools |
| `live-part-flex-2-5-11` | `flex 2.5.11` | `bootstrap/flex-2.5.11-musl.ncl` | post-musl tools |
| `live-part-flex-2-6-4` | `flex 2.6.4` | `bootstrap/flex-2.6.4-musl.ncl` | post-musl tools |
| `live-part-bison-3-4-1` | `bison 3.4.1` | `bootstrap/bison-3.4.1-musl.ncl` | post-musl tools |
| `live-part-diffutils-2-7` | `diffutils 2.7` | `bootstrap/diffutils-2.7-musl.ncl` | post-musl tools |
| `live-part-coreutils-5-0-musl` | `coreutils 5.0` | `bootstrap/coreutils-5.0-musl.ncl` | post-musl tools |
| `live-part-coreutils-6-10` | `coreutils 6.10` | `bootstrap/coreutils-6.10-musl.ncl` | post-musl tools |
| `live-part-gawk-3-0-4` | `gawk 3.0.4` | `bootstrap/gawk-3.0.4-musl.ncl` | post-musl tools |
| `live-part-perl-5-000` | `perl 5.000` | `bootstrap/perl-5.000-musl.ncl` | perl ladder |
| `live-part-perl-5-003` | `perl 5.003` | `bootstrap/perl-5.003-musl.ncl` | perl ladder |
| `live-part-perl-5-004-05` | `perl 5.004_05` | `bootstrap/perl-5.004_05-musl.ncl` | perl ladder |
| `live-part-perl-5-005-03` | `perl 5.005_03` | `bootstrap/perl-5.005_03-musl.ncl` | perl ladder |
| `live-part-perl-5-6-2` | `perl 5.6.2` | `bootstrap/perl-5.6.2-musl.ncl` | perl ladder |
| `live-part-autoconf-2-52` | `autoconf 2.52` | `bootstrap/autoconf-2.52.ncl` | autotools ladder |
| `live-part-automake-1-6-3` | `automake 1.6.3` | `bootstrap/automake-1.6.3.ncl` | autotools ladder |
| `live-part-autoconf-2-53` | `autoconf 2.53` | `bootstrap/autoconf-2.53.ncl` | autotools ladder |
| `live-part-automake-1-7` | `automake 1.7` | `bootstrap/automake-1.7.ncl` | autotools ladder |
| `live-part-autoconf-2-54` | `autoconf 2.54` | `bootstrap/autoconf-2.54.ncl` | autotools ladder |
| `live-part-autoconf-2-55` | `autoconf 2.55` | `bootstrap/autoconf-2.55.ncl` | autotools ladder |
| `live-part-automake-1-7-8` | `automake 1.7.8` | `bootstrap/automake-1.7.8.ncl` | autotools ladder |
| `live-part-autoconf-2-57` | `autoconf 2.57` | `bootstrap/autoconf-2.57.ncl` | autotools ladder |
| `live-part-autoconf-2-59` | `autoconf 2.59` | `bootstrap/autoconf-2.59.ncl` | autotools ladder |
| `live-part-automake-1-8-5` | `automake 1.8.5` | `bootstrap/automake-1.8.5.ncl` | autotools ladder |
| `live-part-autoconf-2-61` | `autoconf 2.61` | `bootstrap/autoconf-2.61.ncl` | autotools ladder |
| `live-part-automake-1-9-6` | `automake 1.9.6` | `bootstrap/automake-1.9.6.ncl` | autotools ladder |
| `live-part-automake-1-10-3` | `automake 1.10.3` | `bootstrap/automake-1.10.3.ncl` | autotools ladder |
| `live-part-autoconf-2-64` | `autoconf 2.64` | `bootstrap/autoconf-2.64.ncl` | autotools ladder |
| `live-part-automake-1-11-2` | `automake 1.11.2` | `bootstrap/automake-1.11.2.ncl` | autotools ladder |
| `live-part-autoconf-2-69` | `autoconf 2.69` | `bootstrap/autoconf-2.69.ncl` | autotools ladder |
| `live-part-automake-1-15-1` | `automake 1.15.1` | `bootstrap/automake-1.15.1.ncl` | autotools ladder |
| `live-part-libtool-2-2-4` | `libtool 2.2.4` | `bootstrap/libtool-2.2.4.ncl` | autotools ladder |
| `live-part-binutils-2-30` | `binutils 2.30` | `bootstrap/binutils-tcc.ncl` | first binutils |
| `live-part-gcc-4-0-4` | `gcc 4.0.4` | `bootstrap/gcc-4.0.ncl` | first gcc |
| `live-part-gmp-6-2-1` | `gmp 6.2.1` | `bootstrap/gmp-6.2.1.ncl` | modern gcc prerequisites |
| `live-part-mpfr-4-1-0` | `mpfr 4.1.0` | `bootstrap/mpfr-4.1.0.ncl` | modern gcc prerequisites |
| `live-part-mpc-1-2-1` | `mpc 3.2.1` | `bootstrap/mpc-1.2.1.ncl` | modern gcc prerequisites |
| `live-part-gcc-4-7-4` | `gcc 4.7.4` | `bootstrap/gcc-4.7.ncl` | C++ capable gcc |
| `live-part-musl-1-2-5-full` | `musl 1.2.5` | `bootstrap/musl-full.ncl` | final seed toolchain |
| `live-part-gcc-10-5-0` | `gcc 10.5.0` | `bootstrap/gcc-10.ncl` | final seed toolchain |
| `live-part-binutils-2-41` | `binutils 2.41` | `bootstrap/binutils-full.ncl` | final seed toolchain |
| `live-part-seed-full` | `gcc 10.5.0 through binutils 2.41` | `bootstrap/seed-full.ncl` | final seed toolchain |
