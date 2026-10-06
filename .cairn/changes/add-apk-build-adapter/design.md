# Design: APK build adapter

## Context

An APK build without Gradle is a fixed sequence of tool invocations over declared inputs. That shape maps one-to-one onto Mantle derivations. The adapter's job is to keep the sequence deterministic and its inputs explicit, and to keep every policy decision out of the execution shell.

Robotnix provides the closest prior art in the Nix family. It builds Android apps from source under Nix and keeps signing keys explicit; its signing path needs sandbox exceptions precisely because keys stayed file-backed and ambient in places. Its `repo2nix` shows how a large upstream source closure becomes derivations. Both patterns inform this design without widening its scope.

## Architecture

```text
ApkPlan authored in Nickel (lib/android.ncl, mkApk contract)
  -> pure plan core (crunch-android-core): validate + lower to typed step plan
  -> std adapter (crunch-android): bind toolchain identities and store paths
  -> five unsigned derivations (aapt2 compile/link, javac, d8, zipalign), or six with apksigner
  -> sandboxed execution through existing Builder and FetchBuildService
```

### Pure plan core

`crates/crunch-android-core` owns the domain meaning of an APK plan:

- module identity, application id, and version;
- manifest source, resource sources, and Java sources as declared members;
- references to toolchain identity records from `lib/android/sources.ncl`;
- signing configuration: unsigned, or keystore input plus key alias and signature schemes;
- reproducibility policy: ZIP-representable fixed entry timestamps from 1980-01-01 through 2107-12-31 UTC, locale C, timezone UTC.

The core validates and lowers the plan to an ordered list of typed derivation-step plans. Each step names its inputs, its toolchain identity, and its output. The core performs no I/O, reads no environment, and touches no clock. Tests run with ordinary values only.

Rejections are typed: missing manifest member, unbound toolchain identity, unpinned timestamp policy, keystore path escape, unknown signature scheme, empty source set.

### Std adapter

`crates/crunch-android` binds a validated plan to the store. It resolves each toolchain identity record to its fetch output, checks digest binding, and generates the derivation scripts for each step. Script generation is the only place host-specific shell text exists. The adapter keeps the same failure contract as the identity binding in the admission change: drift fails before execution.

At render time, `prepare_apk` creates an owned temporary directory and calls
`crunch_eval::stdlib::write_stdlib(Some(dir.path()))` before passing that
directory as the Nickel import path. It materializes the compile-time
embedded `android.ncl` and `android/sources.ncl`, then removes them when
evaluation ends. Neither an empty import path nor the shared
`$XDG_CACHE_HOME/crunch/stdlib` path is safe: the former cannot resolve the
imports, while the latter (or `stdlib_import_path()` selecting a mutable
ancestor `lib/`) could substitute the production cohort.
The adapter checks all three local toolchain archive SHA-256 values against
those pinned records before generating the graph. A six-stage signed-plan
render establishes only derivation declarations and an output identity, not
physical APK bytes or signing.

### Nickel surface

`lib/android.ncl` exposes the `mkApk` contract. Authors write a plan record; the contract validates shape and completeness; lowering produces the derivation set. This mirrors how `builders/mk_derivation.ncl` and `lib/wasm_component.ncl` compose today.

### Determinism discipline

APK bytes are time-sensitive. Zip entries carry timestamps and signing embeds them. Every step pins:

- a fixed entry timestamp epoch (the plan's reproducibility policy);
- `LC_ALL=C` and a fixed `TZ`;
- deterministic zip entry ordering (sorted paths);
- fixed JVM user properties where a tool reads user locale from the environment.

This follows the kbuild metadata pinning already proven necessary for the busybox self-hosting fixed point.

### Signing boundary

The keystore is an explicit derivation input. The adapter never reads `~/.android`, ambient `ANDROID_HOME`, or any user configuration. An unsigned path emits the zipaligned APK before the signing step so CI consumers can sign in a separate authority domain. Robotnix reaches the same boundary from the other side: its signing needs sandbox exceptions where keys are ambient, which is the failure mode this explicit-input rule prevents.

### Local APK input admission

`crunch-android-admit` is an Android-owned, offline command rather than a
general store-ingest CLI or a foreign-realization shortcut. `observe` reads
exactly four distinct, complete directories (`app-source`, `native-glibc`,
`native-libgcc`, `test-signing`) and prints streamed canonical NAR SHA-256,
BLAKE3, and size; it creates no store state and returns `observed-only`, never
Ready. Glibc must contain the declared loader and `libc.so.6`; independent
libgcc must contain `libgcc_s.so.1`. `ApkInputs` declares `native_libgcc_root`
and exactly one runtime library directory under each native root; every SDK
stage binds both complete native source objects. The JSON manifest provides
each role, `source_path`, and exact `store_path` bound by the signed Nickel APK
plan and `ApkInputs`. The operator independently compares staged native
trees with their complete original Nix sources, reviews all four NAR
observations, and adds `expected_nar_sha256`, `expected_nar_blake3`, and
`expected_nar_size` to all four records before calling `admit`. Digest fields
are lowercase hex, and each observed NAR must be at most 1 GiB. The observer
reads SHA-256, BLAKE3, then SHA-256 again
and rejects different size or SHA-256 across traversals; this is a bounded
observation, not an atomic filesystem snapshot. Final signed-PathInfo,
castore, and physical read-back are required before declaring admission.
Members may use only relative symlinks resolved within their source tree;
the keystore and password members cannot be symlinks. The one RSA-3072
keystore for A/B replay is
**test-only**: generate it only after resources and the pinned keytool are
available, supply passwords through 0600 files via keytool's
`-storepass:file`/`-keypass:file` flags, and retain the same immutable source
for both replays. Never put a password on argv, or use production trust here.

`admit` requires separate explicit physical `--store` and `--state-dir`,
`--store-prefix`, an existing Mantle Ed25519 `--signing-key`, and an exact
matching `--trusted-public-key`. It checks every declared app file, the glibc
loader and libc, the independent libgcc library, and test keystore/password
files. It compares observed bytes with all four reviewed full-tree NAR facts
*before* signed PathInfo publication, then calls
`StoreHandle::preflight_verified_source` and `ingest_verified_source` for
those four independent paths only. Because an idempotent ingest may return
cached PathInfo without checking physical output, a cached record is checked
for trusted signature and physical NAR before ingestion.
After ingestion the adapter exports the complete castore node, requires its
persisted signed PathInfo at the exact path, read-backs physical content via
`adopt_verified_local_output` on the **already present** PathInfo, and matches
SHA-256, BLAKE3, and size across observed source, castore, and physical
export. An orphan physical path, missing store object, untrusted signer, or
drifted bytes aborts without a success receipt; a failed multi-input run can
leave earlier genuinely signed entries, so start each A/B replay with a
distinct fresh state and store, not a shared mutable proof store.

The logical store path is explicitly selected by the reviewed APK graph:
`ca=None` source ingestion does **not** derive it from content. The signed
receipt binds the observed NAR, exact logical path, trusted signer, and
physical path but does not prove that the Android upstream archives are
authentic or that the APK was built/signed. A source bundle's virtual
`store-path` record can report Ready without a signed store object; that
class is not admissible proof. The four HTTPS source bundle/pins are separate,
immutable proof inputs; this command neither fetches nor imports them.

## Failure and abuse controls

- Unpinned timestamp policy: core rejects the plan.
- Keystore path escape or ambient reference: core rejects the plan.
- Network access during any step: derivations run in the existing sandbox with no network.
- Stub tool mismatch in tests: throwaway stub scripts MAY help inspect invocation shape, but their echoed argv or environment is not permanent acceptance evidence; a consumer-visible artifact or rejection-before-effects proof is required.

## Testing

- Core unit tests: valid plans lower to five ordered unsigned steps or six signed steps; every typed rejection returns its category at boundary values.
- Adapter tests: wrong tool bytes or identities fail before graph generation; a declared source or signing change changes the final output identity while equivalent declarations remain deterministic.
- Offline integration proof MUST assert typed rejection before effects or deterministic, reproducible APK/ZIP artifact bytes. Stub-only recorded argv and environment are exploratory diagnostics, not permanent tests or proof of real pinned SDK execution. The original stub-only shape-test task remains open until it can be satisfied consistently with this rule; the separate real-build change owns actual prebuilt tool execution.
