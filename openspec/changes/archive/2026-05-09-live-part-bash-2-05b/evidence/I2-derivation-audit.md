# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.bash.2.05b

- Audited `bootstrap/bash-2.05b-tcc.ncl` source pin, generated config stubs, direct TinyCC compile/link steps, and declared inputs.
- Intentional Crunch deviation: derivation bypasses upstream configure/make generated-parser flow with hand-authored bootstrap stubs suitable for the early TinyCC/Mes handoff.
- Required failure semantics: compile/link/object-mode/runtime-smoke failures must propagate; a partial `bash` output is not acceptable runtime proof.
