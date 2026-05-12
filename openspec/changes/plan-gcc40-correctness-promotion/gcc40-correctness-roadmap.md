# GCC 4.0 correctness promotion roadmap

## Current verified boundary

The current `bootstrap/gcc-4.0.ncl` milestone is a GCC-4.0-shaped pass1 bridge and graph-completion artifact for the C language. It is useful bootstrap progress, but it is not native/self-hosted/correct GCC.

Verified artifact-shape milestones already present:

- `gcc-4.0.4 build complete (languages: c)` appears at the end of the derivation build.
- Installed executables/entrypoints: `bin/gcc`, `bin/cc`, `libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1`.
- Installed CRT objects: `crtbegin.o`, `crtbeginS.o`, `crtbeginT.o`, `crtend.o`, `crtendS.o` when produced by the pass1 build tree.
- Installed deterministic `libgcc.a` as a direct ar(5) archive with named members such as `_divdi3.o`, `__gcc_bcmp.o`, `_muldi3.o`, and `_negdi2.o`.
- Object-only smoke: `bin/gcc -c` produces a non-empty object through the TinyCC-backed bridge.

## Current bridge/stub surfaces

### Driver and executable stubs

- `bin/gcc` is a shell bridge that reports `gcc (Crunch pass1 bridge) 4.0.4` for version output and delegates compilation to the validated musl TinyCC toolchain.
- `bin/cc` is a symlink to the bridge driver.
- `cc1` is a shell placeholder that prints the pass1 bridge message and exits successfully.
- Build outputs for `cc1`, `gcov`, `gcov-dump`, `Tcollect2`, `collect2`, `xgcc`, and `cpp` can be emitted as shell stubs when the wrapper sees no C source input.
- `xgcc`/`cpp` compile mode currently recognizes `-c` and `-o`, then asks TinyCC to compile a tiny placeholder object rather than running GCC's real driver/preprocessor pipeline.

### Generator and generated-header stubs

- Several GCC generator sources are recognized and short-circuited with tiny TCC-built objects, including `genconfig`, `gencodes`, `genconditions`, `genpreds`, `genattr`-family sources, and `gcov-iov`.
- Generated programs such as `genmodes`, `gencodes`, `genattr`, `gcov-iov`, `genrecog`, `genextract`, `genpeep`, `genopinit`, `genoutput`, and `genattrtab` can be emitted as scripts that generate minimal headers or source fragments.
- These scripts preserve graph progress but are not proof that GCC's real generator programs compile, execute, and produce semantically equivalent generated files.

### Frontend/backend object bridges

- C frontend surfaces such as `c-lex`, `c-pragma`, and `c-decl` still have bridge bodies or placeholder symbols.
- A broad `bridge_any_gcc_obj` fallback can emit a generic placeholder object for remaining GCC object outputs.
- Backend/common object coverage therefore represents graph completion rather than native object correctness.

### Runtime/archive limitations

- `libgcc.a` has member granularity, but each member body is currently generated as `int <fn>(void) { return 0; }`.
- The archive intentionally bypasses the TCC-built `ar`/`ranlib` path because the current bootstrap `ranlib` rewrites the hand-authored archive into an empty/index-only form.
- Semantic smokes currently prove member names and symbols, not arithmetic/runtime behavior.

## Promotion rule

A GCC 4.0 milestone may be promoted only when its evidence proves one bounded behavior beyond graph completion. Do not collapse multiple promotions into a single broad "GCC complete" claim; each promotion should replace one bridge/stub surface or add one semantic smoke with a concrete pass/fail command.
