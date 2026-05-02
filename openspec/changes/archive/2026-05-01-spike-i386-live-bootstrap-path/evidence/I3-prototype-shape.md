# I3 Prototype shape decision

Task-ID: I3

Covers: `bootstrap.i386-live-bootstrap-spike.prototype-shape`

## Decision

Use **native i386 userspace execution under the amd64 Linux kernel/bwrap sandbox** for the first Crunch-local proof attempt.

Do not start with qemu-user. The current host can execute a minimal i386 static ELF directly, and the same executable runs inside a minimal bubblewrap sandbox. That keeps the proof closer to Crunch's normal native sandbox model and avoids adding an emulator/provider dependency before it is needed.

## Feasibility check

Generated a tiny i386 ET_EXEC binary with code equivalent to:

```asm
mov eax, 1      ; sys_exit
mov ebx, 42
int 0x80
```

Host execution result:

```text
/tmp/crunch-i386-exit42: ELF 32-bit LSB executable, Intel 80386, version 1 (SYSV), statically linked, no section header
native-i386-exit=42
```

Bubblewrap execution result:

```text
bwrap-native-i386-exit=42
```

The host also has `/run/current-system/sw/bin/qemu-i386`, but qemu is not required for the first proof shape.

## Prototype shape

The V1 proof attempt should be a narrow Crunch derivation/proof lane that:

1. Emits or reuses the upstream i386-oriented `tcc-0.9.26 -> tcc-0.9.27` semantics instead of mutating `bootstrap/tinycc.ncl`.
2. Builds `make-3.82` with `TCC_TARGET_I386=1` lineage and the pinned live-bootstrap `steps/make-3.82/pass1.kaem` compile/link list.
3. Adds a stronger Crunch smoke than upstream pass1:
   - `make --version` exits 0;
   - a simple Makefile runs a recipe and produces the expected file/content.
4. Records whether the proof used native i386 execution or had to fall back to qemu.

## Explicit rejection for this slice

Do not switch production `bootstrap/make-tcc.ncl` to i386 yet. The first proof should be a sibling spike/proof target or scratch derivation. If it passes, the next change can add first-class platform support (`'i386-linux` or `'i686-linux`) and decide how to surface the i386 bootstrap path in stable Crunch CLI/API behavior.
