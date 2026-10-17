# T4.2: retained same-client native-provider selective rerun

Accepted only for the frozen signed cohort in the preserved client on
`brittonr@100.90.108.13`, root
`/tmp/mantle-unit-plan-20261013-87lqAtNv`. The proof ran on 2026-10-14 and
completed on **2026-10-15 UTC**. Maintenance owner: Mantle lane operator.

## Actual settlement and outcome

The original existing producer/guard/callback tasks 1061/1062/1063 and the
resulting app/guard tasks 1064/1065 all settled **Success**; no accepted
producer or application was replayed. Exact commands, dependencies, start/end
times and final resource ticks are in
`t42-selective-provider-2026-10-15/t42-pueue-settlement-20261015.txt`.
The earlier provider/config tasks 1058/1059 and callback 1060 also settled
Success. The observer wait exit is not used as task-success evidence.

The actual producer graph completed at 2026-10-15T00:35:42.356719+00:00.
Its three trusted physical producer outputs completed verification at
00:35:43.783950+00:00. The success-only callback selected the actual app
root and scheduled task 1064 with its independently Running guard 1065.
The actual app checker executed with no substitutions and completed exit 0
at 00:51:51.967590+00:00. Named trusted physical app verification completed
at 00:51:52.712125+00:00; the strict comparison completed **exit 0** at
00:51:53.065279+00:00. These exact records are in the retained producer/app
phase logs, not inferred from quiet trace intervals.

Aggregate producer report counts (requested outcomes, not all internal unit
events): `{"succeeded_total":2,"built_total":2,"cached_total":0,"failed_total":0}`.
Aggregate actual app-consumer report counts:
`{"succeeded_total":3,"built_total":1,"cached_total":2,"failed_total":0}`.
The retained physical `result.txt` contains exactly:

`cargo-build-script-fixture: proc-macro2 serde-derive libc native-sys = 42\n`

`variant-native-app-verify.log` records
`4mwhnm01ygapwk05fhbi7cafq2nqmjml-cargo-build-script-units-app-check-variant-87lq`,
`trusted_signatures=1/1`, and `1 checked, 0 mismatches`. The producer verify
log separately names all three actual out/plan/sources roots with trusted
signatures and zero physical mismatches.

## Strict selective evidence

Only the declared provider changes between otherwise identical configs:

- Before: `/nix/store/77giqf2dbsvv1sjcmydx9x366s5614fc-native-answer-0.1`.
- After: `/nix/store/jiwk6cxf281jlsxmlx80d02cmrw7cc1p-native-answer-0.2`.

The role is keyed by the exact Cargo package identity
`path+file:///build/cargo-unit-plan-workspace/workspace/native-answer-sys#0.1.0`;
its `NATIVE_ANSWER_ROOT = "{{role:answer}}"` marker is unchanged. Both
plans contain exactly 34 unique accepted units and 14 separately admitted
source slices with matching declared/observed NAR BLAKE3 values. The checker
retains native role/input, script-flags and app-extern bindings; the producer
recipe may change only the provider/config root and its goal label.

The original retained full trace proves all **34 actual unit builds**. The
new full trace proves exactly **31 positive shared-action admitted-cache
events** for independent complete unchanged unit records and exactly
**3 successful affected builds**: native script, native-sys compilation and
application. Presence and unchanged independent identities for proc_macro2,
serde_derive, libc and serde are required, not inferred from a count alone.
The exact final verdict is retained in the app phase log:

`selective-native-provider-rerun=success ... independent_unit_count=31 all_independent_records_unchanged=true all_independent_positive_cache_events=true affected_successful_build_events=3`

### Nominal labels versus effective identity

The nominal `u.<digest>` labels remain stable. Production lowering hashes
the Cargo unit label, while dynamic registration resolves inputs/dependency
outputs and derives effective derivation/output paths. Stable nominal labels
are therefore not themselves a product bug. Both actual old/new plans declare
all three affected units **input-addressed**, without a fixed output; the
checker expressly rejects applying its output-inequality rule to arbitrary
content-addressed units. Actual successful output paths change:

| Affected unit | Original output basename(s) | Variant output basename(s) |
|---|---|---|
| native script out | ch1aikvb70ilsx8qivz90qg52z8w7g1q-rust-build-script-a858f83affef223a | kx82g89000xa9ic9girkqh9dkcpz3j86-rust-build-script-a858f83affef223a |
| native script flags | id704qbnsnnhmbw6wan9nv7ax9s5b5jp-rust-build-script-a858f83affef223a-flags | zl8f0jz00q2kncwracdscl3izq2f02za-rust-build-script-a858f83affef223a-flags |
| native-sys out | fmv8q10ggyflcpj0wclrfrracfh7m3c7-rust-unit-3d2854f4a1448c88 | aw99fcvqd5502wjjhqam781ivx4s2kvv-rust-unit-3d2854f4a1448c88 |
| application out | 55l3hmyp9an8h7j57r1s6lv8ih904igk-rust-unit-99f32f5ae94f8220 | km2mbn07abmkw5m64vqq2msbf4q5mjv2-rust-unit-99f32f5ae94f8220 |

The actual canonical app derivation changes from
`/nix/store/byyl9i4d9kjyyz7fhi8j5nngy10rnw81-rust-unit-99f32f5ae94f8220.drv`
to
`/nix/store/2n94143ybp2qm4y41826vab0lzbq0ak0-rust-unit-99f32f5ae94f8220.drv`.
Each actual app output is bound to its report's canonical goal and logical
artifact attestation; printed unit-name prefixes alone are insufficient.
Child intermediates may remain in castore; no absent-intermediate physical
export is treated as failure.

## Checker correction and preserved history

The wrong nominal-ID-change predicates were repaired before the existing
callback reached final comparison. The exact old 17,340-byte checker is
retained with BLAKE3
`45cc1a6f750f19cc10ef01497f53aafb7ff469cabf938b6c0017c0929c0f680f`.
The authoritative corrected 21,648-byte checker has BLAKE3
`0fd10b4c333c92cb42fbb53186171b65fc9fa3f82357db67dca21b740be67a1c`.
A notification about the retired prior worker triggered one exact-byte/hash
comparison, which confirmed that the authoritative repair was unchanged; no
stale in-memory candidate was adopted. The old checker, prior goal-role
version, failed initial smoke source/log and passing smoke source/log are
retained. The initial smoke's missing-output mutation had failed to strip
ANSI markup; fixing test-data construction did not weaken the checker.

The actual-data focused smoke passed original34-build parsing, three actual
original successful-output bindings, one actual app-report binding and seven
meaningful refusals: unchanged effective paths, content-addressed
generalization, missing effective output evidence, wrong logical report
binding, duplicate/missing execution events, and a build presented as
positive cache evidence. The eventual final positive is the real completed
31-cache/3-build/app42 campaign, not modeled execution data.

## Retention boundary and prerequisites

`t42-selective-provider-2026-10-15-manifest.json` records exact source/local
paths, byte counts, canonical BLAKE3-256 and separately labelled transport
SHA-256 for **45 raw files / 738,221 bytes**. The prewrite bound was 750,000
raw bytes/45 raw files, with 131,072 bytes for manifest/prose and a 524,288-byte
future-edit/gate reserve. Immediate destination preflight at
2026-10-15T00:56:16.868951+00:00 observed 2,359,296 available bytes and 4,848
available inodes. `f_frsize` was not treated as per-file allocation. Files
were transferred as exact bytes, without JSON reserialization, and every
retained byte count and SHA-256 matched remote settled metadata; canonical
BLAKE3 used only existing LOCAL `/run/current-system/sw/bin/b3sum`.
No remote b3sum install/retry, cache cleanup, quota change or local Root/.omp
ENOSPC remediation is claimed. Large later validation logs remain remote.

The frozen T4.1 manifest/40 raw captures (441,518 bytes) are referenced, not
duplicated. They retain the seven nonzero selected signed pulls with zero
rejection counters, all 59 prior trusted physical root verifications,
repaired current planner/helper/config/tool/runtime inputs, original producer
trace and actual app42, plus the 1047 logical/physical-binding and 1051
post-build goal-role checker history. New provider/config admission and
verification receipts are retained here. Existing T3.3 host-file, TCP,
missing-role and real compiled exit7 sandbox controls remain prerequisite
evidence, not new reruns. No private signing key is copied.

This accepts T4.2 only for these frozen signed inputs and the declared fixture.
It does not accept T4.3 before its separate matrix/test/doc cutover and gates,
full Cargo compatibility, host/pkg-config discovery, arbitrary native C
compilation, compiler correctness, a global current-source proof, baseline
T1.1, sync/archive/publication or release eligibility.
