# V3 smoke: blocked before compiler output

- Task-ID: V3
- Covers: `bootstrap.part.tcc.musl.v2.runtime-validation.compile-smoke`
- Status: blocked
- Reason: no `tcc-0.9.27-musl-v2` output path was produced because prerequisite `sed-4.0.9-tcc.drv` failed first.

No installed `tcc`, `tcc-0.9.27-musl-v2`, `libtcc1.a`, or trivial C compile smoke is claimed here.
