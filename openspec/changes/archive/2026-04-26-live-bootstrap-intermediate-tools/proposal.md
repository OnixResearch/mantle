# Proposal: live-bootstrap intermediate tools

## Problem

The live-bootstrap seed chain (`live-bootstrap-seed-chain`) scaffolded
placeholder derivations for the ~30 intermediate packages between tinycc-0.9.27
and gcc-4.0.4.  These placeholders cannot actually build because the
intermediate tool chain (sed, patch, gawk, m4, flex, bison, diffutils, coreutils,
etc.) is missing.  Without them, binutils-2.30 and gcc-4.0.4 cannot be compiled,
blocking the entire from-source GCC version ladder.

## Proposed solution

Implement each intermediate tool derivation following the proven build order from
the live-bootstrap project (`fosslinux/live-bootstrap` step files).  The
approximate chain between tcc-0.9.27 + make-3.82 and gcc-4.0.4 is:

1.  sed-4.0.9 (tcc)
2.  patch-2.5.9 (tcc)
3.  gzip-1.2.4 (tcc)
4.  tar-1.12 (tcc)
5.  gawk-3.0.4 (tcc)
6.  diffutils-2.7 (tcc)
7.  bash-2.05b (tcc)
8.  coreutils-5.0 (tcc)
9.  musl-1.1.24 (tcc) -- first real libc
10. m4-1.4.7 (tcc + musl)
11. flex-2.5.11 (tcc + musl)
12. bison-2.3 (tcc + musl)
13. grep-2.4 (tcc + musl)
14. binutils-2.30 (tcc + musl + all above)
15. gcc-4.0.4 (tcc + musl + binutils-2.30 + all above)

Each derivation adapts the corresponding `steps/<pkg>/pass1.kaem` (or
`.sh`) from live-bootstrap.  Build scripts use only tools from the prior chain
stage -- no host leakage.

## Scope

- ~15-20 new Nickel derivation files under `bootstrap/`
- Replace the existing placeholder `bootstrap/binutils-tcc.ncl` and
  `bootstrap/gcc-4.0.ncl` with functional derivations
- Validation: each derivation builds and its output tool works
- Does NOT touch the GCC version ladder above 4.0.4 (that stays in the parent
  change or a future sub-change)

## Relationship to parent change

This is a sub-change of `live-bootstrap-seed-chain`.  Completing it unblocks
tasks I8, I9, and transitively V1-V7 in the parent.

## Risks

- Some live-bootstrap steps use bash-specific syntax that the initial kaem/dash
  shell cannot handle; may need careful adaptation
- Source tarball availability: older GNU sources may have moved; mirror URLs
  may be needed
- Build order may require slight reordering once real builds are attempted
