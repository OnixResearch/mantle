# Design: APK build adapter

## Context

An APK build without Gradle is a fixed sequence of tool invocations over declared inputs. That shape maps one-to-one onto Mantle derivations. The adapter's job is to keep the sequence deterministic and its inputs explicit, and to keep every policy decision out of the execution shell.

Robotnix provides the closest prior art in the Nix family. It builds Android apps from source under Nix and keeps signing keys explicit; its signing path needs sandbox exceptions precisely because keys stayed file-backed and ambient in places. Its `repo2nix` shows how a large upstream source closure becomes derivations. Both patterns inform this design without widening its scope.

## Architecture

```text
ApkPlan authored in Nickel (lib/android.ncl, mkApk contract)
  -> pure plan core (crunch-android-core): validate + lower to typed step plan
  -> std adapter (crunch-android): bind toolchain identities and store paths
  -> five derivations (aapt2 compile/link, javac, d8, zipalign, apksigner)
  -> sandboxed execution through existing Builder and FetchBuildService
```

### Pure plan core

`crates/crunch-android-core` owns the domain meaning of an APK plan:

- module identity, application id, and version;
- manifest source, resource sources, and Java sources as declared members;
- references to toolchain identity records from `lib/android/sources.ncl`;
- signing configuration: unsigned, or keystore input plus key alias and signature schemes;
- reproducibility policy: fixed entry timestamp epoch, locale, timezone.

The core validates and lowers the plan to an ordered list of typed derivation-step plans. Each step names its inputs, its toolchain identity, and its output. The core performs no I/O, reads no environment, and touches no clock. Tests run with ordinary values only.

Rejections are typed: missing manifest member, unbound toolchain identity, unpinned timestamp policy, keystore path escape, unknown signature scheme, empty source set.

### Std adapter

`crates/crunch-android` binds a validated plan to the store. It resolves each toolchain identity record to its fetch output, checks digest binding, and generates the derivation scripts for each step. Script generation is the only place host-specific shell text exists. The adapter keeps the same failure contract as the identity binding in the admission change: drift fails before execution.

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

## Failure and abuse controls

- Unpinned timestamp policy: core rejects the plan.
- Keystore path escape or ambient reference: core rejects the plan.
- Network access during any step: derivations run in the existing sandbox with no network.
- Stub tool mismatch in tests: shape tests assert exact argv ordering and environment, so an accidental live tool call fails the test.

## Testing

- Core unit tests: positive plans lower to five ordered steps; every typed rejection returns its category. Table-driven over boundary values.
- Adapter tests: binding resolves identities; drift fails before script generation.
- Offline shape tests: stub executables record argv and environment per step; tests assert ordering, input wiring, pinned env, and absence of ambient paths. Negative fixtures cover each typed rejection end to end.
