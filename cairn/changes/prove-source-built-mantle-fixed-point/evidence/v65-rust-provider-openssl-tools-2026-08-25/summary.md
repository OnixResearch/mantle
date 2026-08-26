# V65 OpenSSL tool-path blocker

## Result

V65 reused V61's validated native prefix. It passed LLVM generation, temporary rustc construction, and produced proc-macro execution.

The Rust-provider stage reached 81 percent of the 393-item Cargo graph. OpenSSL source construction then failed.

The stage observed 49,471 actions. It matched 49,440 actions and denied 31 actions.

The denied actions were bounded path probes for:

- `pkg-config` and `pkgconf`.
- Source-built Perl.
- Source-built Make.
- One ambient `/bin/sh` request.

The OpenSSL generator failed when it could not launch its Perl translator.

## Repair

- The controlled target-tool directory now contains explicit `pkg-config` and `pkgconf` rejection stubs.
- It contains a source-built Perl alias.
- It contains a receipt-bound Make alias.
- The Make alias keeps generated recipes on the receipt-controlled shell.
- Missing optional tools now return deterministic unavailable results without path-search denials.

## Evidence

- `attempt-status.json`
- `source-built-fixed-point-plan.json`
- `native-prefix-reuse.json`
- `imported-native-provider-revalidation.json`
- `mrustc-first-stage-action-plan.json`
- `mrustc-first-stage-action-audit.json.gz`
- `mrustc-first-stage-action-reconciliation.json`
- `mrustc-first-stage-build.log.gz`
- `run-mrustc-first-stage.sh`
- `proof.log`
- `wrapper-status.txt`
