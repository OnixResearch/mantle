Task-ID: V2
Covers: bootstrap.part.tinycc.0.9.26.shift-immediates

# Repaired shift-codegen smoke

Result: PASS. The rebuilt Mes-linked TinyCC 0.9.26 emits nonzero immediate shift counts and still emits the baseline immediate mask correctly.

Compiler:

```text
/home/brittonr/git/crunch/crunch/target/live-part-tinycc-0-9-27/run-current/store/idif5ffznlbrn0ji82q6fjix3pkw1n6s-tinycc-0.9.26/bin/tcc
```

Reproducer:

```c
int shift_right(int x) { return x >> 8; }
int shift_left(int x) { return x << 3; }
int mask_low(int x) { return x & 31; }
```

Command summary:

```sh
OUT=$PWD/target/live-part-tinycc-0-9-27/run-current/store/idif5ffznlbrn0ji82q6fjix3pkw1n6s-tinycc-0.9.26
"$OUT/bin/tcc" -c shift.c -o shift.o
stat -c 'object_size=%s' shift.o
objdump -d shift.o
```

Observed:

```text
object_size=812
12: c1 f8 08              sar    $0x8,%eax
29: c1 e0 03              shl    $0x3,%eax
40: 83 e0 1f              and    $0x1f,%eax
```

Full transcript: `evidence/V2-shift-codegen-full.log`.
