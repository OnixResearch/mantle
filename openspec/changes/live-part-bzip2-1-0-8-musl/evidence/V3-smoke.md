# V3 smoke: deferred by prerequisite failure

- Task-ID: V3
- Covers: `bootstrap.part.bzip2.1.0.8.musl.evidence-isolated`
- Status: blocked/deferred
- Reason: no `bzip2-1.0.8-musl` output path was produced because the prerequisite `tcc-0.9.27-musl-v2.drv` failed first.
- Follow-up owner: `live-part-tcc-musl-v2-runtime-validation` must produce a usable compiler before this part can be rebuilt and smoke-tested.

No bzip2 runtime smoke is claimed by this evidence file.
