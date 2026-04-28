# Live-bootstrap Parts Split Validation Transcript

This transcript backs the part-split scaffold claim. The generated `live-part-*` changes are active scaffolds with implementation and verification tasks still unchecked.

## Upstream reference checks

```console
$ cd ~/git/pi-repos/fosslinux--live-bootstrap && grep -n "^mpc " parts.rst && grep -R "mpc-1.2.1\\|mpc-3.2.1" -n steps/manifest steps/mpc-1.2.1/sources parts.rst
998:mpc 3.2.1
steps/manifest:154:build: mpc-1.2.1
steps/mpc-1.2.1/sources:1:f http://mirrors.kernel.org/gnu/mpc/mpc-1.2.1.tar.gz 17503d2c395dfcf106b622dc142683c1199431d095367c6aacba6eec30340459
```

## Generated change count

```console
$ cd /home/brittonr/git/crunch/crunch && find openspec/changes -maxdepth 1 -type d -name "live-part-*" | wc -l
63
```

## All live-part OpenSpec validations

```console
$ for d in openspec/changes/live-part-*; do change=${d##*/}; openspec validate "$change"; done
Change 'live-part-autoconf-2-52' is valid
Change 'live-part-autoconf-2-53' is valid
Change 'live-part-autoconf-2-54' is valid
Change 'live-part-autoconf-2-55' is valid
Change 'live-part-autoconf-2-57' is valid
Change 'live-part-autoconf-2-59' is valid
Change 'live-part-autoconf-2-61' is valid
Change 'live-part-autoconf-2-64' is valid
Change 'live-part-autoconf-2-69' is valid
Change 'live-part-automake-1-10-3' is valid
Change 'live-part-automake-1-11-2' is valid
Change 'live-part-automake-1-15-1' is valid
Change 'live-part-automake-1-6-3' is valid
Change 'live-part-automake-1-7' is valid
Change 'live-part-automake-1-7-8' is valid
Change 'live-part-automake-1-8-5' is valid
Change 'live-part-automake-1-9-6' is valid
Change 'live-part-bash-2-05b' is valid
Change 'live-part-binutils-2-30' is valid
Change 'live-part-binutils-2-41' is valid
Change 'live-part-bison-3-4-1' is valid
Change 'live-part-bzip2-1-0-8-musl' is valid
Change 'live-part-bzip2-1-0-8-tcc' is valid
Change 'live-part-coreutils-5-0-musl' is valid
Change 'live-part-coreutils-5-0-tcc' is valid
Change 'live-part-coreutils-6-10' is valid
Change 'live-part-diffutils-2-7' is valid
Change 'live-part-flex-2-5-11' is valid
Change 'live-part-flex-2-6-4' is valid
Change 'live-part-gawk-3-0-4' is valid
Change 'live-part-gcc-10-5-0' is valid
Change 'live-part-gcc-4-0-4' is valid
Change 'live-part-gcc-4-7-4' is valid
Change 'live-part-gmp-6-2-1' is valid
Change 'live-part-grep-2-4' is valid
Change 'live-part-gzip-1-2-5' is valid
Change 'live-part-heirloom-devtools' is valid
Change 'live-part-libtool-2-2-4' is valid
Change 'live-part-m4-1-4-7' is valid
Change 'live-part-make-3-82' is valid
Change 'live-part-mes-0-27' is valid
Change 'live-part-mpc-1-2-1' is valid
Change 'live-part-mpfr-4-1-0' is valid
Change 'live-part-musl-1-1-24-tcc' is valid
Change 'live-part-musl-1-1-24-tcc-musl' is valid
Change 'live-part-musl-1-2-5-full' is valid
Change 'live-part-oyacc-6-6' is valid
Change 'live-part-patch-2-5-9' is valid
Change 'live-part-perl-5-000' is valid
Change 'live-part-perl-5-003' is valid
Change 'live-part-perl-5-004-05' is valid
Change 'live-part-perl-5-005-03' is valid
Change 'live-part-perl-5-6-2' is valid
Change 'live-part-sed-4-0-9-musl' is valid
Change 'live-part-sed-4-0-9-tcc' is valid
Change 'live-part-seed-full' is valid
Change 'live-part-stage0-posix' is valid
Change 'live-part-tar-1-12' is valid
Change 'live-part-tcc-musl' is valid
Change 'live-part-tcc-musl-prep' is valid
Change 'live-part-tcc-musl-v2' is valid
Change 'live-part-tinycc-0-9-26' is valid
Change 'live-part-tinycc-0-9-27' is valid
```

## Existing grouped OpenSpec validations

```console
$ openspec validate live-bootstrap-binutils-tcc-chain
Change 'live-bootstrap-binutils-tcc-chain' is valid
$ openspec validate live-bootstrap-gcc-4-0-stage
Change 'live-bootstrap-gcc-4-0-stage' is valid
$ openspec validate live-bootstrap-gcc-4-7-stage
Change 'live-bootstrap-gcc-4-7-stage' is valid
$ openspec validate live-bootstrap-source-chain
Change 'live-bootstrap-source-chain' is valid
```

## Index-to-upstream consistency check

```console
$ rustc /tmp/check_live_parts_index.rs -o /tmp/check_live_parts_index && /tmp/check_live_parts_index
checked rows=63
exact parts.rst heading matches=60
documented heading ranges=2
documented upstream mismatches=1
result=pass
```
