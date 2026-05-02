# R3 next blocker

Task-ID: R3
Covers: bootstrap.i386-mes-libtcc1-flags.next-blocker

After real `libtcc1.o` and `libtcc1.a` creation, the next blocker is `tcc27_compile_object` with rc `139`.

Stderr:

```text
Segmentation fault (core dumped)
```

Interpretation: the repaired libtcc1 flags move the boundary forward. The next slice should diagnose `tcc26-i386` compiling TinyCC 0.9.27 `tcc.c` with a real i386 Mes runtime archive.
