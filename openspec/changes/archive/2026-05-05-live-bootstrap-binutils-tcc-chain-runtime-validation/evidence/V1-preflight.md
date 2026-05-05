# V1 build preflight

Task-ID: V1
Covers: `bootstrap.binutils.tcc.runtime-validation`

## Result

PASS: build-profile preflight succeeded with bubblewrap on PATH and writable local state/store directories.

## Command

```sh
mkdir -p .crunch-drain/binutils-tcc-chain-store \
  .crunch-drain/binutils-tcc-chain-state \
  openspec/changes/live-bootstrap-binutils-tcc-chain-runtime-validation/evidence
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch doctor --profile build \
  --store "$PWD/.crunch-drain/binutils-tcc-chain-store" \
  --state-dir "$PWD/.crunch-drain/binutils-tcc-chain-state" \
  2>&1 | tee openspec/changes/live-bootstrap-binutils-tcc-chain-runtime-validation/evidence/V1-preflight.log
```

Exit status: 0

## Transcript

See `openspec/changes/live-bootstrap-binutils-tcc-chain-runtime-validation/evidence/V1-preflight.log`.

Key result:

```text
doctor profile: build
status: ok
- [ok] bwrap: found /nix/store/dk9qhjgg469lv6mriys7v4c59igarmvx-bubblewrap-0.11.1/bin/bwrap
- [ok] sandbox-shell: found static sandbox shell /nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox
- [ok] fusermount3: found /run/wrappers/bin/fusermount3
- [ok] state-dir: state directory is writable via /home/brittonr/git/crunch/crunch/.crunch-drain/binutils-tcc-chain-state
- [ok] store-dir: store directory is writable via /home/brittonr/git/crunch/crunch/.crunch-drain/binutils-tcc-chain-store
```
