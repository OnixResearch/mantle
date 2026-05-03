## Tasks

- [x] Add focused source-emission/reduction diagnostics for the i386 tcc27 predecessor handoff.
- [x] Build `bootstrap/spike-i386-mes-runtime-layout.ncl` and save diagnostic logs/evidence.
- [x] Update the task/spec evidence with the repaired path or the next narrowed blocker.
- [x] Repair the remaining `tcc26-i386` segfault while compiling `tccgen.c` / full `ONE_SOURCE=1` input.
- [x] Verify and commit the i386 tcc27 compile repair; leave link repair as the next blocker.
- [x] Narrow the post-compile link blocker with explicit object-existence and startup/library-search diagnostics.
- [x] Build/import a real i386 Mes `libc.a` and rerun the explicit `-nostdlib` tcc27 object link before returning to Make 3.82.
