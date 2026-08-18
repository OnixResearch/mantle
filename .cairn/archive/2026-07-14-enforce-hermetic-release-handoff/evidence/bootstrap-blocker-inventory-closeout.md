# Bootstrap blocker inventory closeout

Recorded: 2026-07-14

## Baseline

The pre-change report-only rail passed its own positive and negative matcher
self-tests and reported:

```text
bootstrap blocker inventory: 40 findings across 4 classes,
396 evidence-backed suppressions, 0 promotion claims, enforce=false
```

The actionable classes were 13 `bridge-output`, 10
`compiler-runtime-crash-boundary`, 14 `placeholder-deferred`, and 3
`prerequisite-gated-evidence` findings.

## Resolution

The repair did not delete fail-closed diagnostic branches or relax the clean
inventory requirement.

- `bootstrap/rust-source-plan.ncl` now identifies the implemented
  `mantle bootstrap rust-source-provider` route and leaves materialization,
  smoke, self-build, and fixed-point authority with the archived
  `source-built-rust-seed-closure` receipts.
- The diagnostic-boundary inventory and its checker now bind the current i386
  diagnostic markers and current GCC 4.0 frontier locations. This restores the
  intended evidence-backed distinction between diagnostic negative paths and
  production blockers; stale line/marker drift no longer duplicates checked
  boundaries as new findings.
- `spike-i386-mes-runtime-layout.ncl` now publishes its validated Mes runtime
  tree. `spike-i386-tcc27-make-pass1.ncl` imports that derivation, validates the
  successful real-runtime summary and required files, and uses the runtime
  paths for the next TinyCC/Make diagnostic stage.
- `bootstrap/gcc.ncl` now excludes nondeterministic generated manpages from the
  bootstrap compiler contract instead of overwriting them with content that
  could be mistaken for real GCC documentation.
- The Nix format check now derives its first-party package scope from the
  reviewable `[workspace.metadata.tigerstyle].default_scope` list. Vendored
  workspaces remain outside the first-party format gate, as required by the
  repo-local validation notes. Existing first-party format drift exposed by
  that corrected scope was formatted.

## Focused validation

The local enforced inventory and its matcher self-tests pass:

```text
$ ./scripts/check-bootstrap-blocker-inventory.sh --self-test
bootstrap blocker inventory: 0 findings across 0 classes,
434 evidence-backed suppressions, 0 promotion claims, enforce=true
```

The same gate passes through the Nix check surface:

```text
$ nix build .#checks.x86_64-linux.bootstrap-blocker-inventory \
    --option secret-key-files '' --option builders '' -L
PASS
```

The corrected first-party format check passes:

```text
$ nix build .#checks.x86_64-linux.fmt \
    --option secret-key-files '' --option builders '' -L
PASS
```

A fresh bounded bootstrap validation of `bootstrap/gcc.ncl` passed all warmups,
the GCC build, and the host-leakage scan:

```text
bootstrap validation: Passed
target: bootstrap/gcc.ncl
build exit code: 0
```

The output was
`/tmp/mantle-gcc-manpages-store-v2/15z4208hdghbz555d7ghc6vhdvwx4s5h-gcc`.
Its `share/` tree contains no `man/` directory; the compiler output keeps real
compiler artifacts without fabricated documentation.

The fresh i386 sibling build validated and consumed the real Mes runtime
handoff. An initial rerun exposed that the older Make probe had retained flags
that reintroduced the already-repaired object-emission segfault. Aligning its
patches and compile flags with the proven handoff moved the frontier:

```text
runtime_handoff_validate rc=0
tcc27_compile_object rc=0
tcc27_object_exists rc=0
tcc27_link rc=1
```

The current diagnostic summary records `blocked_step=tcc27_link`; the TinyCC
linker reports the validated object path as not found even though the explicit
file check succeeds. This is bounded diagnostic progress only. It does not
claim a TinyCC 0.9.27 executable, GNU Make smoke, or production route switch.

The final bounded Cairn lifecycle rerun passed repository validation and the
proposal, design, and tasks gates for `enforce-hermetic-release-handoff` with no
issues. The exact Cairn invocation first reproduced the unavailable host signing
key, so the passing rerun disabled only host signing and remote builders.

## Remaining flake frontier

Both the exact and bounded flake commands now pass the bootstrap inventory and
format checks, then stop at the independently configured Tiger Style consumer
check:

```text
$ nix flake check
error: could not compile `crunch-release-core` (lib) due to 418 previous errors

$ nix flake check --option secret-key-files '' --option builders ''
error: could not compile `crunch-release-core` (lib) due to 418 previous errors
```

The failures span the explicitly deny-level first-party Tiger Style policy,
including assertion density, panic/unwrap handling, bounded growth, arithmetic,
interface, and conversion findings in multiple crates. Mantle's
`dylint.toml` explicitly enables every Tiger Style lint at `deny`, and the
workspace metadata explicitly selects all first-party packages. Pinning an
older lint engine, demoting levels, disabling families, or shrinking the scope
would weaken that checked policy and was not done.

Therefore the original 40-finding bootstrap inventory blocker is closed, but
the lifecycle task remains unchecked. No spec sync or archive is justified
until a separately scoped workspace-wide Tiger Style debt drain makes the
mandatory flake check pass. The previously observed unavailable host signing
key also remains a host-level exact-check constraint after code-level checks are
clean.
