# binutils-tcc runtime probe — 2026-05-12

## Evaluation

Command:

```sh
timeout 5m ./target/debug/crunch eval bootstrap/binutils-tcc.ncl >/tmp/binutils-tcc.json
python - <<'PY'
import json
j=json.load(open('/tmp/binutils-tcc.json'))
print(j['name'], 'script_bytes', len(j['args'][1]), 'inputs', len(j['inputs']))
open('/tmp/binutils-tcc.sh','w').write(j['args'][1])
PY
/bin/sh -n /tmp/binutils-tcc.sh
```

Result:

```text
binutils-2.30-tcc script_bytes 17176 inputs 12
exit=0
```

## Bounded build probe

Command:

```sh
timeout 9m ./target/debug/crunch build bootstrap/binutils-tcc.ncl \
  --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned
```

Result:

```text
hermeticity: practical (no degraded facts)
/home/brittonr/git/crunch/crunch/.crunch-drain/store/59vj0rbxkwj1wdn7kcak0i8zm132b1qn-binutils-2.30-tcc (cached)
exit=0
```

A second no-substitute attempt against a fresh local store directory also returned the same cached output path under `.crunch-drain/store-binutils-tcc-probe` and did not materialize the runtime closure members needed by the wrapper tools.

## Tool smoke blocker

Direct smoke of the produced `as` wrapper failed because it references logical `/crunch/store` dependencies that are not present in the local physical store:

```text
/home/brittonr/git/crunch/crunch/.crunch-drain/store/59vj0rbxkwj1wdn7kcak0i8zm132b1qn-binutils-2.30-tcc/bin/as: line 13: /crunch/store/lyjdchshm4gs387y7295b32wl8jnkxq0-tcc-0.9.27-musl-v2/bin/tcc: No such file or directory
exit=127
```

A bwrap smoke with `.crunch-drain/store` bound at `/crunch/store` reached the wrapper but then failed because the required musl runtime closure member is also absent from the local store:

```text
tcc: error: file '/crunch/store/pf59q46fx6gzwhwaz8zmwv07q9hg4fm9-musl-1.1.24-tcc-musl/lib/libc.a' not found
tcc: error: file '/crunch/store/pf59q46fx6gzwhwaz8zmwv07q9hg4fm9-musl-1.1.24-tcc-musl/lib/libc.a' not found
exit=1
```

## Conclusion

The derivation evaluates and returns a cached output, but the current local probe cannot produce the checked `bootstrap/evidence/binutils-tcc-tool-smoke.json` transcript because the output's runtime closure is incomplete in the probe store. `binutils.tcc` must remain a parity blocker until the next increment either materializes the closure for the custom store smoke or changes the build/cache path to preserve the required closure members.
