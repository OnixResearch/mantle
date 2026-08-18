# Source-root provider materialization validation

Task-ID: I4
Covers: rust_package_planning.source_built_toolchain_closure
Date: 2026-05-31
Owner: agent

## Scope

This evidence completes the normalized source-root/seed provider materialization slice. It does **not** claim the end-to-end source-built toolchain closure fixed-point proof. The manifest explicitly records host provider-build tools as a trust note, and the source-built closure success claim remains blocked until the later fixed-point proof task succeeds.

## Inputs

Tracked manifest:

- `cairn/changes/source-built-toolchain-closure/evidence/source-root-provider-manifest.json`

Source artifacts were downloaded from responsive upstream mirrors and BLAKE3-pinned in the manifest:

- Linux headers: `https://mirrors.edge.kernel.org/pub/linux/kernel/v6.x/linux-6.6.32.tar.xz`
- musl: `https://codeload.github.com/ifduyue/musl/tar.gz/refs/tags/v1.2.5`
- binutils: `https://mirrors.kernel.org/gnu/binutils/binutils-2.41.tar.xz`
- GCC: `https://mirrors.kernel.org/gnu/gcc/gcc-10.5.0/gcc-10.5.0.tar.xz`
- GMP: `https://mirrors.kernel.org/gnu/gmp/gmp-6.2.1.tar.xz`
- MPFR: `https://mirrors.kernel.org/gnu/mpfr/mpfr-4.1.0.tar.xz`
- MPC: `https://mirrors.kernel.org/gnu/mpc/mpc-1.2.1.tar.gz`

Manifest digest recorded by Mantle:

```text
54ca36ac9eacc0193929cf20e457ccc065866839028146ad13d552346505f900
```

## Command

Pueue task: `172`

```text
nix shell nixpkgs#gnumake nixpkgs#gnutar nixpkgs#binutils nixpkgs#glibc.bin -c bash -lc 'set -euo pipefail
cd /home/brittonr/git/mantle
stamp=$(date -u +%Y%m%dT%H%M%SZ)
run_dir=/home/brittonr/git/mantle/.pi/source-root-provider-run-$stamp
mkdir -p "$run_dir/store" "$run_dir/state"
manifest=/home/brittonr/git/mantle/cairn/changes/source-built-toolchain-closure/evidence/source-root-provider-manifest.json
seed="$run_dir/seed.ncl"
log="$run_dir/bootstrap-source-root.log"
echo "run_dir=$run_dir"
echo "log=$log"
{
  echo "run_dir=$run_dir"
  echo "manifest=$manifest"
  echo "store=$run_dir/store"
  echo "state=$run_dir/state"
  echo "seed=$seed"
  echo "PATH=$PATH"
  echo "make=$(command -v make)"
  echo "tar=$(command -v tar)"
  echo "readelf=$(command -v readelf)"
  echo "ldd=$(command -v ldd || true)"
  echo "started=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  /home/brittonr/.cargo-target/debug/mantle --verbose --log-level info --store "$run_dir/store" --state-dir "$run_dir/state" bootstrap --source-root "$manifest" -o "$seed"
  echo "finished=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
} >"$log" 2>&1'
```

## Result

Pueue status:

```text
Task 172: Success, elapsed 4m 52s
```

Materialization log is copied to:

- `cairn/changes/source-built-toolchain-closure/evidence/source-root-provider-materialization.log`

Log digest:

```text
08538715e7d8b4ba7952d7a7e6fca83ee6b749466757b5062aea05205ffdd728
```

Output excerpt:

```text
[1/5] installing Linux kernel headers
[2/5] building binutils
[3/5] building GCC stage 1 (C compiler, no libc)
[4/5] building musl libc
[5/5] building GCC stage 2 (C + C++)
[normalize] producing seed contract layout
[self-contain] bundling host runtime libraries
Materialized source-root provider /home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain
  manifest_digest: 54ca36ac9eacc0193929cf20e457ccc065866839028146ad13d552346505f900
  output_digest: 70674e2e762f22ccf6512ad36d90bc07b608ae0495c216be300a60e61e74c41b
  expected_output_roles: 23
  dependency_trace_urls: 7
Wrote /home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/seed.ncl
finished=2026-05-31T23:19:47Z
```

Provider output digest:

```text
70674e2e762f22ccf6512ad36d90bc07b608ae0495c216be300a60e61e74c41b
```

Provider output path:

```text
/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain
```

Generated seed contract copy:

- `cairn/changes/source-built-toolchain-closure/evidence/source-root-provider-seed.ncl`
- BLAKE3: `3b9b61a3813e4d1c72b6ec6744cdec2bdbfc8a1c4d5de2d5c4d63ab79a6edc05`

Provider metadata copy:

- `cairn/changes/source-built-toolchain-closure/evidence/source-root-provider-output-provider.json`
- BLAKE3: `931200fdcbebb3bb6ad25212412fd71808df9a5dbed19d4efd71136611041f5e`

Machine summary:

- `cairn/changes/source-built-toolchain-closure/evidence/source-root-provider-materialization-summary.env`

## Smoke compile

Command:

```text
provider=.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain
work=.pi/source-root-provider-run-20260531T231455Z/smoke
mkdir -p "$work"
cat > "$work/hello.c" <<'EOF'
#include <stdio.h>
int main(void) { puts("mantle-source-root-smoke"); return 0; }
EOF
"$provider/bin/x86_64-linux-musl-gcc" -static "$work/hello.c" -o "$work/hello" 2>"$work/gcc.stderr"
"$work/hello" >"$work/hello.stdout"
file "$work/hello"
```

Output:

```text
mantle-source-root-smoke
.pi/source-root-provider-run-20260531T231455Z/smoke/hello: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), statically linked, not stripped
```

Copies:

- `cairn/changes/source-built-toolchain-closure/evidence/source-root-provider-smoke-stdout.txt`
- `cairn/changes/source-built-toolchain-closure/evidence/source-root-provider-smoke-stderr.txt`

## Repair note

The first positive runs exposed two provider-materialization environment issues before success:

- parent PATH needed a real `make`; the final command runs under a Nix shell carrying `gnumake`, `gnutar`, `binutils`, and `glibc.bin`.
- binutils/GCC source builds needed `MAKEINFO=true` passed to `make`, not only exported in the child environment, so generated info-page targets did not fail on hosts without `makeinfo`.

The code fix for the second issue is covered by focused `source_root_provider` tests and this positive materialization run.

## Decision

I4 may now be checked: a real normalized source-root provider was materialized from BLAKE3-pinned source archives, emitted the expected provider role trace, produced a source-root seed contract, and compiled a static smoke binary through the generated `x86_64-linux-musl-gcc`.

The end-to-end source-built toolchain closure fixed-point proof remains unchecked.
