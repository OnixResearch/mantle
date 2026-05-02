# S2 build evidence

Task-ID: S2
Covers: bootstrap.i386-mes-runtime-layout.evidence

Command transcript: `evidence/S2-build.stdout` and `evidence/S2-build.stderr`.

Output path: `/home/brittonr/git/crunch/crunch/.crunch-drain/make-diag-stage2-store/ahssz0dbcs0116pjbnpw0iwimli5a672-spike-i386-mes-runtime-layout`

Output summary:

```text
status=blocked
blocked_step=tcc27_compile_object
blocked_rc=139
note=The i386 Mes runtime/header layout proof found the first concrete blocker before tcc27 can use the layout.
```

Copied derivation logs: `evidence/logs/`.

Key step return codes:

- `runtime_crt1_object`: `0`
- `runtime_libtcc1_object`: `139`
- `tcc27_compile_object`: `139`
