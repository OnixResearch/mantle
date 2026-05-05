# GCC 4.0 libiberty configure/CPP boundary

Focused validation command:

```sh
timeout 520 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-progress-store" \
  bootstrap validate bootstrap/gcc-4.0.ncl \
  --evidence-dir "$PWD/.crunch-drain/gcc40-progress-evidence" --resume
```

Result:

- status: `build-failed`
- build_exit_code: `1`
- doctor_ok: `True`
- leakage_findings: `[{'source': 'stdout', 'needle': '/bin/', 'count': 2}]`

Boundary reached after slimming GCC inputs and switching from the hanging top-level configure to live-bootstrap-style per-subdirectory configure:

```text
store error: build: nonzero exit code: exit status: 1
configuring libiberty
checking whether to enable maintainer-specific portions of Makefiles... no
checking for makeinfo... no
checking for perl... no
checking build system type... x86_64-unknown-linux-gnu
checking host system type... x86_64-unknown-linux-gnu
checking for x86_64-unknown-linux-gnu-ar... /crunch/store/rzzs05bbvjbn8d7ng6vd9hjw8n6azq6r-binutils-2.30-tcc/bin/ar
checking for x86_64-unknown-linux-gnu-ranlib... /crunch/store/rzzs05bbvjbn8d7ng6vd9hjw8n6azq6r-binutils-2.30-tcc/bin/ranlib
checking for x86_64-unknown-linux-gnu-gcc... gcc40-cc
checking for suffix of object files...
checking whether we are using the GNU C compiler... no
checking whether gcc40-cc accepts -g... no
checking for gcc40-cc option to accept ANSI C... none needed
checking how to run the C preprocessor... gcc40-cpp
configure: error: C preprocessor "gcc40-cpp" fails sanity check
See `config.log' for more details.

```

The build now reaches `libiberty` configure deterministically and fails at the `gcc40-cpp` C preprocessor sanity check instead of hanging during root evaluation/top-level configure.
