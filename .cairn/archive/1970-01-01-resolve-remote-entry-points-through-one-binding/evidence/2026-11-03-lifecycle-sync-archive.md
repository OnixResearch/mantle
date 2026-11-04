# Spec sync and archive (2026-11-03)

The change was synced and archived with the pinned Cairn 0.1.0 policy
(`cairn-default`, receipt `34758b788bb92748806173fb0fccebec9ec3dadbb5c33051a50c401424a65179`).

## Commands and results

```text
cairn sync resolve-remote-entry-points-through-one-binding --root . --policy …
  dry plan hash 3dac75e1d9413b50b7da306b581cdd1d884704ecd7ea242cdcbf4ddc158be4e2
  execute: merged specs/remote-builds/spec.md into .cairn/specs/remote-builds/spec.md
  (added remote_builds.single_binding_resolution and remote_builds.binding_revocation)

cairn archive resolve-remote-entry-points-through-one-binding --root . --policy …
  dry plan hash ad5e065cffac369b78c6e9f8dffe4e8826d1f485e61ce025c9c35d88f7f3ac5c
  execute: moved the active change to
  .cairn/archive/1970-01-01-resolve-remote-entry-points-through-one-binding

cairn validate --root . --policy …
  exit 0, issues: [], change_issues: []
```

The archive directory carries the tool's epoch date rather than the run date; the
contents and evidence are the archived change.

## Notes

- T1.1, T4.1, and T4.2 are evidenced before the archive; T4.3 records this sync
  and archive.
- Readiness reported an unavailable completion observation; acceptance is not
  manufactured here, and the archived files retain their non-claims.
