# Broad validation

## Workspace check

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Checking fuse-backend-rs v0.12.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/vendor/fuse-backend-rs)
    Checking crunch-nar v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-nar)
    Checking crunch-eval v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-eval)
    Checking crunch-wasm-component v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-wasm-component)
    Checking crunch-eval-budget-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-eval-budget-core)
    Checking crunch-kernelscript-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-kernelscript-core)
    Checking crunch-hardware-simulation-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-hardware-simulation-core)
    Checking crunch-spacewasm-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-spacewasm-core)
   Compiling nix-compat v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/vendor/nix-compat)
    Checking nix-compat-derive v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/vendor/nix-compat-derive)
    Checking crunch-shell v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-shell)
    Checking crunch-rust-cache-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-rust-cache-core)
    Checking crunch-bootstrap-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-bootstrap-core)
    Checking crunch-attestation v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-attestation)
    Checking crunch-delta-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-delta-core)
    Checking crunch-action-result-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-action-result-core)
    Checking crunch-glue v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-glue)
    Checking crunch-release-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-release-core)
    Checking crunch-shell-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-shell-core)
    Checking mantlepkgs-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/mantlepkgs-core)
    Checking crunch-composition-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-composition-core)
    Checking snix-tracing v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/vendor/snix-tracing)
    Checking crunch-project-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-project-core)
    Checking crunch-gc-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-gc-core)
    Checking crunch-wasm-component-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-wasm-component-core)
    Checking crunch-attestation-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-attestation-core)
    Checking crunch-project v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-project)
    Checking crunch-overlay-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-overlay-core)
    Checking crunch-repair-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-repair-core)
    Checking mantle-portable-client-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/mantle-portable-client-core)
    Checking crunch-spacewasm v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-spacewasm)
    Checking snix-castore v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/vendor/snix-castore)
    Checking crunch-hardware-simulation v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-hardware-simulation)
    Checking crunch-kernelscript-adapter v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-kernelscript-adapter)
    Checking snix-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/vendor/snix-store)
    Checking snix-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/vendor/snix-build)
    Checking crunch-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-store)
    Checking crunch-delta v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-delta)
    Checking crunch-rust-cache v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-rust-cache)
    Checking crunch-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-build)
    Checking crunch-rustc-wrapper v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-rustc-wrapper)
    Checking crunch-pipeline v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-pipeline)
    Checking mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809)
error[E0063]: missing field `resource_metrics` in initializer of `BenchmarkResult`
   --> examples/benchmark_lazy_eval.rs:128:22
    |
128 |         results.push(BenchmarkResult {
    |                      ^^^^^^^^^^^^^^^ missing `resource_metrics`

error[E0063]: missing field `resource_metrics` in initializer of `BenchmarkResult`
   --> examples/benchmark_lazy_eval.rs:166:22
    |
166 |         results.push(BenchmarkResult {
    |                      ^^^^^^^^^^^^^^^ missing `resource_metrics`

error[E0063]: missing field `resource_metrics` in initializer of `BenchmarkResult`
   --> examples/benchmark_lazy_eval.rs:221:22
    |
221 |         results.push(BenchmarkResult {
    |                      ^^^^^^^^^^^^^^^ missing `resource_metrics`

error[E0063]: missing field `resource_metrics` in initializer of `BenchmarkResult`
   --> examples/benchmark_lazy_eval.rs:254:22
    |
254 |         results.push(BenchmarkResult {
    |                      ^^^^^^^^^^^^^^^ missing `resource_metrics`

For more information about this error, try `rustc --explain E0063`.
error: could not compile `mantle` (example "benchmark_lazy_eval") due to 4 previous errors
warning: build failed, waiting for other jobs to finish...

exit_status=101
```

## Nix flake evaluation

```text
evaluating flake...
checking flake output 'packages'...
checking derivation packages.x86_64-linux.default...
derivation evaluated to /nix/store/vl94mqz16r2m4cgw0v8030m0f83nq80k-mantle-0.1.0.drv
checking derivation packages.x86_64-linux.crunch...
derivation evaluated to /nix/store/vl94mqz16r2m4cgw0v8030m0f83nq80k-mantle-0.1.0.drv
checking derivation packages.x86_64-linux.ast-grep-toolchain...
derivation evaluated to /nix/store/hwz43xm09gs5v38a4pfyc2dhs5n9h1pj-mantle-ast-grep-toolchain-0.42.1.drv
checking derivation packages.x86_64-linux.ast-grep-package-identity...
derivation evaluated to /nix/store/6f1xqfw3dc0hca3x412qqcgb4cca9d99-mantle-ast-grep-package-identity-smoke.drv
checking derivation packages.x86_64-linux.wasm-component-toolchain...
derivation evaluated to /nix/store/jip7bl880iii0mzaywr3jzfdmfpkb1gb-mantle-wasm-component-toolchain-v1.drv
checking derivation packages.x86_64-linux.wasm-component-toolchain-identity...
derivation evaluated to /nix/store/kqy8bp2zq2zzjnpxkqxaslph51gailg8-mantle-wasm-component-toolchain-identity.drv
checking derivation packages.x86_64-linux.wasm-component-toolchain-compatibility...
derivation evaluated to /nix/store/ja7yc1yr65jdwd66ixf1lr1flpy632mb-mantle-wasm-component-toolchain-compatibility.drv
checking derivation packages.x86_64-linux.mantle-transcript-quality...
fatal: repository 'https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git/' not found
warning: could not get HEAD ref for repository 'https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git'; using expired cached ref 'refs/heads/main'
derivation evaluated to /nix/store/1qkg7j9n7kjnlgkswc04z48j1ys0isfi-mantle-transcript-quality-nextest-0.1.0.drv
checking derivation packages.x86_64-linux.release-nix-witness-quality...
derivation evaluated to /nix/store/4nk61zw9dfad8ajaxz404z8pvmrb8xm7-crunch-release-nix-witness-quality-nextest-0.1.0.drv
checking derivation packages.x86_64-linux.check-store-retention-policy...
derivation evaluated to /nix/store/5xc5yw2mnizcln8b4f52fmgxb861s4z7-check-store-retention-policy.drv
checking derivation packages.x86_64-linux.check-store-overlay-policy...
derivation evaluated to /nix/store/358z0ygz6qib9gpgs79sp2llm0d3q2ls-check-store-overlay-policy.drv
checking derivation packages.x86_64-linux.oci-distribution-registry...
derivation evaluated to /nix/store/7h1b5w2s2gqhb0dgmiydw4xcl2blxnnv-distribution-3.1.0.drv
checking derivation packages.x86_64-linux.rustc-wrapper...
derivation evaluated to /nix/store/ky90w7wdlfv1bsr4qq2r6is2caj1vxh9-mantle-rustc-wrapper-0.1.0.drv
checking derivation packages.x86_64-linux.kernelscript-compiler...
derivation evaluated to /nix/store/z1dps3ryzhmn8iyx6jnpfyxfqlnzqpzq-ocaml5.2.1-kernelscript-0.1.2.drv
checking derivation packages.x86_64-linux.kernelscript-core-adapter...
derivation evaluated to /nix/store/4z4fqz624mmlgfyqvfrswxa9r9h614mr-crunch-kernelscript-adapter-0.1.0.drv
checking derivation packages.x86_64-linux.kernelscript-production...
derivation evaluated to /nix/store/gg2xf7n3mmjywxf8p25bgp6aic6mp8na-mantle-kernelscript-production-artifacts.drv
checking derivation packages.x86_64-linux.kernelscript-production-cohort...
derivation evaluated to /nix/store/dxzvi99rvi9n22rjiqn1yzlv0mv0z5pp-mantle-kernelscript-production-cohort.drv
checking derivation packages.x86_64-linux.kernelscript-production-shell...
derivation evaluated to /nix/store/3sja6hm11zcs46pf2zyq12hnd9q6kg50-mantle-kernelscript-production.drv
checking derivation packages.x86_64-linux.kernelscript-production-runtime-check...
derivation evaluated to /nix/store/fkkwrjksp4yihzwx8gwxzglcp253902d-vm-test-run-mantle-kernelscript-production-runtime.drv
checking derivation packages.x86_64-linux.spacewasm-reference-bundle...
derivation evaluated to /nix/store/l6r3ig41jrdqfivsvnl88z0w7gq1kfj0-mantle-spacewasm-reference-bundle-e24cf09355a90497148eb5029fdb8e3400bd63e3.drv
checking derivation packages.x86_64-linux.spacewasm-reference-bundler...
derivation evaluated to /nix/store/ipyyk9zxh3kbvk9c1znx79jcdrrvvg38-mantle-spacewasm-reference-bundler-0.1.0.drv
checking derivation packages.x86_64-linux.spacewasm-reference-evidence...
derivation evaluated to /nix/store/y3py5q02d27fq2nywdidz9zj3vcfz3zm-spacewasm-e24cf09355a90497148eb5029fdb8e3400bd63e3-evidence.drv
checking derivation packages.x86_64-linux.spacewasm-reference-rust-toolchain...
derivation evaluated to /nix/store/ib3maa2zj4d9ikmlx4x15xaiz9vxh55r-rust-minimal-1.91.1.drv
checking flake output 'apps'...
checking app 'apps.x86_64-linux.tigerstyle'...
warning: app 'apps.x86_64-linux.tigerstyle' lacks attribute 'meta'
checking flake output 'checks'...
checking derivation checks.x86_64-linux.tigerstyle...
derivation evaluated to /nix/store/01bj5avm0qyjpvj66bpccsxk7ja8zw36-tigerstyle-consumer-check.drv
checking derivation checks.x86_64-linux.crunch...
derivation evaluated to /nix/store/vl94mqz16r2m4cgw0v8030m0f83nq80k-mantle-0.1.0.drv
checking derivation checks.x86_64-linux.ast-grep-package-identity...
derivation evaluated to /nix/store/6f1xqfw3dc0hca3x412qqcgb4cca9d99-mantle-ast-grep-package-identity-smoke.drv
checking derivation checks.x86_64-linux.wasm-component-toolchain-identity...
derivation evaluated to /nix/store/kqy8bp2zq2zzjnpxkqxaslph51gailg8-mantle-wasm-component-toolchain-identity.drv
checking derivation checks.x86_64-linux.wasm-component-toolchain-compatibility...
derivation evaluated to /nix/store/ja7yc1yr65jdwd66ixf1lr1flpy632mb-mantle-wasm-component-toolchain-compatibility.drv
checking derivation checks.x86_64-linux.mantle-transcript-quality...
derivation evaluated to /nix/store/1qkg7j9n7kjnlgkswc04z48j1ys0isfi-mantle-transcript-quality-nextest-0.1.0.drv
checking derivation checks.x86_64-linux.release-nix-witness-quality...
derivation evaluated to /nix/store/4nk61zw9dfad8ajaxz404z8pvmrb8xm7-crunch-release-nix-witness-quality-nextest-0.1.0.drv
checking derivation checks.x86_64-linux.kernelscript-production...
derivation evaluated to /nix/store/a6scha39wvvj0smnk9r7w82g0b2ii8m2-mantle-kernelscript-production-structural-check.drv
checking derivation checks.x86_64-linux.spacewasm-reference-bundle...
derivation evaluated to /nix/store/l6r3ig41jrdqfivsvnl88z0w7gq1kfj0-mantle-spacewasm-reference-bundle-e24cf09355a90497148eb5029fdb8e3400bd63e3.drv
checking derivation checks.x86_64-linux.bootstrap-blocker-inventory...
derivation evaluated to /nix/store/pqdfxccv0kgxv3j5qrafi4fsv5gar24b-bootstrap-blocker-inventory.drv
checking derivation checks.x86_64-linux.nickel-export-core-pin...
derivation evaluated to /nix/store/zpiv229a33pa5vsj35rji5dsbwjmb0cx-mantle-nickel-export-core-pin.drv
checking derivation checks.x86_64-linux.content-bound-requirement-source-closure...
derivation evaluated to /nix/store/sll5v7jwxxxgr5x2z7j1imfjp7k4cgw1-mantle-content-bound-requirement-source-closure.drv
checking derivation checks.x86_64-linux.content-bound-requirement-evidence...
derivation evaluated to /nix/store/f43p0855fydamnx4nvzhrs465czss85x-mantle-content-bound-requirement-evidence.drv
checking derivation checks.x86_64-linux.store-retention-policy...
derivation evaluated to /nix/store/jfv4f78fm4g3zgrfd771ng8ial06nq24-mantle-store-retention-policy.drv
checking derivation checks.x86_64-linux.store-overlay-policy...
derivation evaluated to /nix/store/5icrrz7czjyd04wa6lp2n4p8v1v9zl14-mantle-store-overlay-policy.drv
checking derivation checks.x86_64-linux.release-determinism-quality...
derivation evaluated to /nix/store/mh5s646v4gxx43l4gfnxpj85hlx83vkv-crunch-release-determinism-quality-nextest-0.1.0.drv
checking derivation checks.x86_64-linux.artifact-auth-radicle-cutover...
derivation evaluated to /nix/store/p549xbhl124r4bpyg4lki1n5ji68ydxr-mantle-artifact-auth-radicle-cutover.drv
checking derivation checks.x86_64-linux.durable-file-publication-adoption...
derivation evaluated to /nix/store/vp4qiyqf711y6ihxpp3mdb8hci6g4617-mantle-durable-file-publication-adoption.drv
checking derivation checks.x86_64-linux.nextest...
derivation evaluated to /nix/store/bk1zdfpvlk1cjs4lwjm7q9d116lln2qa-mantle-nextest-0.1.0.drv
checking derivation checks.x86_64-linux.clippy...
derivation evaluated to /nix/store/phwxslbfcwf6kbslab4yg8ah5as8qy7b-mantle-clippy-0.1.0.drv
checking derivation checks.x86_64-linux.fmt...
derivation evaluated to /nix/store/kspiz8ywvd6zypzqb6np8pj09kskdvwm-mantle-fmt-0.1.0.drv
checking derivation checks.x86_64-linux.spacewasm-reference-profile...
derivation evaluated to /nix/store/n9c40y8a14cabyv75rpssccf8a9k7sxr-spacewasm-reference-profile-export.drv
checking derivation checks.x86_64-linux.spacewasm-reference-host-library...
derivation evaluated to /nix/store/j6r8vlls8h06h61gq52fgcg1b7593bib-spacewasm-e24cf09355a90497148eb5029fdb8e3400bd63e3-host-library.drv
checking derivation checks.x86_64-linux.spacewasm-reference-wasm-library...
derivation evaluated to /nix/store/f7ghaq9l9f7jfvkipzq3mfqzpj716riv-spacewasm-e24cf09355a90497148eb5029fdb8e3400bd63e3-wasm-library.drv
checking derivation checks.x86_64-linux.spacewasm-reference-host-runner...
derivation evaluated to /nix/store/xv1a7k2m125vv3003ii3gf2ymwygly2x-spacewasm-e24cf09355a90497148eb5029fdb8e3400bd63e3-diagnostic-runner.drv
checking derivation checks.x86_64-linux.spacewasm-reference-upstream-unit-tests...
derivation evaluated to /nix/store/pqdh5xf8y0k76h7pjvaskxbq289crkwg-spacewasm-e24cf09355a90497148eb5029fdb8e3400bd63e3-unit-tests.drv
checking derivation checks.x86_64-linux.spacewasm-reference-spectest-address...
derivation evaluated to /nix/store/b5pnd8m8xmnfgfgdxchr639gvgcl7yp9-spacewasm-e24cf09355a90497148eb5029fdb8e3400bd63e3-spectest-address.drv
checking derivation checks.x86_64-linux.spacewasm-reference-fixtures...
derivation evaluated to /nix/store/77xymn526iq43h5ggnckzw6id3znjk5x-spacewasm-e24cf09355a90497148eb5029fdb8e3400bd63e3-fixture-report.drv
checking derivation checks.x86_64-linux.spacewasm-reference-negative...
derivation evaluated to /nix/store/5r89n7f0qdvyp4335j6jaanx1y3vcmc1-mantle-spacewasm-reference-negative-check.drv
checking flake output 'devShells'...
checking derivation devShells.x86_64-linux.default...
derivation evaluated to /nix/store/63x0lqdzsl6yqmvg0k99b1sanjpcy0pr-nix-shell.drv
all checks passed!
warning: The check omitted these incompatible systems: aarch64-darwin, aarch64-linux, x86_64-darwin
Use '--all-systems' to check all.

exit_status=0
```

## Root-package formatting rail

```text
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/benchmark_compare.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/benchmark_eval_backends.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/benchmark_eval_smoke.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/benchmark_lazy_eval.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/benchmark_scheduler_priority.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/benchmark_suite.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/hardware_simulation_plan.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/picolibc_compare.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/projects/delta-substitution/demo.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/projects/release-witness-handoff/demo.rs"
[example (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/projects/shared-action-result-roundtrip/publish.rs"
[lib (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/src/lib.rs"
[bin (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/src/main.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/attest_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/audit_support.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/benchmark_harness.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/bootstrap_eval.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/bootstrap_parity_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/bootstrap_validate_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/cargo_free_self_build_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/cargo_import_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/composition_root_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/evaluator_budget_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/example_projects.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/examples_build.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/examples_eval.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/examples_inventory.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/examples_workflow_gallery.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/foreign_import_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/freshness_offline_rail.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/identity_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/integration.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/integration_build.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/kernel_bundle_oci_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/kernel_bundle_oci_registry_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/kernelscript_experiment.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/lock_importer_offline_rail.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/machine_schema_contracts.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/nix_free_demo_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/offline_build_runbook_docs.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/offline_cargo_project.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/operator_diagnostics.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/pin_import_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/project_build_smoke.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/project_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/project_refresh_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/release_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/remote_credentials_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/remote_rail_offline_rail.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/remote_stdio_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/remote_transfer_production.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/removed_system_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/retention_offline_rail.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/rust_compatibility_rail.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/rust_plan_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/scheduling_policy.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/self_hosting.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/smoke.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/source_bundle_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/source_bundle_hydration_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/stdlib_tests.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/store_archive_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/store_gc_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/transcript_cli.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/trust_policy_offline_rail.rs"
[test (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/wasm_component_cli.rs"
[bin (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tools/generate_operator_command_contract.rs"
[bin (2024)] "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tools/generate_portable_platform_profiles.rs"
rustfmt --edition 2024 --check /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/benchmark_compare.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/benchmark_eval_backends.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/benchmark_eval_smoke.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/benchmark_lazy_eval.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/benchmark_scheduler_priority.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/benchmark_suite.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/hardware_simulation_plan.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/picolibc_compare.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/projects/delta-substitution/demo.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/projects/release-witness-handoff/demo.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/examples/projects/shared-action-result-roundtrip/publish.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/src/lib.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/src/main.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/attest_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/audit_support.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/benchmark_harness.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/bootstrap_eval.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/bootstrap_parity_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/bootstrap_validate_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/cargo_free_self_build_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/cargo_import_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/composition_root_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/evaluator_budget_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/example_projects.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/examples_build.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/examples_eval.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/examples_inventory.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/examples_workflow_gallery.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/foreign_import_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/freshness_offline_rail.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/identity_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/integration.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/integration_build.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/kernel_bundle_oci_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/kernel_bundle_oci_registry_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/kernelscript_experiment.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/lock_importer_offline_rail.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/machine_schema_contracts.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/nix_free_demo_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/offline_build_runbook_docs.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/offline_cargo_project.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/operator_diagnostics.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/pin_import_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/project_build_smoke.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/project_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/project_refresh_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/release_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/remote_credentials_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/remote_rail_offline_rail.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/remote_stdio_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/remote_transfer_production.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/removed_system_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/retention_offline_rail.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/rust_compatibility_rail.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/rust_plan_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/scheduling_policy.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/self_hosting.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/smoke.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/source_bundle_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/source_bundle_hydration_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/stdlib_tests.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/store_archive_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/store_gc_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/transcript_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/trust_policy_offline_rail.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tests/wasm_component_cli.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tools/generate_operator_command_contract.rs /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/tools/generate_portable_platform_profiles.rs
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/src/source_built_fixed_point_shell.rs:18:
 use crate::errors::RunError;
 use crate::full_source_rust_binding_shell::FullSourceRustHostToolMaterializationRequest;
 use crate::native_toolchain_closure::NativeToolchainClosureOptions;
[31m-use crate::source_built_fixed_point::plan_source_built_fixed_point;
(B[m use crate::source_built_fixed_point::InitialOutputAuthorityState;
 use crate::source_built_fixed_point::ProofHermeticityMode;
[32m+use crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX;
(B[m use crate::source_built_fixed_point::SourceAuthorityInput;
 use crate::source_built_fixed_point::SourceAuthorityRole;
 use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/src/source_built_fixed_point_shell.rs:28:
 use crate::source_built_fixed_point::SourceBuiltFixedPointPolicies;
 use crate::source_built_fixed_point::SourceBuiltFixedPointResourceBounds;
 use crate::source_built_fixed_point::SourceContentKind;
[31m-use crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::dev_provider_cache_key;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::evaluate_fast_fail;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::evaluate_provider_cache_lookup;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::validate_stage_marker;
(B[m[32m+use crate::source_built_fixed_point::plan_source_built_fixed_point;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::DEV_CACHE_ENTRY_FILE;
(B[m use crate::source_built_fixed_point_dev_cache::DevCachePolicies;
 use crate::source_built_fixed_point_dev_cache::DevProviderCacheEntry;
 use crate::source_built_fixed_point_dev_cache::DevProviderCacheLookup;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/src/source_built_fixed_point_shell.rs:39:
[32m+use crate::source_built_fixed_point_dev_cache::FAST_FAIL_SCHEMA;
(B[m use crate::source_built_fixed_point_dev_cache::FastFailDecision;
 use crate::source_built_fixed_point_dev_cache::StageCompletionMarker;
 use crate::source_built_fixed_point_dev_cache::StageMarkerValidation;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/src/source_built_fixed_point_shell.rs:42:
[31m-use crate::source_built_fixed_point_dev_cache::DEV_CACHE_ENTRY_FILE;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::FAST_FAIL_SCHEMA;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::dev_provider_cache_key;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::evaluate_fast_fail;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::evaluate_provider_cache_lookup;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::validate_stage_marker;
(B[m[32m+use crate::source_bundle::SourceBuiltFixedPointProfileRecords;
(B[m[32m+use crate::source_bundle::SourceRecord;
(B[m use crate::source_bundle::assemble_source_bundle;
 use crate::source_bundle::materialize_source_record_payload;
 use crate::source_bundle::source_built_fixed_point_profile_records;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/src/source_built_fixed_point_shell.rs:47:
[31m-use crate::source_bundle::SourceBuiltFixedPointProfileRecords;
(B[m[31m-use crate::source_bundle::SourceRecord;
(B[m use crate::stagex_provider::StagexProviderRequest;
 use crate::stagex_transition::StagexTransitionRequest;


exit_status=1
```

## Tiger Style consumer rail

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Checking serde v1.0.228
    Checking tinystr v0.8.3
    Checking potential_utf v0.1.5
    Checking futures-executor v0.3.32
    Checking n0-error v0.1.3
    Checking clap-verbosity-flag v3.0.4
    Checking typed-builder v0.22.0
    Checking crunch-overlay-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-overlay-core)
    Checking crunch-gc-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-gc-core)
    Checking crunch-repair-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-repair-core)
    Checking ouroboros v0.18.5
    Checking spki v0.7.3
   Compiling maybe-async v0.2.10
   Compiling owo-colors v4.3.0
   Compiling litrs v1.0.0
    Checking rustc-demangle v0.1.28
   Compiling cap-primitives v4.0.2
   Compiling linkme-impl v0.3.37
    Checking icu_collections v2.2.0
    Checking futures v0.3.32
   Compiling num-bigint-dig v0.8.6
    Checking rand_core v0.9.5
    Checking io-extras v0.19.0
    Checking signal-hook-mio v0.2.5
error: function `revalidate_overlay` has 0 assertion(s) in 22 lines (need 2+)
   --> crates/crunch-overlay-core/src/lib.rs:222:103
    |
222 |   pub fn revalidate_overlay(expected: &OverlayPlan, observed: &OverlayPlan) -> Result<(), OverlayError> {
    |  _______________________________________________________________________________________________________^
223 | |     if expected.policy_id != observed.policy_id || expected.logical_prefix != observed.logical_prefix {
224 | |         return Err(OverlayError::CompositionDrift);
...   |
242 | |     Ok(())
243 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density
    = note: `-D tigerstyle::assertion-density` implied by `-D assertion-density`
    = help: to override `-D assertion-density` add `#[allow(tigerstyle::assertion_density)]`

error: `report` looks like a quantity but has no unit suffix
   --> crates/crunch-gc-core/src/retention.rs:333:13
    |
333 |     let mut report = UsageReport {
    |             ^^^^^^
    |
    = help: Tiger Style (Unit Suffixes): put the unit in the name of numeric quantities Example: timeout_ms
    = note: structured suggestion applicability: HasPlaceholders
    = note: see Tiger Style guide: Unit Suffixes
    = note: `-D tigerstyle::numeric-units` implied by `-D numeric-units`
    = help: to override `-D numeric-units` add `#[allow(tigerstyle::numeric_units)]`
help: rename with an explicit unit suffix like `_ms`, `_secs`, or `_bytes`
    |
333 |     let mut report_<unit> = UsageReport {
    |                   +++++++

error: function `validate_policy` has 0 assertion(s) in 25 lines (need 2+)
   --> crates/crunch-overlay-core/src/lib.rs:245:72
    |
245 |   fn validate_policy(policy: &OverlayPolicy) -> Result<(), OverlayError> {
    |  ________________________________________________________________________^
246 | |     if policy.policy_id.is_empty() {
247 | |         return Err(OverlayError::EmptyPolicyId);
...   |
268 | |     Ok(())
269 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `validate_observation` has 0 assertion(s) in 41 lines (need 2+)
   --> crates/crunch-overlay-core/src/lib.rs:308:31
    |
308 |   ) -> Result<(), OverlayError> {
    |  _______________________________^
309 | |     if observation.declaration_index != expected_index {
310 | |         return Err(OverlayError::InvalidDeclarationIndex {
311 | |             expected: expected_index,
...   |
347 | |     validate_generation_members(policy, observation)
348 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `validate_generation_members` has 0 assertion(s) in 41 lines (need 2+)
   --> crates/crunch-overlay-core/src/lib.rs:350:115
    |
350 |   fn validate_generation_members(policy: &OverlayPolicy, observation: &BaseObservation) -> Result<(), OverlayError> {
    |  ___________________________________________________________________________________________________________________^
351 | |     if observation.members.is_empty() {
352 | |         return Err(OverlayError::EmptyGeneration {
353 | |             declaration_index: observation.declaration_index,
...   |
389 | |     Ok(())
390 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `descriptor_for_observation` has 0 assertion(s) in 22 lines (need 2+)
   --> crates/crunch-overlay-core/src/lib.rs:403:105
    |
403 |   fn descriptor_for_observation(mut observation: BaseObservation) -> Result<BaseDescriptor, OverlayError> {
    |  _________________________________________________________________________________________________________^
404 | |     observation.members.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
405 | |     let observed_bytes = observation.members.iter().try_fold(0_u64, |total, member| {
406 | |         total.checked_add(member.bytes).ok_or(OverlayError::GenerationBytesOverflow {
...   |
423 | |     })
424 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `validate_policy` has 0 assertion(s) in 25 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:351:76
    |
351 |   fn validate_policy(policy: &RetentionPolicy) -> Result<(), RetentionError> {
    |  ____________________________________________________________________________^
352 | |     if policy.policy_id.is_empty() {
353 | |         return Err(RetentionError::EmptyPolicyId);
...   |
374 | |     Ok(())
375 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density
    = note: `-D tigerstyle::assertion-density` implied by `-D assertion-density`
    = help: to override `-D assertion-density` add `#[allow(tigerstyle::assertion_density)]`

error: function `normalize_roots` has 0 assertion(s) in 37 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:380:49
    |
380 |   ) -> Result<Vec<RetentionRoot>, RetentionError> {
    |  _________________________________________________^
381 | |     for root in &roots {
382 | |         validate_path_id(&root.path_id)?;
383 | |         if root.owner_scope.is_empty() {
...   |
415 | |     Ok(roots)
416 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `generation_ranks` has 0 assertion(s) in 38 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:418:97
    |
418 |   fn generation_ranks(roots: &[RetentionRoot]) -> Result<BTreeMap<String, usize>, RetentionError> {
    |  _________________________________________________________________________________________________^
419 | |     let mut groups = BTreeMap::<(RootClass, String, String, String), Vec<(u64, String)>>::new();
420 | |     for root in roots {
421 | |         if !matches!(root.class, RootClass::ProjectOutputGeneration | RootClass::ProjectSourceGeneration) {
...   |
455 | |     Ok(ranks)
456 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `decide_root` has 0 assertion(s) in 35 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:463:48
    |
463 |   ) -> Result<RetentionDecision, RetentionError> {
    |  ________________________________________________^
464 | |     let (mut disposition, mut reason) = match root.class {
465 | |         RootClass::ExplicitPin => (RetentionDisposition::Keep, RetentionReason::ExplicitPin),
466 | |         RootClass::ProjectOutputGeneration => generation_decision(
...   |
496 | |     })
497 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `lease_decision` has 0 assertion(s) in 31 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:522:70
    |
522 |   ) -> Result<(RetentionDisposition, RetentionReason), RetentionError> {
    |  ______________________________________________________________________^
523 | |     let Some(lease) = &root.lease else {
524 | |         return Err(RetentionError::MissingLease {
525 | |             path_id: root.path_id.clone(),
...   |
552 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: `lease_duration_is_unsafe` looks like a quantity but has no unit suffix
   --> crates/crunch-gc-core/src/retention.rs:535:9
    |
535 |     let lease_duration_is_unsafe = lease
    |         ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: Tiger Style (Unit Suffixes): put the unit in the name of numeric quantities Example: timeout_ms
    = note: structured suggestion applicability: HasPlaceholders
    = note: see Tiger Style guide: Unit Suffixes
help: rename with an explicit unit suffix like `_ms`, `_secs`, or `_bytes`
    |
535 |     let lease_duration_is_unsafe_<unit> = lease
    |                                 +++++++

error: function `retention_plan_id` has 0 assertion(s) in 37 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:558:46
    |
558 |   ) -> Result<RetentionPlanId, RetentionError> {
    |  ______________________________________________^
559 | |     let mut hasher = blake3::Hasher::new();
560 | |     hasher.update(RETENTION_PLAN_DOMAIN);
561 | |     hash_string(&mut hasher, &policy.policy_id)?;
...   |
593 | |     Ok(RetentionPlanId(*hasher.finalize().as_bytes()))
594 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `hash_root` has 0 assertion(s) in 26 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:596:95
    |
596 |   fn hash_root(hasher: &mut blake3::Hasher, root: &RetentionRoot) -> Result<(), RetentionError> {
    |  _______________________________________________________________________________________________^
597 | |     hash_string(hasher, &root.path_id)?;
598 | |     hash_string(hasher, root.class.as_str())?;
599 | |     hash_string(hasher, &root.owner_scope)?;
...   |
620 | |     Ok(())
621 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `validate_objects` has 0 assertion(s) in 37 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:693:29
    |
693 |   ) -> Result<(), UsageError> {
    |  _____________________________^
694 | |     for pair in objects.windows(2) {
695 | |         if pair[0].object_id == pair[1].object_id {
696 | |             return Err(UsageError::DuplicateObject {
...   |
728 | |     Ok(())
729 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `accumulate_usage_object` has 0 assertion(s) in 35 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:736:29
    |
736 |   ) -> Result<(), UsageError> {
    |  _____________________________^
737 | |     let Some(bytes) = object.bytes else {
738 | |         report.unknown_object_count = report.unknown_object_count.checked_add(1).ok_or(UsageError::ByteOverflow)?;
739 | |         for root_id in &object.retaining_root_ids {
...   |
769 | |     Ok(())
770 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `usage_class` has 0 assertion(s) in 23 lines (need 2+)
   --> crates/crunch-gc-core/src/retention.rs:780:119
    |
780 |   fn usage_class(decisions: &BTreeMap<&str, &RetentionDecision>, root_ids: &[String]) -> Result<UsageClass, UsageError> {
    |  _______________________________________________________________________________________________________________________^
781 | |     if root_ids.is_empty() {
782 | |         return Ok(UsageClass::Reclaimable);
...   |
801 | |     Ok(UsageClass::Reclaimable)
802 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `validate_links` has 0 assertion(s) in 27 lines (need 2+)
   --> crates/crunch-gc-core/src/lib.rs:371:30
    |
371 |   ) -> Result<(), GcPlanError> {
    |  ______________________________^
372 | |     for root in roots {
373 | |         if !entries_by_id.contains_key(root.as_str()) {
374 | |             return Err(GcPlanError::MissingRoot { path_id: root.clone() });
...   |
396 | |     Ok(())
397 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

error: function `compute_retaining_roots` has 0 assertion(s) in 27 lines (need 2+)
   --> crates/crunch-gc-core/src/lib.rs:424:49
    |
424 |   ) -> Result<Vec<GcRetainingRoots>, GcPlanError> {
    |  _________________________________________________^
425 | |     let mut roots_by_path = BTreeMap::<String, BTreeSet<String>>::new();
426 | |     for root in roots {
427 | |         let mut visited = BTreeSet::new();
...   |
449 | |         .collect())
450 | | }
    | |_^
    |
    = help: Tiger Style (Assertion Density): add enough assertions to make local invariants visible near the logic they defend
    = note: see Tiger Style guide: Assertion Density

    Checking num-iter v0.1.46
    Checking fs-set-times v0.20.3
    Checking icu_locale_core v2.2.0
error: boolean binding `bounds_are_valid` should have a predicate prefix
   --> crates/crunch-overlay-core/src/lib.rs:252:9
    |
252 |     let bounds_are_valid = policy.max_base_layers > 0
    |         ^^^^^^^^^^^^^^^^
    |
    = help: Tiger Style (Positive Predicates): name booleans as predicates so call sites read like questions Example: let is_ready = status == Ready;
    = note: structured suggestion applicability: MaybeIncorrect
    = note: see Tiger Style guide: Positive Predicates
    = note: `-D tigerstyle::bool-naming` implied by `-D bool-naming`
    = help: to override `-D bool-naming` add `#[allow(tigerstyle::bool_naming)]`
help: rename binding to `is_bounds_are_valid`
    |
252 |     let is_bounds_are_valid = policy.max_base_layers > 0
    |         +++

error: boolean binding `valid` should have a predicate prefix
   --> crates/crunch-overlay-core/src/lib.rs:272:9
    |
272 |     let valid = logical_prefix.starts_with('/')
    |         ^^^^^
    |
    = help: Tiger Style (Positive Predicates): name booleans as predicates so call sites read like questions Example: let is_ready = status == Ready;
    = note: structured suggestion applicability: MaybeIncorrect
    = note: see Tiger Style guide: Positive Predicates
help: rename binding to `is_valid`
    |
272 |     let is_valid = logical_prefix.starts_with('/')
    |         +++

    Checking rustix-linux-procfs v0.1.1
   Compiling syn v3.0.3
error: could not compile `crunch-overlay-core` (lib) due to 7 previous errors
warning: build failed, waiting for other jobs to finish...
error: boolean binding `owner_is_eligible` should have a predicate prefix
   --> crates/crunch-gc-core/src/retention.rs:389:13
    |
389 |         let owner_is_eligible = policy
    |             ^^^^^^^^^^^^^^^^^
    |
    = help: Tiger Style (Positive Predicates): name booleans as predicates so call sites read like questions Example: let is_ready = status == Ready;
    = note: structured suggestion applicability: MaybeIncorrect
    = note: see Tiger Style guide: Positive Predicates
    = note: `-D tigerstyle::bool-naming` implied by `-D bool-naming`
    = help: to override `-D bool-naming` add `#[allow(tigerstyle::bool_naming)]`
help: rename binding to `is_owner_is_eligible`
    |
389 |         let is_owner_is_eligible = policy
    |             +++

error: collection grows in loop without a prior reservation or explicit local bound
   --> crates/crunch-gc-core/src/retention.rs:451:13
    |
451 |             ranks.insert(path_id.clone(), current_rank);
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: Tiger Style (Bounded Resource Growth): reserve collection capacity or add an explicit local length bound before growing it in a loop Example: let mut items = Vec::with_capacity(limit);
    = note: see Tiger Style guide: Bounded Resource Growth
    = note: `-D tigerstyle::unbounded-collection-growth` implied by `-D unbounded-collection-growth`
    = help: to override `-D unbounded-collection-growth` add `#[allow(tigerstyle::unbounded_collection_growth)]`

error: boolean binding `lease_duration_is_unsafe` should have a predicate prefix
   --> crates/crunch-gc-core/src/retention.rs:535:9
    |
535 |     let lease_duration_is_unsafe = lease
    |         ^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: Tiger Style (Positive Predicates): name booleans as predicates so call sites read like questions Example: let is_ready = status == Ready;
    = note: structured suggestion applicability: MaybeIncorrect
    = note: see Tiger Style guide: Positive Predicates
help: rename binding to `is_lease_duration_is_unsafe`
    |
535 |     let is_lease_duration_is_unsafe = lease
    |         +++

error: 2 consecutive `u64` parameters are easy to swap by accident
   --> crates/crunch-gc-core/src/retention.rs:804:1
    |
804 | / fn checked_add(left: u64, right: u64) -> Result<u64, UsageError> {
805 | |     left.checked_add(right).ok_or(UsageError::ByteOverflow)
806 | | }
    | |_^
    |
    = help: Tiger Style (Explicit Interfaces): replace same-type parameter clusters with named fields or an options struct Example: fn connect(opts: ConnectOptions) -> ...
    = note: see Tiger Style guide: Explicit Interfaces
    = note: `-D tigerstyle::ambiguous-params` implied by `-D ambiguous-params`
    = help: to override `-D ambiguous-params` add `#[allow(tigerstyle::ambiguous_params)]`

error: could not compile `crunch-gc-core` (lib) due to 18 previous errors

exit_status=1
```

## Workspace check after benchmark result repair

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Checking crunch-eval-budget-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809/crates/crunch-eval-budget-core)
    Checking mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-enforce-evaluator-resource-budgets-20260809)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 37.74s

exit_status=0
```

## Review

- The first workspace check found four missing `resource_metrics` fields in `examples/benchmark_lazy_eval.rs`.
- The repaired workspace check passed with all targets.
- Nix flake evaluation passed.
- The root formatting rail still reports only the pre-existing `src/source_built_fixed_point_shell.rs` difference. This change does not modify that excluded fixed-point surface.
- The workspace Tiger Style rail still stops in the pre-existing `crunch-gc-core` findings. Focused Tiger Style Clippy for the evaluator core and Mantle evaluator surfaces passed in `focused-validation.md`.
- No broad failure was hidden or converted into an evaluator-budget claim.
