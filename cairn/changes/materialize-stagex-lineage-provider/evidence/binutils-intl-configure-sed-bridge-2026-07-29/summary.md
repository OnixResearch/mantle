# Binutils `intl/configure` sed bridge

## Result

Mantle ran the authenticated binutils 2.30 `intl/configure` script with the protected later Bash.

The existing protected sed works correctly with named regular files. It loses required input when the same scripts read standard input through a pipe.

The new bridge copies bounded standard input into create-new regular files. The unchanged protected sed still parses and executes each sed program.

The configure command exited with status 0. It created `config.status`, `Makefile`, `config.intl`, and `config.h`.

## Identities

- Binutils source: `809e8c1d946b14650362cf2929520efe07623fe069ce8c16c5870dafe6d93603`
- Binutils recipe: `406d4aaecf2cc8c9b357463fb8f48f185acaa67e77c4136e5908cca851efd177`
- Launcher source: `606c52085de42d0221ba5490e81d8c539a50a65aaa89f7b4b478371bdc643dab`
- Bridge script source: `d2c195da10d206de2538c03ad7f10579d3c50acbbca649ce0952bd102da467d0`
- Single-thread semaphore source: `52c3ec19c484b0b4c3c5de84fc7ae77f40601fc81993d16ef6e15eb7401ee084`
- Compiled launcher: `9b4d6a5eca05f55c407a70f7e426f756f482c9ae8f76d0a22d1b1b46e7dba1a1`
- Generated `config.intl`: `dd19f59458971e4d566f38f93a529bf7bbee80eede3112a850f02ff59391af4d`
- Generated `config.h`: `62ff9551e66878ada3292c9af93cfca1413d8c99feab391388261d7f057a0e00`

Generated `config.status` and `Makefile` contain scratch paths. Their retained evidence digests are path-scoped.

## Bounds and negative cases

- The bridge permits at most 4,096 invocations.
- Each input and output has an 8 MiB limit.
- The native launcher permits at most 128 arguments.
- The final configure run recorded 157 bridge invocations.
- The compiler wrapper recorded 11 bounded preprocessor probes.
- The bridge rejected mixed standard-input and explicit-file authority.
- The native launcher rejected missing Bash and script authority.
- Mantle validated contiguous raw invocation numbers and wrote a canonical multiset audit.

## Validation

- Task `1132`: the exact authenticated configure test passed in 25.89 seconds.
- Task `1081`: six positive and negative focused tests passed.
- Task `1107`: strict first-party Clippy passed.
- Task `1108`: formatting and `git diff --check` passed.
- Task `1180`: the cached Cairn binary passed validation and the tasks gate with the sibling generated policy.

The current sibling Cairn source build remains blocked by its unrelated unclosed delimiter. Task `1180` is supplemental validation, not a fresh sibling-source build.

`configure.stderr.txt` retains the configure warnings and one `Broken pipe` diagnostic. The command still returned status 0 and created all required outputs.

## Boundary

The bridge does not implement sed or Autoconf substitution semantics. It only changes standard-input transport to a bounded regular file.

This evidence proves the bounded `intl/configure` result only. It does not prove GNU binutils, general sed correctness, compiler correctness, or provider admission.
