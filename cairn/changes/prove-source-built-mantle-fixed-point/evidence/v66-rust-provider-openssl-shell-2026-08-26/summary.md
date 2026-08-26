# V66 OpenSSL ambient shell blocker

## Result

V66 reused V61's validated native prefix and passed every earlier Rust-provider action boundary.

It reached 81 percent of the Cargo graph. The stage observed 49,422 actions and matched 49,421 actions.

One action was denied: OpenSSL's Perl assembly pipeline requested ambient `/bin/sh`.

The assembly translator failed after that denial. No other action was denied.

## Repair

The selected full-source bootstrap now configures vendored OpenSSL with `no-asm`.

This removes Perl assembly pipes from the bootstrap closure. The ordinary portable OpenSSL C implementation remains available.

The change is full-source-only. Compatibility mode remains unchanged.

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
