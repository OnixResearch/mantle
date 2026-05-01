## Implementation

- [x] I1 Reproduce the TinyCC 0.9.27 amd64 static link segfault with a minimal Crunch diagnostic derivation. ✅ reproduced 2026-05-01T00:28:18Z [covers=bootstrap.compiler.tinycc.0.9.27.amd64.static-link] [evidence=evidence/I1-reproduction.md]
- [x] I2 Repair TinyCC 0.9.27 amd64 link behavior in the bootstrap source/build recipe without host-tool substitution. ✅ repaired link output 2026-05-01T11:31:00-04:00 [covers=bootstrap.compiler.tinycc.0.9.27.amd64.static-link] [evidence=evidence/I2-repair.md]

## Verification

- [x] V1 Build `bootstrap/tinycc.ncl` with Crunch and preserve the full transcript. ✅ passed 2026-05-01T11:30:00-04:00 [covers=bootstrap.compiler.tinycc.0.9.27.amd64.static-link] [evidence=evidence/V1-build.md]
- [~] V2 Run compile/link/execute smoke for `int main(void) { return 0; }` using the repaired TinyCC output. ⏱ blocked on runtime segfault 2026-05-01T11:25:36-04:00 [covers=bootstrap.compiler.tinycc.0.9.27.amd64.static-link] [evidence=evidence/V2-link-smoke.md]
- [ ] V3 Re-run `repair-make-tcc-amd64-varargs-runtime-validation` V1 or record the next concrete blocker. [covers=bootstrap.part.make.3.82.amd64.runtime-validation] [evidence=evidence/V3-make-validation-handoff.md]
- [ ] V4 Run OpenSpec validation for this repair. [covers=bootstrap.compiler.tinycc.0.9.27.amd64.static-link] [evidence=evidence/V4-openspec.md]
