Task-ID: I3
Covers: bootstrap.part.tinycc.0.9.26.shift-immediates

# TinyCC 0.9.27 follow-up patch

Result: PASS. The predecessor shift fix exposed one remaining TinyCC 0.9.27 Mes-runtime formatting hazard during GNU make `getopt.c` compilation.

After rebuilding TinyCC 0.9.27 from the repaired TinyCC 0.9.26, a minimal global-reference compile such as:

```c
int first_nonopt;
int f(void) { return first_nonopt; }
```

segfaulted in the Mes `snprintf`/`vsnprintf` path while creating a relocation section name. GDB showed the call chain reaching the `snprintf(buf, sizeof(buf), REL_SECTION_FMT, s->name)` site in `tccelf.c::put_elf_reloca(...)`; the varargs path passed a bad `%s` argument to `strlen`.

Patch:

```sh
sed -i 's|snprintf(buf, sizeof(buf), REL_SECTION_FMT, s->name);|strcpy(buf, ".rela"); strcat(buf, s->name);|' tccelf.c
```

Scope: this is the same narrow direct-string workaround family already used for Mes-linked TinyCC bootstrap path construction. The target is amd64/ELF64 where `REL_SECTION_FMT` is `.rela%s`, so the replacement keeps the intended relocation-section name without entering the broken Mes varargs formatting path.

Verification: the repaired TinyCC 0.9.27 build now compiles `hello.c` and GNU make `getopt.c` objects under bounded timeouts; see `evidence/V4-smoke.md`.
