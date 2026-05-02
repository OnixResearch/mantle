# V1 supplemental attempt: mes prerequisite audit build

Task-ID: V1

Covers: `bootstrap.i386-live-bootstrap-spike.runtime-proof`

## Status

Incomplete/hung supplemental audit. This was not the main i386 proof target; it was launched to see whether the existing `bootstrap/mes.ncl` output could be inspected for i386 Mes runtime artifacts before implementing a sibling i386 TinyCC path.

## Command

```sh
rm -rf .crunch-drain/i386-mes-audit-store .crunch-drain/i386-mes-audit-state
mkdir -p .crunch-drain/i386-mes-audit-store .crunch-drain/i386-mes-audit-state
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute   --store "$PWD/.crunch-drain/i386-mes-audit-store"   --state-dir "$PWD/.crunch-drain/i386-mes-audit-state"   bootstrap/mes.ncl
```

## Result

The process produced only initial state setup/redb repair lines and no derivation log after roughly eleven minutes, then was killed. The state directory should be treated as contaminated and not reused.

Captured process output:

```text
Generated signing key: crunch-britton-desktop-1 (/home/brittonr/git/crunch/crunch/.crunch-drain/i386-mes-audit-state/signing-key)
2026-05-01T23:43:07.930241Z  WARN redb::db: Database "FileBackend { lock_supported: true, file: File { fd: 10, path: "/home/brittonr/git/crunch/crunch/.crunch-drain/i386-mes-audit-state/directories.redb", read: true, write: true } }" not shutdown cleanly. Repairing
2026-05-01T23:43:07.940076Z  WARN redb::db: Database "FileBackend { lock_supported: true, file: File { fd: 11, path: "/home/brittonr/git/crunch/crunch/.crunch-drain/i386-mes-audit-state/pathinfo.redb", read: true, write: true } }" not shutdown cleanly. Repairing
```

## Interpretation

This does not change the spike decision. Native i386 execution is proven by `V1-proof-attempt.md`; the remaining blocker is constructing the i386 Mes/TinyCC lineage itself. A future retry should use a fresh state/store and preferably target a dedicated i386 Mes/TinyCC proof derivation rather than waiting on the existing amd64 `mes.ncl` audit path.
