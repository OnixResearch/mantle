Task-ID: I3
Covers: bootstrap.part.tinycc.0.9.27

# tinycc 0.9.27 derivation fix

Result: PASS pending paired V2/V3 transcripts.

Files changed:

- `bootstrap/tinycc.ncl`

Implemented changes:

- applies the live-bootstrap `tcctools.c` file-open adjustment;
- applies the live-bootstrap `tccelf.c` Fiwix physical-address adjustment;
- applies the live-bootstrap `tccelf.c` null-GOT relocation guard;
- normalizes predecessor-sensitive doubling expressions that the amd64 `tinycc 0.9.26` compiler can miscompile as `shl $0`;
- rewrites Mes-runtime-sensitive `snprintf`/`vsnprintf` path formatting in bootstrap-only `tcc 0.9.27` sources to direct string operations;
- builds `tcc 0.9.27` with the declared predecessor `tinycc 0.9.26`;
- installs `bin/tcc`, `bin/tcc-0.9.27`, `include/mes`, and `lib/mes` runtime artifacts into the produced output.

Intentional Crunch amd64 deviation:

- upstream `pass1.kaem` is x86-oriented and rebuilds Mes runtime with the newly built `tcc 0.9.27`;
- Crunch's amd64 Mes CRT sources still contain inline assembly forms that this Mes-linked `tcc 0.9.27` cannot parse (`bad operand with opcode 'mov'` during the attempted CRT rebuild);
- this part therefore copies the predecessor Mes runtime contract forward and leaves the next runtime/compiler handoff to a downstream musl-oriented part.

Acceptance boundary:

- source-pin audit is recorded in `V1-source-pins.md`;
- build success is recorded in `V2-build-success.md`;
- output contract smoke is recorded in `V3-smoke.md`.
