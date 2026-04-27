Task-ID: I1
Covers: bootstrap.source.chain.implementation, bootstrap.fullsource.claim.evidence, bootstrap.stagex.selfbuild.proof

# Live-bootstrap source-chain inventory

Status: complete for inventory. This file does not claim any placeholder is
functional; it records the derivation/source/provenance work required before the
implementation tasks may start.

Sources used for this inventory:

- Checked-in crunch placeholders under `bootstrap/*.ncl`.
- Archived partial-scaffolding changes:
  - `openspec/changes/archive/2026-04-26-live-bootstrap-seed-chain/`
  - `openspec/changes/archive/2026-04-26-live-bootstrap-intermediate-tools/`
- External reference: `fosslinux/live-bootstrap` commit
  `9a268c4c39cae952b268bc86da342be2175f03d4`, specifically `parts.rst` and
  `steps/*/sources`.

## Existing chain derivations in crunch

These already exist as checked-in derivation files and are prerequisites for the
missing chain:

| Derivation | Output role | Source/provenance state |
|---|---|---|
| `bootstrap/stage0-posix.ncl` | hex0/stage0-posix base tools | already checked in |
| `bootstrap/mes.ncl` | GNU Mes and mescc | already checked in |
| `bootstrap/tinycc.ncl` / `bootstrap/tinycc-mes.ncl` | tcc 0.9.26/0.9.27 bridge | already checked in |
| `bootstrap/make-tcc.ncl` | make 3.82 for tcc era | already checked in |
| `bootstrap/sed-tcc.ncl` | sed 4.0.9 tcc-era tool | currently uses GNU mirror + SHA-256 SRI |
| `bootstrap/patch-tcc.ncl` | patch 2.5.9 tcc-era tool | currently uses GNU mirror + SHA-256 SRI |
| `bootstrap/gzip-tcc.ncl` | gzip 1.2.4 tcc-era tool | currently uses GNU mirror + SHA-256 SRI; upstream live-bootstrap current uses gzip 1.2.5 |
| `bootstrap/tar-tcc.ncl` | tar 1.12 tcc-era tool | currently uses GNU mirror + SHA-256 SRI |

## Required derivation inventory

### `bootstrap/binutils-tcc.ncl`

Purpose: replace the placeholder with the complete TinyCC-era and post-musl
intermediate chain ending in binutils 2.30.

Required ordered derivations:

1. existing `stage0-posix`, `mes`, `tinycc`, `make-tcc`, `sed-tcc`,
   `patch-tcc`, `gzip-tcc`, and `tar-tcc`.
2. `bootstrap/bzip2-tcc.ncl` — bzip2 1.0.8.
3. `bootstrap/coreutils-5.0-tcc.ncl` — coreutils 5.0 early utility set.
4. `bootstrap/oyacc-tcc.ncl` — oyacc 6.6.
5. `bootstrap/bash-2.05b-tcc.ncl` — bash 2.05b.
6. `bootstrap/tcc-musl-prep.ncl` — patched tcc 0.9.27 used to build the first
   musl boundary.
7. `bootstrap/musl-1.1.24-tcc.ncl` — first musl libc built by TinyCC.
8. `bootstrap/tcc-musl.ncl` — tcc 0.9.27 rebuilt against musl.
9. `bootstrap/musl-1.1.24-tcc-musl.ncl` — second musl rebuild with tcc-musl.
10. `bootstrap/tcc-musl-v2.ncl` — tcc rebuilt after the fixed musl.
11. `bootstrap/grep-2.4-musl.ncl` — grep 2.4 linked against musl.
12. `bootstrap/musl-1.1.24-v3.ncl` — musl rebuild unlocking post-musl support.
13. `bootstrap/sed-4.0.9-musl.ncl` — sed rebuilt against musl.
14. `bootstrap/bzip2-1.0.8-musl.ncl` — bzip2 rebuilt against musl.
15. `bootstrap/m4-1.4.7-musl.ncl` — m4 1.4.7.
16. `bootstrap/heirloom-devtools.ncl` — Heirloom yacc/lex bridge.
17. `bootstrap/flex-2.5.11-musl.ncl` — initial flex.
18. `bootstrap/flex-2.6.4-musl.ncl` — refreshed flex.
19. `bootstrap/bison-2.3-musl.ncl` — legacy bison required by older autotools.
20. `bootstrap/bison-3.4.1-musl.ncl` — modern bison bootstrap stage.
21. `bootstrap/diffutils-2.7-musl.ncl` — diffutils.
22. `bootstrap/coreutils-5.0-musl.ncl` — rebuilt coreutils.
23. `bootstrap/coreutils-6.10-musl.ncl` — date/mktemp/sha256sum support.
24. `bootstrap/gawk-3.0.4-musl.ncl` — awk for later generated sources.
25. `bootstrap/perl-5.000-musl.ncl`.
26. `bootstrap/perl-5.003-musl.ncl`.
27. `bootstrap/perl-5.004_05-musl.ncl`.
28. `bootstrap/perl-5.005_03-musl.ncl`.
29. `bootstrap/perl-5.6.2-musl.ncl`.
30. `bootstrap/autoconf-2.52.ncl`.
31. `bootstrap/automake-1.6.3.ncl`.
32. `bootstrap/autoconf-2.53.ncl`.
33. `bootstrap/automake-1.7.ncl`.
34. `bootstrap/autoconf-2.54.ncl`.
35. `bootstrap/autoconf-2.55.ncl`.
36. `bootstrap/automake-1.7.8.ncl`.
37. `bootstrap/autoconf-2.57.ncl`.
38. `bootstrap/autoconf-2.59.ncl`.
39. `bootstrap/automake-1.8.5.ncl`.
40. `bootstrap/autoconf-2.61.ncl`.
41. `bootstrap/automake-1.9.6.ncl`.
42. `bootstrap/automake-1.10.3.ncl`.
43. `bootstrap/autoconf-2.64.ncl`.
44. `bootstrap/automake-1.11.2.ncl`.
45. `bootstrap/autoconf-2.69.ncl`.
46. `bootstrap/libtool-2.2.4.ncl`.
47. `bootstrap/automake-1.15.1.ncl`.
48. `bootstrap/binutils-tcc.ncl` — final binutils 2.30 derivation consuming the
    above chain.

Required source/provenance entries for new and not-yet-pinned inputs:

| Step | URL or path | Digest | Provenance note |
|---|---|---|---|
| bzip2-1.0.8 | `https://sourceware.org/pub/bzip2/bzip2-1.0.8.tar.gz` | SHA-256 `ab5a03176ee106d3f0fa90e381da478ddae405918153cca248e682cd0c4a2269` | live-bootstrap `steps/bzip2-1.0.8/sources` |
| coreutils-5.0 | `https://mirrors.kernel.org/gnu/coreutils/coreutils-5.0.tar.bz2` | SHA-256 `c25b36b8af6e0ad2a875daf4d6196bd0df28a62be7dd252e5f99a4d5d7288d95` | live-bootstrap `steps/coreutils-5.0/sources` |
| oyacc-6.6 | `https://github.com/ibara/yacc/releases/download/oyacc-6.6/oyacc-6.6.tar.gz` | SHA-256 `eb0866e740b79bd3a23e0ca47885eb3148aab18d77a4bedba96e979d8b4ebfe1` | live-bootstrap `steps/oyacc-6.6/sources` |
| bash-2.05b | `https://mirrors.kernel.org/gnu/bash/bash-2.05b.tar.gz` | SHA-256 `ba03d412998cc54bd0b0f2d6c32100967d3137098affdc2d32e6e7c11b163fe4` | live-bootstrap `steps/bash-2.05b/sources` |
| tcc-0.9.27 | `https://download.savannah.gnu.org/releases/tinycc/tcc-0.9.27.tar.bz2` | SHA-256 `de23af78fca90ce32dff2dd45b3432b2334740bb9bb7b05bf60fdbfc396ceb9c` | live-bootstrap `steps/tcc-0.9.27/sources` plus step patches/checksums |
| musl-1.1.24 | `https://musl.libc.org/releases/musl-1.1.24.tar.gz` | SHA-256 `1370c9a812b2cf2a7d92802510cca0058cc37e66a7bedd70051f0a34015022a3` | live-bootstrap `steps/musl-1.1.24/sources` plus step patches |
| grep-2.4 | `https://mirrors.kernel.org/gnu/grep/grep-2.4.tar.gz` | SHA-256 `a32032bab36208509466654df12f507600dfe0313feebbcd218c32a70bf72a16` | live-bootstrap `steps/grep-2.4/sources` |
| m4-1.4.7 | `https://mirrors.kernel.org/gnu/m4/m4-1.4.7.tar.bz2` | SHA-256 `a88f3ddaa7c89cf4c34284385be41ca85e9135369c333fdfa232f3bf48223213` | live-bootstrap `steps/m4-1.4.7/sources` |
| heirloom-devtools-070527 | `http://downloads.sourceforge.net/project/heirloom/heirloom-devtools/070527/heirloom-devtools-070527.tar.bz2` | SHA-256 `9f233d8b78e4351fe9dd2d50d83958a0e5af36f54e9818521458a08e058691ba` | live-bootstrap `steps/heirloom-devtools-070527/sources` |
| flex-2.5.11 | `https://github.com/westes/flex/archive/d160f0247ba1611aa59d28f027d6292ba24abb50.tar.gz` | SHA-256 `68aa10c473b6010ffad680cada09fc4eec6b3cc6e415cc2339e5fc2385ccc142` | live-bootstrap `steps/flex-2.5.11/sources` git snapshot |
| flex-2.6.4 | `https://github.com/westes/flex/releases/download/v2.6.4/flex-2.6.4.tar.gz` | SHA-256 `e87aae032bf07c26f85ac0ed3250998c37621d95f8bd748b31f15b33c45ee995` | live-bootstrap `steps/flex-2.6.4/sources` |
| bison-2.3 | `http://mirrors.kernel.org/gnu/bison/bison-2.3.tar.bz2` | SHA-256 `b10d7e9e354be72aee4e4911cf19dd27b5c527d4e7200857365b5fcdeea0dffb` | live-bootstrap `steps/bison-2.3/sources` |
| bison-2.3 gnulib | `https://https.git.savannah.gnu.org/git/gnulib.git~b28236b` / `gnulib-b28236b.tar.gz` | SHA-256 `0190f28cb155fedd22bf8558c3e8705eed9eacfb7ae29e7508d025a68eb90899` | live-bootstrap `steps/bison-2.3/sources` git snapshot |
| bison-3.4.1 | `https://mirrors.kernel.org/gnu/bison/bison-3.4.1.tar.xz` | SHA-256 `27159ac5ebf736dffd5636fd2cd625767c9e437de65baa63cb0de83570bd820d` | live-bootstrap `steps/bison-3.4.1/sources` |
| diffutils-2.7 | `https://mirrors.kernel.org/gnu/diffutils/diffutils-2.7.tar.gz` | SHA-256 `d5f2489c4056a31528e3ada4adacc23d498532b0af1a980f2f76158162b139d6` | live-bootstrap `steps/diffutils-2.7/sources` |
| coreutils-6.10 | `https://mirrors.kernel.org/gnu/coreutils/coreutils-6.10.tar.lzma` | SHA-256 `8b05bba1b2726a164e444c314e3f359604b58216be704bed8f2e028449cc6204` | live-bootstrap `steps/coreutils-6.10/sources` |
| gawk-3.0.4 | `https://mirrors.kernel.org/gnu/gawk/gawk-3.0.4.tar.gz` | SHA-256 `5cc35def1ff4375a8b9a98c2ff79e95e80987d24f0d42fdbb7b7039b3ddb3fb0` | live-bootstrap `steps/gawk-3.0.4/sources` |
| perl-5.000 | `https://github.com/Perl/perl5/archive/perl-5.000.tar.gz` | SHA-256 `1ae43c8d2983404b9eec61c96e3ffa27e7b07e08215c95c015a4ab0095373ef3` | live-bootstrap `steps/perl-5.000/sources` |
| perl-5.003 | `https://github.com/Perl/perl5/archive/perl-5.003.tar.gz` | SHA-256 `9fa29beb2fc4a3c373829fc051830796de301f32a719d0b52a400d1719bbd7b1` | live-bootstrap `steps/perl-5.003/sources` |
| perl-5.004_05 | `https://www.cpan.org/src/5.0/perl5.004_05.tar.gz` | SHA-256 `1184478b298978b164a383ed5661e3a117c48ab97d6d0ab7ef614cdbe918b9eb` | live-bootstrap `steps/perl5.004-05/sources` |
| perl-5.005_03 | `https://www.cpan.org/src/5.0/perl5.005_03.tar.gz` | SHA-256 `93f41cd87ab8ee83391cfa39a63b076adeb7c3501d2efa31b98d0ef037122bd1` | live-bootstrap `steps/perl5.005-03/sources` |
| perl-5.6.2 | `https://www.cpan.org/src/5.0/perl-5.6.2.tar.gz` | SHA-256 `a5e66f6ebf701b0567f569f57cae82abf5ce57af70a2b45ae71323b61f49134e` | live-bootstrap `steps/perl-5.6.2/sources` |
| autoconf-2.52 | `https://mirrors.kernel.org/gnu/autoconf/autoconf-2.52.tar.bz2` | SHA-256 `4681bcbb9c9298c506f6405a7deb62c54fc3b339d3239a8f36a5df83daaec94f` | live-bootstrap `steps/autoconf-2.52/sources` |
| automake-1.6.3 | `https://mirrors.kernel.org/gnu/automake/automake-1.6.3.tar.bz2` | SHA-256 `0dbafacaf21e135cab35d357a14bdcd981d2f2d00e1387801be8091a31b7bb81` | live-bootstrap `steps/automake-1.6.3/sources` |
| autoconf-2.53 | `https://mirrors.kernel.org/gnu/autoconf/autoconf-2.53.tar.bz2` | SHA-256 `6b217a064c6d06603d50a3ad05129aef9435367810c10894210b8dad965d2306` | live-bootstrap `steps/autoconf-2.53/sources` |
| automake-1.7 | `https://mirrors.kernel.org/gnu/automake/automake-1.7.tar.bz2` | SHA-256 `6633ee1202375e3c8798a92e1b7f46894f78d541aeea7f49654503fdc0b28835` | live-bootstrap `steps/automake-1.7/sources` |
| autoconf-2.54 | `https://mirrors.kernel.org/gnu/autoconf/autoconf-2.54.tar.bz2` | SHA-256 `a74aea954f36c7beeb6cc47b96a408c3e04e7ad635f614e65250dbcd8ec0bd28` | live-bootstrap `steps/autoconf-2.54/sources` |
| autoconf-2.55 | `https://mirrors.kernel.org/gnu/autoconf/autoconf-2.55.tar.bz2` | SHA-256 `f757158a04889b265203eecd8ca92568e2a67c3b9062fa6bff7a0a6efd2244ac` | live-bootstrap `steps/autoconf-2.55/sources` |
| automake-1.7.8 | `https://mirrors.kernel.org/gnu/automake/automake-1.7.8.tar.bz2` | SHA-256 `2dddc3b51506e702647ccc6757e15c05323fa67245d2d53e81ed36a832f9be42` | live-bootstrap `steps/automake-1.7.8/sources` |
| autoconf-2.57 | `https://mirrors.kernel.org/gnu/autoconf/autoconf-2.57.tar.bz2` | SHA-256 `e1035aa2c21fae2a934d1ab56c774ce9d22717881dab8a1a5b16d294fb793489` | live-bootstrap `steps/autoconf-2.57/sources` |
| autoconf-2.59 | `https://mirrors.kernel.org/gnu/autoconf/autoconf-2.59.tar.bz2` | SHA-256 `f0cde70a8f135098a6a3e85869f2e1cc3f141beea766fa3d6636e086cd8b90a7` | live-bootstrap `steps/autoconf-2.59/sources` |
| automake-1.8.5 | `https://mirrors.kernel.org/gnu/automake/automake-1.8.5.tar.bz2` | SHA-256 `84c93aaa3c3651a9e7474b721b0e6788318592509e7de604bafe4ea8049dc410` | live-bootstrap `steps/automake-1.8.5/sources` |
| autoconf-2.61 | `https://mirrors.kernel.org/gnu/autoconf/autoconf-2.61.tar.bz2` | SHA-256 `93a2ceab963618b021db153f0c881a2de82455c1dc7422be436fcd5c554085a1` | live-bootstrap `steps/autoconf-2.61/sources` |
| automake-1.9.6 | `https://mirrors.kernel.org/gnu/automake/automake-1.9.6.tar.bz2` | SHA-256 `8eccaa98e1863d10e4a5f861d8e2ec349a23e88cb12ad10f6b6f79022ad2bb8d` | live-bootstrap `steps/automake-1.9.6/sources` |
| automake-1.10.3 | `https://mirrors.kernel.org/gnu/automake/automake-1.10.3.tar.bz2` | SHA-256 `e98ab43bb839c31696a4202e5b6ff388b391659ef2387cf9365019fad17e1adc` | live-bootstrap `steps/automake-1.10.3/sources` |
| autoconf-2.64 | `https://mirrors.kernel.org/gnu/autoconf/autoconf-2.64.tar.xz` | SHA-256 `32d977213320b8ae76c71175305301197f2b0e04e72d70694bc3d3e2ae6c7248` | live-bootstrap `steps/autoconf-2.64/sources` |
| automake-1.11.2 | `https://mirrors.kernel.org/gnu/automake/automake-1.11.2.tar.bz2` | SHA-256 `4f46d1f9380c8a3506280750f630e9fc915cb1a435b724be56b499d016368718` | live-bootstrap `steps/automake-1.11.2/sources` |
| autoconf-2.69 | `https://mirrors.kernel.org/gnu/autoconf/autoconf-2.69.tar.xz` | SHA-256 `64ebcec9f8ac5b2487125a86a7760d2591ac9e1d3dbd59489633f9de62a57684` | live-bootstrap `steps/autoconf-2.69/sources` |
| libtool-2.2.4 | `https://mirrors.kernel.org/gnu/libtool/libtool-2.2.4.tar.lzma` | SHA-256 `d81839fa4d566dbef7c286fdca9b430d3530983fff6d389fac0f08baf27e4c3a` | live-bootstrap `steps/libtool-2.2.4/sources` |
| automake-1.15.1 | `https://mirrors.kernel.org/gnu/automake/automake-1.15.1.tar.xz` | SHA-256 `af6ba39142220687c500f79b4aa2f181d9b24e4f8d8ec497cea4ba26c64bedaf` | live-bootstrap `steps/automake-1.15.1/sources` |
| binutils-2.30 | `https://mirrors.kernel.org/gnu/binutils/binutils-2.30.tar.xz` | SHA-256 `6e46b8aeae2f727a36f0bd9505e405768a72218f1796f0d09757d45209871ae6` | live-bootstrap `steps/binutils-2.30/sources` |

### `bootstrap/gcc-4.0.ncl`

Purpose: replace the placeholder with gcc-core 4.0.4 built from TinyCC + musl +
binutils 2.30 + the intermediate chain.

Required derivations:

- all `bootstrap/binutils-tcc.ncl` prerequisites above;
- `bootstrap/binutils-tcc.ncl` functional output;
- `bootstrap/gcc-4.0.ncl` final gcc-core 4.0.4 output.

Required source/provenance entries:

| Step | URL or path | Digest | Provenance note |
|---|---|---|---|
| gcc-core-4.0.4 | `https://mirrors.kernel.org/gnu/gcc/gcc-4.0.4/gcc-core-4.0.4.tar.bz2` | SHA-256 `e9bf58c761a4f988311aef6b41f12fd5c7e51d09477468fb73826aecc1be32e7` | live-bootstrap `steps/gcc-4.0.4/sources` |
| automake-1.16.3 helper | `https://mirrors.kernel.org/gnu/automake/automake-1.16.3.tar.xz` | SHA-256 `ff2bf7656c4d1c6fdda3b8bebb21f09153a736bcba169aaf65eab25fa113bf3a` | live-bootstrap `steps/gcc-4.0.4/sources`; verify whether still required by crunch adaptation before pinning |

### `bootstrap/gcc-4.7.ncl`

Purpose: replace the placeholder with gcc 4.7.4 built by gcc 4.0.4 and the
intermediate autotools/libtool/perl chain.

Required derivations:

- all `bootstrap/gcc-4.0.ncl` prerequisites;
- gcc 4.0.4 functional output;
- post-gcc-4.0 support tools required by the adapted live-bootstrap step;
- `bootstrap/gcc-4.7.ncl` final gcc 4.7.4 output.

Required source/provenance entries:

| Step | URL or path | Digest | Provenance note |
|---|---|---|---|
| gcc-4.7.4 | `https://mirrors.kernel.org/gnu/gcc/gcc-4.7.4/gcc-4.7.4.tar.bz2` | SHA-256 `92e61c6dc3a0a449e62d72a38185fda550168a86702dea07125ebd3ec3996282` | live-bootstrap `steps/gcc-4.7.4/sources` |

### `bootstrap/gcc-10.ncl`

Purpose: replace the placeholder with gcc 10.5.0 or the selected modern GCC from
the pinned live-bootstrap profile.

Required derivations:

- all `bootstrap/gcc-4.7.ncl` prerequisites;
- gcc 4.7.4 functional output;
- `gmp`, `mpfr`, and `mpc` derivations required by the GCC 10 build (versions
  and source pins must come from the selected live-bootstrap commit before I5);
- `bootstrap/gcc-10.ncl` final gcc 10.x output.

Required source/provenance entries:

| Step | URL or path | Digest | Provenance note |
|---|---|---|---|
| gcc-10.5.0 | `https://mirrors.kernel.org/gnu/gcc/gcc-10.5.0/gcc-10.5.0.tar.xz` | SHA-256 `25109543fdf46f397c347b5d8b7a2c7e5694a5a51cce4b9c6e1ea8a71ca307c1` | live-bootstrap `steps/gcc-10.5.0/sources` |

### `bootstrap/musl-full.ncl`

Purpose: replace the placeholder with final musl 1.2.5 built by the modern
source-built GCC.

Required derivations:

- gcc 10.x functional output;
- `bootstrap/musl-full.ncl` final musl 1.2.5 output.

Required source/provenance entries:

| Step | URL or path | Digest | Provenance note |
|---|---|---|---|
| musl-1.2.5 | `https://musl.libc.org/releases/musl-1.2.5.tar.gz` | SHA-256 `a9a118bbe84d8764da0ea0d28b3ab3fae8477fc7e4085d90102b8596fc7c75e4` | live-bootstrap `steps/musl-1.2.5/sources` |

### `bootstrap/binutils-full.ncl`

Purpose: replace the placeholder with final binutils 2.41 built by the modern
source-built GCC.

Required derivations:

- gcc 10.x functional output;
- final musl output when the binutils build is configured for the target sysroot;
- `bootstrap/binutils-full.ncl` final binutils 2.41 output.

Required source/provenance entries:

| Step | URL or path | Digest | Provenance note |
|---|---|---|---|
| binutils-2.41 | `https://mirrors.kernel.org/gnu/binutils/binutils-2.41.tar.xz` | SHA-256 `ae9a5789e23459e59606e6714723f2d3ffc31c03174191ef0d015bdf06007450` | live-bootstrap `steps/binutils-2.41/sources` |

### `bootstrap/seed-full.ncl`

Purpose: normalize source-built gcc, musl, and binutils outputs into the same
provider contract currently supplied by `bootstrap/seed-legacy.ncl`.

Required derivations:

- `bootstrap/gcc-10.ncl` functional output;
- `bootstrap/musl-full.ncl` functional output;
- `bootstrap/binutils-full.ncl` functional output;
- `bootstrap/seed-full.ncl` final normalized provider output.

Required generated/provenance entries:

| Artifact | Required metadata | Provenance note |
|---|---|---|
| provider metadata | provider id, selected profile, source-root or lineage manifest digest, provider output digest, retained tools, reduction metadata, provider notes | generated by `seed-full.ncl`; digest in proof metadata |
| normalized tool paths | target-prefixed `gcc`, `g++`, `cpp`, `ar`, `as`, `ld`, `nm`, `objcopy`, `objdump`, `ranlib`, `readelf`, `strip` | copied from source-built gcc/binutils outputs |
| normalized sysroot | `<target>/include`, `<target>/lib/libgcc_s.so*`, `libc.so`, C++ runtime as needed | copied from source-built musl/gcc outputs |

## Patch and generated-artifact rule for all future tasks

When I2-I8 import a live-bootstrap patch, generated parser/lexer source, or
other carried artifact, the task must record:

- repository path where crunch stores the artifact;
- BLAKE3 digest for crunch-owned/vendored artifact bytes;
- upstream source path, commit `9a268c4c39cae952b268bc86da342be2175f03d4`, and
  license/provenance note;
- the first derivation that consumes it.

No later task may replace a placeholder by embedding an untracked patch or
source blob directly in a builder script.

## Inventory verification

Command:

```sh
test -s openspec/changes/live-bootstrap-source-chain/evidence/I1-chain-inventory.md
rg 'bootstrap/binutils-tcc.ncl|bootstrap/gcc-4.0.ncl|bootstrap/gcc-4.7.ncl|bootstrap/gcc-10.ncl|bootstrap/musl-full.ncl|bootstrap/binutils-full.ncl|bootstrap/seed-full.ncl|source-root|stagex|9a268c4c39cae952b268bc86da342be2175f03d4' openspec/changes/live-bootstrap-source-chain/evidence/I1-chain-inventory.md
openspec validate live-bootstrap-source-chain
```

Result: all commands passed on 2026-04-27.
