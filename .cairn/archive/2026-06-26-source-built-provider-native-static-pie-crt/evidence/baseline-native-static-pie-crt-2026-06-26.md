# Baseline: provider native static-PIE CRT blocker

Task-ID: I1
Covers: r[rust_package_planning.source_built_toolchain_closure.native_static_pie_crt]

## Source evidence

The current frontier was recorded by archived change `cairn/archive/2026-06-26-source-built-provider-mantle-bin-warning-frontier/evidence/provider-rerun-next-blocker-2026-06-26.md`.

Proof bundle:

```text
/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-mantle-bin-warning-frontier-2026-06-26
```

Parsed receipt:

```text
receipt: /home/brittonr/git/mantle-source-built-rust-provider-fixed-point-mantle-bin-warning-frontier-2026-06-26/stage1/receipt.json
status: blocked
fixed_point: false
stage1.execution_status: blocked
stage1.status_code: 0
stage1.unit_count: 679
stage1.failed_unit_count: 1
blocker_class: rustc-failed
failed_package: path+native#mantle@0.1.0
failed_target: crunch
failed_kind: bin
failed_unit_id: native:b766e54ac96cfcf5c4b1b5f6fda714924cbc7d046bb286d74e270959753b401b:path+native#mantle@0.1.0:crunch:bin:build
```

The failed derivation uses the receipt-bound guard linker alias:

```text
-C linker=/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-mantle-bin-warning-frontier-2026-06-26/stage1/cargo-guard-bin/cc
```

The generated alias currently downgrades static-PIE mode but does not replace `rcrt1.o`:

```text
#!/bin/sh
remaining=$#
while [ "$remaining" -gt 0 ]; do
  arg=$1
  shift
  case "$arg" in
    -static-pie) set -- "$@" -static ;;
    *) set -- "$@" "$arg" ;;
  esac
  remaining=$((remaining - 1))
done
exec '/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain/bin/x86_64-linux-musl-gcc' -L'/home/brittonr/git/mantle-source-built-rust-provider-fixed-point-mantle-bin-warning-frontier-2026-06-26/stage1/cargo-guard-bin/.toolchain-runtime' "$@"
```

Relevant blocker diagnostics:

```text
relocation R_X86_64_32 against `.bss.maplock' can not be used when making a PIE object; recompile with -fPIE
failed to set dynamic section sizes: bad value
collect2: error: ld returned 1 exit status
error: aborting due to 1 previous error
```

## Baseline assessment

The previous work moved the frontier past warning noise. The remaining failure is a receipt-bound source-root musl native link issue: the current alias normalizes the link mode but does not normalize the static-PIE startup object to the declared non-PIE CRT object.
