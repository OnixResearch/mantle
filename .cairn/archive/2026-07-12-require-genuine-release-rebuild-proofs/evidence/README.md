# Genuine release rebuild proof evidence

## Outcome

The focused implementation rail passed on the isolated implementation branch and again after main-branch integration. Exact branch command output is recorded in [`validation.txt`](validation.txt).

Observed focused results:

- `crunch-release-core`: 186 passed.
- `release_reproduce_`: 13 passed, including source-derived positive runs and
  direct-copy/alias/symlink/hardlink/output-authority negatives.
- `release_verify_`: 54 passed.
- `release_nix_witness_`: 6 passed.
- Bootstrap-parity `self_build_`: 13 passed, including partial-on-genuine-v2 and
  blocked-on-legacy/missing authority behavior.
- Receipt checker, summary renderer, production recipe, and operator-guide
  self/drift checks passed.
- Focused Rust formatting, `crunch-release-core` Clippy with warnings denied,
  Mantle binary check, Cairn validation, and proposal/design/tasks gates passed.
- Main-branch pueue task `341` reran release core, bootstrap core, Cairn handoff, release-evidence, release CLI handoff, and all 13 `release_reproduce_` tests as one successful chain in an isolated target. Task `346` reran Cairn validation and proposal/design/tasks gates; the final tasks gate remained `PASS` with no issues.

## Decision evidence

- The v2 receipt binds the accepted content descriptor and authority-plan BLAKE3.
- Only separately materialized regular-file capabilities enter proof sandboxes;
  the complete release bundle is absent.
- Post-integration adversarial review found and closed a measurement-to-copy replacement window: each materialized input is now checked against the originally measured size and BLAKE3 identity, with a same-size replacement negative test.
- Cairn handoff bytes introduced by the concurrent release change are measured into the atomic publication plan before staging and checked again from the staged manifest before commit; no handoff file bypasses the no-partial-publication boundary.
- Published target bytes, content-identical aliases, hardlinks, symlinks, prior
  proof outputs, and ordinary outputs are non-authorities.
- Release verification, standalone checking, summaries, Nix witnesses, and
  bootstrap parity all reject legacy or incomplete genuine-rebuild evidence.
- The checked bootstrap parity descriptor remains legacy v1, so the current
  project-root row is intentionally blocked until a genuine v2 release run
  refreshes that evidence.

## Bounded non-claims

No full production rebuild, compiler/verifier soundness proof, self-hosting proof,
full-bootstrap reproducibility proof, Guix parity proof, or StageX parity proof was
run or claimed by this focused validation.

## Accepted-spec synchronization

Pueue task `356` dry-ran and executed Cairn sync without blockers, but Cairn left both accepted specs unchanged and the new requirement IDs absent. The reviewed deltas were therefore materialized manually. Pueue task `372` proved byte-for-byte suffix equality:

- `cairn/specs/build-correctness/spec.md`: 3202 bytes, BLAKE3 `014ad3c047d21fda494ba8234a663ec55e7f7ee31018282568803f7414b91264`.
- `cairn/specs/release-provenance/spec.md`: 1942 bytes, BLAKE3 `e773a1b112f0086b12d6e1ee9e30353172af4b39741bd15e673db926f5f15738`.

## Archive and post-archive validation

Pueue task `389` dry-ran and executed archive without blockers at `cairn/archive/2026-07-12-require-genuine-release-rebuild-proofs`. Pueue task `393` re-proved both accepted suffix identities above and captured final post-archive validation:

```text
{
  "change_issues": [],
  "changes": 7,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 33,
  "valid": true
}
```
