# Baseline and search frame

## Goal

Select a real normalized source-built bootstrap provider, then prove the authenticated fresh-clone fixed point with that provider and zero undeclared source acquisition.

## Observable completion evidence

- The evaluated provider closure excludes `seed-legacy.ncl` and musl.cc.
- GCC 4.0, GCC 4.7, GCC 10, musl 1.2.5, and binutils 2.41 build real admitted artifacts from declared sources.
- Candidate-owned C, C++, assembler/linker/archive/object, static/dynamic libc, and libgcc smoke pass.
- Provider metadata binds source and output BLAKE3 identities.
- The selected provider drives stage0 and stage2 from authenticated source state with zero undeclared fetches and matching stage1/stage2 binaries.

## False completions

Static contract validation, executable-bit checks, version strings, host-assisted source-root materialization, pass1 wrappers that delegate to TinyCC, fabricated generator/backend objects, and contract-only downstream receipts do not complete the goal.

## Initial approach registry

| Family | Mechanism | State | Evidence / blocker | Next discriminating check |
|---|---|---|---|---|
| direct-chain | Promote existing `seed-full.ncl` unchanged | falsified | `gcc-4.0-native-boundary.json` is boundary-only; later receipts are contract-only | Real GCC 4.0 source build |
| source-root-reuse | Relabel `bootstrap --source-root` output | falsified | `docs/source-root-capabilities.md` records host compiler/linker/make/runtime influence and `full_source_bootstrap_eligible: false` | None without a new source-built lineage |
| upstream-faithful | Regenerate and build GCC 4.0 as upstream live-bootstrap does | active | Existing declared autotools/Bison/Flex/Perl stages make this materially different from the v22 wrapper probe | Build a separate upstream-faithful GCC 4.0 root and require real `cc1` |
| wrapper-frontier | Continue semantic slices in the pass1 wrapper | blocked | Twenty-two reductions still leave `c-parse.o` absent and almost all GCC backend objects bridged | Only materially new native compilation evidence can reopen |

## Baseline observations

`./scripts/check-bootstrap-blocker-inventory.sh` reported zero actionable findings only because it also reported 434 evidence-backed suppressions; this is not promotion evidence. `bootstrap/seed.ncl` still selects `seed-legacy.ncl`. Current GCC 4.0 evidence explicitly says `boundary-only`, `frontier-only`, and `does not prove native GCC 4.0 correctness`.

The local VibeThinker review endpoint failed to return a response, so the approach lenses above are serial/correlated rather than independent.

## Current implementation frontier

The isolated source-built chain has advanced beyond the initial wrapper frontier without changing `bootstrap/seed.ncl`:

- Clean upstream binutils 2.30 (`binutils-2.30-gcc-pass4-v16`) completed configure, build, install, and positive/negative runtime checks for assembler, linker, archive/index, ELF inspection, symbol/object transforms, size, strings, and strip.
- Conventional GCC/musl strengthening completed through GCC pass6 and musl pass5. These stages remain marked `NON_ADMISSION` because the later compiler ladder, dynamic runtime, normalized provider, and fresh-clone proof are unfinished.
- Genuine bootstrap generators now pass focused positive and negative runtime rails:
  - GNU grep 2.4 v8: `$HOME/.cache/mantle-full-source-20260718/gcc40-store/fprlsj3w9wm1sqq1xwcxlp7bq6k7l0j4-grep-2.4-gcc-v8`.
  - GNU M4 1.4.7 v2: `$HOME/.cache/mantle-full-source-20260718/gcc40-store/dbwryx6n4cv9yqcb71mdgsjd1i5bsnmf-m4-1.4.7-gcc-v2`.
  - GNU Bison 2.3 v6: `$HOME/.cache/mantle-full-source-20260718/gcc40-store/57dqxg2kjddkvjmwkvkr54nifqfs38l0-bison-2.3-gcc-v6`.
  - GNU Awk 3.0.4 v1: `$HOME/.cache/mantle-full-source-20260718/gcc40-store/495wd9lc1b20sxbcsjgb4dz1ma4r8qhz-gawk-3.0.4-gcc-v1`.
  - Configured Perl 5.6.2 v16: `$HOME/.cache/mantle-full-source-20260718/gcc40-store/2hc4r8n2hf5jy3r59v8jl94407c3awnh-perl-5.6.2-configured-gcc-v16`, including static `Data::Dumper`, `Fcntl`, `IO`, `POSIX`, and `File::Glob`, plus separately generated `Errno.pm`.
- The complete Autoconf ladder reached 2.52, 2.53, 2.54, 2.55, 2.57, 2.59, 2.61, 2.64, and 2.69. Each root performs byte-identical repeat generation, executes a positive generated `configure`, and rejects `m4_fatal([mantle-negative])`. The final output is `$HOME/.cache/mantle-full-source-20260718/gcc40-store/vib6d3qkbqxdafd8lzdf6nk29qlvaafv-autoconf-2.69-gcc-v1`.
- The complete Automake ladder reached 1.6.3, 1.7, 1.7.8, 1.8.5, 1.9.6, 1.10.3, 1.11.2, and 1.15.1. Each root generates and builds a positive project, checks repeat `Makefile.in` equality, and rejects an undefined conditional. Durable early outputs and logs are:
  - `$HOME/.cache/mantle-full-source-20260718/gcc40-store/5i4bfpgyzdinx5zw4kbpsg5yqk591cdk-automake-1.6.3-gcc-v4`, log `x24gw84xl3r08xh3mgrdyy51la8n9jiz-automake-1.6.3-gcc-v4.drv.log`.
  - `$HOME/.cache/mantle-full-source-20260718/gcc40-store/9yhdcmmdjn2jk39vfighx4jf8x2vh0l2-automake-1.7-gcc-v4`, log `10n93wgx2w156l3wsdlnv58jw9jdamqm-automake-1.7-gcc-v4.drv.log`.
  - Final `$HOME/.cache/mantle-full-source-20260718/gcc40-store/h86ri5h9ady5d7w29lin4h0x38fbqzx6-automake-1.15.1-gcc-v1`.
- Parser/scanner bootstrapping now crosses a tested Heirloom bridge and Flex 2.5.36 bridge, then genuinely regenerates Flex 2.5.11 and 2.6.4. Bison 3.4.1 v2 regenerates its parser and three scanners with Bison v1/Flex 2.6.4, executes a generated arithmetic parser (`19 + 23 = 42`), and rejects a malformed grammar. Its output is `$HOME/.cache/mantle-full-source-20260718/gcc40-store/sm4gk4wj1baxpwjv27nkr8bx7h1b3lpv-bison-3.4.1-gcc-v2`; log `hzyh6ssk9nvn46sv7rh04nilw34bg5jc-bison-3.4.1-gcc-v2.drv.log` records success.
- Regenerated GCC 4.0.4 C/C++ v6 is `$HOME/.cache/mantle-full-source-20260718/gcc40-store/2fc9jz82fy4jzln73s399vx3vfi37sqp-gcc-4.0.4-musl-cxx-v6`. It regenerates `gcc/configure` with the documented exact Autoconf 2.59, `gcc/gengtype-yacc.c/.h` with Bison 3.4.1, `gcc/gengtype-lex.c` with Flex 2.6.4, and the built C frontend `c-parse.c` from `c-parse.in`. The bounded Bison-3 compatibility correction restores the historical no-argument `YYLEX` macro used directly by GCC 4.0 semantic actions. Log `pj6nc1mrs3sy0a3kcx063gs7b3f9927g-gcc-4.0.4-musl-cxx-v6.drv.log` records successful static C and C++ execution, exceptions, RTTI, allocation, `dynamic_cast`, malformed C/C++ rejection, and a copied-tree relocation compile/run using explicit assembler, linker, libc, CRT, and libgcc override seams.
- GCC 4.0.4's own prerequisites document an exact Autoconf 2.13 requirement for top-level `configure.in`, while `gcc/` and most subdirectories require exact 2.59. The v6 root therefore retains the authenticated shipped top-level `configure` and names source-built Autoconf 2.13 as an open predecessor instead of mis-regenerating that file with 2.59.
- The GCC 4.7 math prerequisites now precede the compiler instead of forming the earlier cyclic graph. GCC-4.0-built static GMP 6.2.1, MPFR 4.1.0, and MPC 1.2.1 are respectively `$HOME/.cache/mantle-full-source-20260718/gcc40-store/rl1pii3ncwhz2gwajis6w7000klgqa5k-gmp-6.2.1-gcc40-v1`, `0xya3489xzld6riahmliij1hbzrnfm1f-mpfr-4.1.0-gcc40-v1`, and `hiw34cxynjhy14p4vb9afdxx6fl5z5n6-mpc-1.2.1-gcc40-v1`. Each root validates its archive, executes a semantic program, and rejects malformed C without accepting an output object.
- Input-addressed GCC 4.7.4 C/C++ v1 completed at `$HOME/.cache/mantle-full-source-20260718/gcc40-store/0l0rj25bnyypy6kf4mswmlwsildkmxc1-gcc-4.7.4-musl-gcc40-v1`, keeping GCC's embedded configured prefix equal to its published logical output path. Success log `snncymj4f59wnwr0npdgmmfgsz98y0l6-gcc-4.7.4-musl-gcc40-v1.drv.log` records real `cc1`, `cc1plus`, `libgcc.a`, `libstdc++.a`, and `libsupc++.a` construction; static C execution; C++11 `auto`, allocation, virtual dispatch, RTTI/`dynamic_cast`, and exceptions; malformed-C++ rejection with no object; and copied-tree C compile/run through explicit assembler, linker, libc, libm, libgcc, and CRT seams. The build uses GCC's bundled zlib, with its archive selected only for compiler construction.
- Input-addressed GCC 10.5.0 v1 completed at `$HOME/.cache/mantle-full-source-20260718/gcc40-store/crv06yhczdf5qb3q1la5z6y1k0qmjhrh-gcc-10.5.0-musl-gcc47-v1`. Success log `nqsdjq8bhqxj3vkfdvxvsb1ixa3n8j19-gcc-10.5.0-musl-gcc47-v1.drv.log` records real `cc1`, `cc1plus`, `libgcc.a`, `libstdc++.a`, and `libsupc++.a`; static C execution and ELF inspection; C++17 `constexpr`, final classes, allocation, RTTI/`dynamic_cast`, and exceptions; malformed C and C++ rejection with no objects; and copied-tree C++17 execution. This pass intentionally uses release-generated GCC 10 sources and the predecessor musl sysroot.
- Input-addressed musl 1.2.5 v1 completed at `$HOME/.cache/mantle-full-source-20260718/gcc40-store/193p3d1x7j8dl9dng3aan44q1gc2h8ky-musl-1.2.5-gcc10-v1`. Success log `x254d6mnnn65f05k9wiy4mp60lrqnibp-musl-1.2.5-gcc10-v1.drv.log` records nonempty `libc.a`, ELF `libc.so`, `ld-musl-x86_64.so.1`, and the new `crt1.o`/`Scrt1.o`/`rcrt1.o`/`crti.o`/`crtn.o`; semantic semaphore/stdio runtime execution in both static and dynamic executables; an interpreter-path assertion against the published musl output; malformed-C rejection; and explicit links that use the newly built CRT objects rather than GCC 10's predecessor-sysroot defaults.

- The input-addressed final GCC 10.5.0 cycle completed at `$HOME/.cache/mantle-full-source-20260718/gcc40-store/lfy0n3xn4vvwvc473wcjbis9rni65b9n-gcc-10.5.0-musl-final-v1`. Success log `y8vp3ai8ar8s4w65jxzw4hm8p9rr1x1c-gcc-10.5.0-musl-final-v1.drv.log` records installed `libgcc.a`, `libgcc_eh.a`, `libgcc_s.so.1`, `libstdc++.a`, and `libstdc++.so.6.0.28`; static and dynamic C++17 execution with allocation, exceptions, RTTI, and `dynamic_cast`; the musl 1.2.5 interpreter and final-GCC runtime search paths; malformed-C++ rejection with no accepted output; and copied-tree C++17 compile/run through explicit assembler, linker, libc, CRT, static/shared libgcc, and exception-runtime seams.
- Input-addressed binutils 2.41 completed at `$HOME/.cache/mantle-full-source-20260718/gcc40-store/f99n9wql2dz4409x4iwzx01l41zh0qp2-binutils-2.41-gcc10-v1`. Success log `j7gw4sm39pg428lak4ipzk7akpaxcgdb-binutils-2.41-gcc10-v1.drv.log` records positive assembler/linker execution, archive/index behavior, ELF/symbol/object inspection and transforms across `as`, `ld`, `ar`, `ranlib`, `readelf`, `nm`, `objcopy`, `objdump`, `size`, `strings`, and `strip`; malformed-assembly rejection; undefined-symbol rejection with no linker output; and copied-tree assembler/linker execution.

These outputs prove their bounded runtime surfaces only. The Perl/Autoconf/Automake/Flex/Bison, regenerated-GCC, math-library, GCC 4.7, GCC 10, musl 1.2.5, and binutils 2.41 roots are currently selected through state-pinned `/mantle/store` paths bound into a private diagnostic `/nix/store` overlay. GCC 4.7, GCC 10, and binutils 2.41 still compile authenticated release-generated sources rather than locally regenerated sources. The roots retain earlier source-chain inputs transitively and do not prove derivational closure, provider closure purity, top-level GCC Autoconf 2.13 regeneration, provider normalization, or fixed-point admission.

The next discriminating work is source-built Autoconf 2.13, replacement of state-pinned generator/compiler/math inputs with a tractable derivational graph, regeneration of the GCC 4.7/GCC 10/binutils 2.41 sources, and normalized-provider closure/purity evidence.
