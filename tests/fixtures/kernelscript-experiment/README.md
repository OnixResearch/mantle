# KernelScript planning fixtures

These files exercise Mantle's pure profile, generated-project, planner, target,
inspection, handoff, and receipt logic. They are manually reviewed fixture bytes,
not output from a successfully materialized KernelScript compiler.

`generated-userspace-probe/Makefile` and `generated-kfunc-module/Kbuild` are
retained evidence only. Tests require the Mantle plan to consume neither file.
The synthetic ELF fixtures are built in Rust tests and prove parser rejection and
shape admission only—not compiler success, BPF verifier acceptance, module load,
kernel safety, runtime behavior, Onix compatibility, ChaosControl evidence,
deployability, or production readiness.
