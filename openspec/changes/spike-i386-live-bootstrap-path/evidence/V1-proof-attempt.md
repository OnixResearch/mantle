# V1 Proof attempt: native i386 execution under Crunch sandbox

Task-ID: V1

Covers: `bootstrap.i386-live-bootstrap-spike.runtime-proof`

## Status

Partial proof passed. This evidence proves the selected execution model from I3: Crunch's normal bwrap-backed derivation builder can execute a native i386 static ELF on this amd64 host. It does **not** yet prove the full `tcc-0.9.27 -> make-3.82 pass1` i386 chain, so V1 remains unchecked until a Make 3.82 proof target passes or records a concrete blocker.

## Implemented proof target

Added `bootstrap/spike-i386-native-smoke.ncl`.

The derivation writes a 96-byte i386 static ELF whose program body exits via `int 0x80` with status `42`, executes it inside the Crunch build sandbox, requires `rc=42`, and installs the executable plus a result marker.

## Command

```sh
rm -rf .crunch-drain/i386-smoke-store .crunch-drain/i386-smoke-state
mkdir -p .crunch-drain/i386-smoke-store .crunch-drain/i386-smoke-state
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute   --store "$PWD/.crunch-drain/i386-smoke-store"   --state-dir "$PWD/.crunch-drain/i386-smoke-state"   bootstrap/spike-i386-native-smoke.ncl
```

## Result

Command exit: `0`

Output path:

```text
/home/brittonr/git/crunch/crunch/.crunch-drain/i386-smoke-store/njp9r73v2qvfdcfgky0gcdbv5xkra6c8-spike-i386-native-smoke
```

Saved derivation log: `evidence/V1-i386-native-smoke.drv.log`

Key log excerpt:

```text
-rwxr-xr-x    1 nixbld   nixbld          96 May  1 23:40 i386-exit42
i386-exit42 rc=42
```

## Interpretation

Native i386 execution is not the blocker. The next proof slice should implement the sibling i386 TinyCC/Make path itself rather than falling back to qemu or abandoning the StageX/live-bootstrap comparison.
