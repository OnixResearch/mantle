# Implementation checkpoint

## Outcome

The native matcher now retains source-qualified Git dependencies without changing resolved package keys. Explicit source routes select the correct vendor directory. Missing, conflicting, and ambiguous vendor lookups reject instead of selecting another revision.

The CLI now runs the shared topology preflight before child-action runtime startup. A blocked plan produces its full receipt and no unit execution records. Child-action mode retains a nonzero exit status. Ordinary diagnostic topology mode keeps its existing receipt-based status behavior.

The initial rustc identity check remains before graph capture. It still requires valid authority, the canonical compiler path, and matching compiler bytes. The repair does not bypass that check.

## Current checks

The focused baseline passed: five native capture tests, two lockfile parser tests, and two vendor-root tests. The new regression suite then reproduced three failures on the original matcher.

The final native-planner run reported:

```text
test result: ok. 235 passed; 0 failed; 0 ignored; 0 measured; 2422 filtered out; finished in 3.93s
```

The full CLI suite reported:

```text
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.58s
```

First-party Clippy, Rust formatting, and the debug binary build passed. The machine contract check passed with 34 contracted and 68 classified surfaces. The generator changed only two OCI producer identities because the shared CLI source changed. It did not change schemas, fixtures, or consumer policy.

The architecture check reported zero findings with 13 negative fixtures. The earlier compatibility runs also passed 93 CLI-application tests and 88 source-built fixed-point tests, with three long proofs ignored.

The focused Nix rerun and full planning replay remain pending at this checkpoint. This checkpoint does not close I3, V2, V3, or V4.

## Preserved failed checks

- `regressions-before.log` records the original source-reference and vendor-binding failures.
- `cli-after.attempt1.log` records an invalid fixture assumption. Authority must supply rustc identity before graph capture. The corrected fixture supplies valid rustc authority and leaves another fixed executable absent.
- `cli-after.attempt2.log` records a trait import collision. A glob import brought `sha2::Digest` into scope and changed method resolution for `blake3::Hasher::finalize`. Explicit imports removed that collision.
- `cli-after.attempt3.log` records the fixture's missing executable mode and the host's unavailable `ld.lld`. The fixture now has explicit executable permissions. The full suite passed with the installed LLD 22.1.2 directory added to the validation PATH.
- `quality.attempt1.log` records the stale OCI producer binding. The repository generator refreshed the two producer identities.
- `nix.attempt1.log` records the earlier trait collision in the Nix snapshot. Its Tiger Style build completed, but the combined Nix command failed.

Logs retain command output except for trailing whitespace. No production dependency, lockfile, vendor config, failed proof input, or historical proof changed.

## Evidence correction

The original stage-1 receipt was empty. Zero derivations came from the separate planning-only replay, not from a unit count in the original stage-1 metadata. The copied diagnosis retains the earlier wording as historical evidence.

## Capability and repeatability

The immediate outcome is correct native admission of the two Git-source fixtures. The durable contribution is source-aware lookup and a preflight that preserves blocked-plan evidence. Mantle owns maintenance through the native planner and CLI suites. The new Nix checks make both regression groups repeatable.

No bootstrap restart, cache adoption, prefix restoration, promoted proof, release-alias change, archive, or publication occurred in this repair.
