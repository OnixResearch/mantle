# Protected transition v47 audit-ID rejection

The create-new v47 execution built second-musl and recorded all protected events. Final audit review rejected the compiler events because the validator expected a later duplicate authorization ID.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v47-musl-pass2-20260728`
- Pueue task: `3003`
- Exact error: `Audit("coreutils event for .../tcc-musl-stage/runtime/output/bin/tcc-0.9.27-musl did not match exact planned authority")`
- Runtime: 1468.19 seconds

The supervisor selected the first exact path-and-digest authorization: `planned:tcc-musl-materialization:exec:tcc-musl-smoke:tcc`. The rejected validator expected a later stage-specific duplicate. The retained plan, audit, and second-musl inventory are diagnostics only.
