# V3 smoke

Focused validation built and ran the produced output-contract smokes inside the derivation. The builder executed:

```sh
test -x ./bzip2
test -x ./bzip2recover
./bzip2 --help 2>&1 | $BB head -1
./bzip2recover 2>&1 | $BB head -1

$BB install -Dm755 bzip2 "$out/bin/bzip2"
$BB install -Dm755 bzip2recover "$out/bin/bzip2recover"

"$out/bin/bzip2" --help 2>&1 | $BB head -1
"$out/bin/bzip2recover" 2>&1 | $BB head -1
```

Validation summary: `evidence/V2-bzip2-musl-pass-validation-summary.md` reports `Status: Passed` and `Build exit code: Some(0)`.

Produced files:

- `/home/brittonr/git/crunch/crunch/.crunch-drain/bzip2-musl15-store/xwwwxypc42pncdzl4bj46cwjw4lzdhfz-bzip2-1.0.8-musl/bin/bzip2`
- `/home/brittonr/git/crunch/crunch/.crunch-drain/bzip2-musl15-store/xwwwxypc42pncdzl4bj46cwjw4lzdhfz-bzip2-1.0.8-musl/bin/bzip2recover`
