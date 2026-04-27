Task-ID: V2
Covers: bootstrap.binutils.tcc.chain

Status: blocked.

## Blocker

No `crunch` binary available in this environment:
- `~/.cargo-target/debug/crunch` does not exist
- Building crunch requires nightly Rust + clang + mold + pkg-config + openssl-dev + SNIX_BUILD_SANDBOX_SHELL
- This worktree is detached HEAD with no dev shell

## Prerequisite

V2 also depends on V1 hash correction: 39 of 46 source pins use flat archive
SHA-256 hashes instead of NAR/recursive hashes required by `crunch.fetchTarball`.
Each first build will fail with a hash mismatch and report the correct NAR hash.

## Required when unblocked

Build all 48 derivations in dependency order:
1. Early tcc epoch: bzip2-tcc, coreutils-5.0-tcc, oyacc-tcc, bash-2.05b-tcc
2. Libc boundary: tcc-musl-prep, musl-1.1.24-tcc, tcc-musl, musl-1.1.24-tcc-musl, tcc-musl-v2
3. Post-musl tools: grep-2.4-musl, sed-4.0.9-musl, bzip2-1.0.8-musl, m4-1.4.7-musl, heirloom-devtools, flex-2.5.11-musl, flex-2.6.4-musl, bison-2.3-musl, bison-3.4.1-musl
4. Utilities: diffutils-2.7-musl, coreutils-5.0-musl, coreutils-6.10-musl, gawk-3.0.4-musl
5. Perl ladder: perl-5.000 through perl-5.6.2
6. Autotools ladder: autoconf/automake interleaved (17 files)
7. libtool-2.2.4
8. binutils-tcc (final)

Record: command, exit status, output path, build duration for each.

Verified: 2026-04-27 (blocker recorded)
