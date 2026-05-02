# S2 build evidence and blocker

Task-ID: S2
Covers: bootstrap.i386-tcc27-make-pass1.spike.blocker

Command transcript: `evidence/S2-build-flags.stdout` and `evidence/S2-build-flags.stderr`.

Output path: `/home/brittonr/git/crunch/crunch/.crunch-drain/make-diag-stage2-store/qd5mnj5wb548ihhsxr4vc2bk49la3dm6-spike-i386-tcc27-make-pass1`

Summary:

```text
status=blocked
blocked_step=tcc27_compile_object
blocked_rc=60
note=The proven tcc26-i386 predecessor is no-libc; a full tcc27/make pass requires confirming or creating an i386 Mes runtime layout.
```

Predecessor compiler version:

```text
tcc version 0.9.26-i386-spike (i386 Linux)
```

Failing step stderr:

```text
In file included from %s:%d:
%s:%d: error: include file '%s' not found
In file included from %s:%d:
%s:%d: error: '%c' expected (got "%s")
```

Interpretation: the `tcc26-i386` predecessor itself is proven for no-libc i386 object/executable emission, but the TinyCC 0.9.27 handoff immediately needs a real i386 Mes header/runtime layout. The corrupted literal `%s:%d` diagnostics are the same class of Mes/TCC formatter issue seen earlier, but the high-level blocker is earlier than Make: the sibling proof has no i386 Mes include tree or CRT/libc/libtcc1 archive to use as TinyCC's configured runtime.
