# Expected error transcript

This transcript demonstrates fail-closed expected-error semantics: the command
must fail, and the normalized diagnostic must contain the stable fragment below.

```mantle:error
mantle eval missing-file.ncl
```

```expect
missing-file.ncl
```
