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

## Phased correctness roadmap

### Phase 1: First semantic `libgcc.a` member

Goal: prove that at least one member is more than a symbol-shaped placeholder.

Candidate order:

1. `_negdi2`: implement signed 64-bit negation with simple deterministic inputs.
2. `_muldi3`: implement 64-bit multiplication after `_negdi2` proves the archive path.
3. Shifts/comparisons: `_lshrdi3`, `_ashldi3`, `_ashrdi3`, `_cmpdi2`, `_ucmpdi2`.

Required evidence:

- Rebuild `bootstrap/gcc-4.0.ncl` with the deterministic ar(5) archive still valid.
- Host-extract the selected object from `libgcc.a` and confirm the symbol is present.
- Compile/link a tiny semantic smoke using the artifact and prove non-placeholder behavior for positive, negative, and boundary-ish inputs.
- Keep the rest of `libgcc.a` explicitly described as placeholder until each member is promoted.

### Phase 2: Driver/preprocessor behavior

Goal: replace `xgcc`/`cpp` compile-mode placeholder output with behavior that preserves user source content through at least preprocessing and object compilation.

Candidate order:

1. Make `bin/gcc`/`xgcc` reject unsupported driver modes instead of silently writing success scripts.
2. Add a compile smoke where a source-level symbol or constant appears in the output object, not just `crunch_gcc40_xgcc_object_stub`.
3. Add a preprocessing smoke for `cpp` that emits deterministic macro-expanded text for a tiny input.

Required evidence:

- `gcc -c` succeeds for a tiny C input and the resulting object contains a source-derived symbol.
- Unsupported driver modes fail closed with a diagnostic that says which behavior is still bridged.
- `cpp` smoke output is deterministic and does not depend on host include paths.

### Phase 3: `cc1` and generator replacement

Goal: replace shell-script success paths with real compiled programs or with narrower fail-closed boundaries.

Candidate order:

1. Pick one generator whose generated output is already approximated by a script, such as `gencodes` or `genattr`.
2. Compile/run the real generator far enough to produce a checked generated header fragment.
3. Replace one frontend/backend placeholder object with a real object only when its dependencies are understood.
4. Promote `cc1` last; require source-driven parsing/codegen evidence rather than only executable presence.

Required evidence:

- Real generator output is compared against the minimal currently scripted contract.
- Any remaining generator scripts are listed in this roadmap/inventory before claiming completion.
- `cc1` is not promoted until it handles a tiny translation unit through the intended GCC path.

### Phase 4: CRT and runtime/link evidence

Goal: move from object-only compile smokes toward link/runtime smokes without hiding current bwrap/store constraints.

Candidate order:

1. Keep object-only smoke as the stable baseline.
2. Add a link-only smoke that checks the GCC runtime search path and selected CRT object usage.
3. Add a runtime smoke only after the artifact can run without host dynamic-linker or writable-store leakage.

Required evidence:

- Bwrap binds the actual `.crunch-drain/store` parent as `/crunch/store` when testing artifact paths.
- The smoke names whether it proves object creation, link behavior, or runtime behavior.
- Failures are recorded as blocked runtime/link promotion, not as GCC correctness regressions.

## Promotion rule

A GCC 4.0 milestone may be promoted only when its evidence proves one bounded behavior beyond graph completion. Do not collapse multiple promotions into a single broad "GCC complete" claim; each promotion should replace one bridge/stub surface or add one semantic smoke with a concrete pass/fail command.
