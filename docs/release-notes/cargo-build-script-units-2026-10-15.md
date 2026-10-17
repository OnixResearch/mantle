# Cargo unit-plan build scripts: bounded signed-client acceptance

Cargo unit plans execute build scripts as declared sandbox units with typed
`out`/`flags` outputs, declared compiler target facts and native input roles.
The supported directive vocabulary and fail-closed limits are documented in
`docs/operator-workflows.md`; ambient host discovery is not a native-input API.

The retained 2026-10-14/15 signed-client fixture executes proc-macro2,
serde-derive, libc and a native-sys static-link application and observes 42.
Changing only its declared Mantle-built prebuilt-C provider records exactly
31 independent positive cache-admission events and 3 affected actual builds,
then executes app42 with trusted physical verification. Nominal Cargo unit
labels remain stable; actual input-addressed derivation/output paths change.
See the T4.1/T4.2 exact-byte manifests and receipts under
`.cairn/changes/run-cargo-build-scripts-as-plan-units/evidence/`.

The matrix distinguishes this narrow `declared-native-static-link` surface
from the original full `native-link-metadata` group, which remains blocked:
pkg-config execution, C compilation inside a build script, and full links
metadata consumption are not established by a prebuilt static archive.
Unexercised offline Cargo/no-oracle surfaces no longer cite the retired
source scanner or fake-Cargo output as compilation/blocker evidence.

Supported `unit_plan` rows without a named fixture now fail the production
Nickel row contract with the row ID in the error. The drift rail mutates the
real matrix and exercises that boundary, rather than only checking that its
current fixture list happens to be nonempty. The helper fixture also links
and executes a real consumer whose generated source, cfgs and script-provided
environment combine to yield 84; library-file existence alone is insufficient.

These claims do not establish full Cargo compatibility, pkg-config discovery,
compiler correctness, global current-source reproducibility, baseline/archive
completion, bootstrap correctness or release eligibility. Ordinary tests do
not replay the preserved heavyweight signed-client proof.
