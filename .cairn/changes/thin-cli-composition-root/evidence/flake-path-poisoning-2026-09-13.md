# Sandbox flake: PATH poisoning vs generated stage1 script

Recorded 2026-09-13 while draining `thin-cli-composition-root` (I3/F5 slices).

## Observation

`nix flake check -L` fails intermittently at `checks.x86_64-linux.crunch` on the
same test:

```
mantle> thread 'rust_source_provider::tests::rustc_stage1_provider_candidate_rejects_tampered_artifact'
        panicked at src/rust_source_provider.rs:10362:85:
mantle> called `Result::unwrap()` on an `Err` value:
        Build("rustc stage1 build failed with status exit status: 127;
        log=/build/.tmpPmaXg6/scratch/rustc-stage1-build.log;
        tail=\"/build/.tmpPmaXg6/scratch/run-rustc-stage1.sh: line 16: mkdir: not found\\n\"")
```

The same test passes locally in the dev shell (`cargo test -p mantle --bin
mantle rustc_stage1_provider_candidate_rejects_tampered_artifact` → 3 passed),
and the attribute passed on retry earlier the same day. Run history for the
attribute in one session: fail, fail (different victims: `remote_transfer_*`),
pass on retry, fail again with this test. One run reported 2435 passed, 1
failed.

## Cause

Two facts together:

1. `src/self_build.rs` tests mutate the shared process environment
   (`std::env::set_var("PATH", ...)`, including `PATH = ""`). They serialize
   among themselves with the file-local `PATH_MUTEX`, but nothing serializes
   them against *other* tests that spawn PATH-dependent children.
2. `src/rust_source_provider.rs` generates the stage1 build script in
   production code (`run-rustc-stage1.sh`) and that script invokes utilities
   such as `mkdir` through the ambient `PATH`.

When a `self_build` test holds an emptied PATH while the stage1 test runs the
script, the script cannot find `mkdir` and the build exits 127. That is why the
victim rotates with libtest's scheduling and why it never reproduces in a
focused single-test run.

## Fix options

- **Production robustness (preferred for the artifact):** the generated stage1
  script must not depend on an inherited PATH for its utilities. Give the script
  an explicit tool search path, or call absolute utility paths resolved when the
  plan is written. A sandboxed bootstrap script inheriting a sibling process's
  environment is fragile beyond tests.
- **Test isolation (root cause):** stop mutating the shared process environment
  in `self_build` tests; pass the intended PATH to the specific child command
  (`Command::env("PATH", ...)`) as the repo already requires for host-env
  leakage tests elsewhere.

Either fix needs its own verified commit; both were left undone in this slice
because the change under way was a filesystem-authority extraction.

## Non-claims

This note records a flake and its cause. It does not claim the full flake check
is green for the revision that produced this evidence, and it does not claim
either fix is implemented.
