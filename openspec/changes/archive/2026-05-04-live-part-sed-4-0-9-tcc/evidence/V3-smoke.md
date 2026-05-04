# V3 sed-tcc output smoke

- Task-ID: V3
- Covers: `bootstrap.part.sed.4.0.9.tcc`
- Status: passed
- Output path: `/home/brittonr/git/crunch/crunch/.crunch-drain/sed-tcc-store/2xngwf3d5m7gl54cph0s0x24vb5ykyps-sed-4.0.9-tcc`

Smoke command:

```sh
OUT="$PWD/.crunch-drain/sed-tcc-store/2xngwf3d5m7gl54cph0s0x24vb5ykyps-sed-4.0.9-tcc"
test -x "$OUT/bin/sed"
"$OUT/bin/sed" --version | head -1
printf 'test\n' | "$OUT/bin/sed" 's/test/ok/'
```

Observed output:

```text
GNU sed version 4.0.9
ok
```

The derivation-local smoke in `V2-sed-tcc-root-derivation.log` also records `GNU sed version 4.0.9` and `ok`.
