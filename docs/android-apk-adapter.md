# Android APK adapter (offline, staged)

The APK adapter does not run Gradle or download dependencies. `crunch-android-core` validates a bounded typed plan without I/O, and `crunch-android` binds independently reviewed prebuilt records to fixed-output archives. `android.reviewed`'s `mantle-android-prebuilt-record-v1` metadata digest is **not** the published ADR 0089 `mantle-android-source-v1` record identity; the same archive has different hashes under their different preimages. The finite reviewed-to-canonical bridge checks exact aliases, versions, URL, archive root, platform, and pinned hex/SRI SHA-256 equivalence, then requires published source-v1 validation and binding before `android.reviewed.bind_tool` returns an executable consumer. Authoring a reviewed APK plan checks the same canonical cohort but does not itself run tools or constitute a signed execution receipt. Pass expected build-tools, Temurin JDK and Android platform records from `android.reviewed.reviewed_source`, not copies from the authored plan. Each archive is checked against the reviewed metadata and SHA-256 before execution. Manifest, resource and Java members must be regular non-symlink files beneath the app source tree; the keystore member must be a regular non-symlink file beneath a distinct signing store object, with an explicit password-file input. Do not import the user's `HOME`, `ANDROID_HOME`, SDK installation or signing config.

Author APK adapter plans with `let android = import "android.ncl" in android.reviewed.mkApk { module = "demo", application_id = "org.example.demo", version_code = 1, manifest_member = "AndroidManifest.xml", resources = ["res/values/strings.xml"], java_sources = ["src/Demo.java"] }`. This named reviewed API supplies only the reviewed build-tools 35.0.0, Temurin 17.0.17+10 and Android platform 35 r02 metadata and the fixed ZIP epoch unless explicitly supplied matching reviewed records/policy. It rejects unsafe source members, metadata drift, unsupported signing schemes and missing source sets before fetch or execution. `android.reviewed.fetch_toolchain` constructs three independently pinned FOD fetch derivations; evaluating `android.reviewed.mkApk` only produces the typed plan. Top-level `android.mkApk` remains the distinct published APK contract; it does not infer reviewed authority from the presence of a field. Pass the reviewed plan and independently admitted archive paths into the Rust adapter. An explicitly signed plan names a relative `keys/` keystore member inside a separately admitted signing store object and a scheme set; the password file is supplied separately to the signing derivation.
The reviewed fetch root lowers the already-checked published source-v1 records, rather than independently naming fetch derivations from the reviewed metadata; its original-HTTPS source-bundle identities therefore match the pinned published cohort. This does not equate the reviewed metadata digest with a source-v1 identity.

Trust seams remain explicit: `android.reviewed.bind_tool` calls the published Nickel validator/binder before admitting a canonical source-v1 fetch input. Direct Rust `crunch_android::PreparedApk::bind` instead requires a separately approved reviewed `Toolchain` argument and verifies the archive bytes against its SHA-256; it **does not** implicitly admit an ADR 0089 source-v1 record. The distinct `crunch-android-apk` CLI uses `crunch_android::pinned::prepare_apk`, which evaluates the published cohort itself. Do not cite a direct reviewed Rust binder result as evidence of published source-v1 admission.

One deterministic extraction derivation per archive rechecks SHA-256 and extracts into its output. The source archive and statically linked busybox-style utility must both be declared store inputs. The *same-run* trust boundary uses the real builder's noncached `BuildOutcome` for each exact converted extraction `.drv`, and reads independently trusted FOD/utility/output PathInfo from `crunch-store::OutputLookup` before admitting extracted trees. PathInfo's `deriver` field is **not signed** and cannot establish execution provenance on its own. Reject cache/fresh-process reuse until a signed derivation binding or independently recomputed extracted content can establish that edge. The offline sandbox test exercises this boundary with real fetch and extraction derivations; it does not run SDK stages.

Tool stages are separate derivations: `aapt2 compile`, `aapt2 link` (manifest/application ID/version code), `javac`, `d8`, `jar` update then `zipalign`, and optional `apksigner`. A build with no signing configuration ends at `unsigned.apk`. Every stage pins `SOURCE_DATE_EPOCH`, `LC_ALL=C`, `TZ=UTC`, JVM locale/timezone/encoding options and sorted source/class/resource order. The `jar` update stamps its DEX entry from the fixed epoch before zip alignment. The direct adapter requires the keystore to be the plan's declared `keys/` member inside a **separate signing store object**; neither it nor the password may share the app source object. This lexical/file check is not a signature check: the builder must establish trusted PathInfo for the complete signing object. Both signing input store objects are declared only on the `apksigner` derivation. A reviewed prebuilt metadata digest is not an archive-content digest or a published ADR 0089 source-v1 identity; archive SHA-256 and store trust remain separate checks.

The offline real-builder fixture requires static BusyBox via `SNIX_BUILD_SANDBOX_SHELL` (the flake's nextest check supplies `pkgs.pkgsStatic.busybox`); a missing utility fails the test rather than skipping it. Its signed, symlink-free store tree member named `busybox` has its parent store object declared as a dependency. Three local `file://` fixture archives have their exact SHA-256 fixed-output derivations built through `FetchBuildService`. `BubblewrapBuildService` then realizes each extraction derivation. The test rejects swapped extraction outcomes, missing trust keys, cached outcomes, and runtime digest drift before an SDK stage is generated. Synthetic SDK members fail closed if invoked; the test produces no APK and claims no official SDK execution, APK installability, or network access.

Content-addressed output paths are unknown until each stage is realized. A consumer must realize the extraction derivations, verify the same-run outcomes against the configured store keys, and then realize APK stages in order, providing already-verified output paths from earlier stages. There is no guessed store path and no caller-supplied extracted tool tree. The source admission and real-SDK build proofs are separate changes. The offline extraction/admission test does not establish an official APK's installability, behavior, byte-for-byte reproducibility, or release eligibility.

Official Google build-tools and Temurin binaries are dynamically linked: the reviewed `aapt2` requests `/lib64/ld-linux-x86-64.so.2`, while the builder mounts no host `/lib64`. The adapter instead binds separately reviewed glibc/libgcc source cohorts by full-tree NAR SHA-256 and signed PathInfo before SDK execution. Both complete store objects enter **every** SDK derivation; its command script explicitly invokes the declared glibc loader with a declared-library search path for `aapt2`, `javac`, `d8`, `jar`, `zipalign`, and `apksigner`. Do not bind the host root, `/lib64`, or an ambient `/nix/store` path into the sandbox. The separately tracked official-build rail checks these actual binaries; the offline fixture does not.

In the real-SDK proof, `SourceAdmission::ingest` signs a complete castore source but does not export a physical root. The fixture exports each exact signed PathInfo into its own store, compares the exported and persisted identity with physical NAR read-back, and rechecks the signature before binding the loader, app source, utility, keystore or password. A castore-only signed source does not make a native sandbox input available.

Both authoring domains require at least `1980-01-01T00:00:02Z`, their default epoch, with locale `C` and timezone `UTC`. Their upper limits follow distinct renderers: the published adapter stamps sorted files with BusyBox `touch -t`, then runs `jar --create` **without** `--date`, retaining the DOS ZIP upper endpoint `2107-12-31T23:59:59Z` (4354819199); the reviewed adapter uses `jar --update --date` and admits through `2099-12-31T23:59:59Z` (4102444799).

An isolated, non-SDK check with the installed OpenJDK 17.0.20+2 and the pinned static BusyBox consumed both upper-endpoint archives successfully; ZIP's two-second timestamp representation stored nominal second 59 as 58. The reviewed `--date` option rejected the first 2100 timestamp. That observation supports the distinct renderer contracts, not execution of the pinned Temurin SDK cohort, APK/DEX construction, signing, or publication.

The separate current-source embedded-authoring smoke (2026-10-17) used the
public string-deserialization API for both published and reviewed routes
across 14 date cases. It observed unsigned plans ending at zipalign, signed
plans declaring apksigner, and rejection of v4 signing and a substituted JDK
identity. This is evaluation/declaration evidence, not execution of an Android
SDK, APK/DEX construction, signing, or publication.

`apksigner` reads the keystore and private-key passwords independently. The direct adapter requires distinct regular files for `STORE_PASSWORD_FILE` and `KEY_PASSWORD_FILE` (their contents may match), in a signed store object separate from the app source; using one path twice exhausts its password stream. Only the signing derivation declares that object and the independently signed keystore object.

For the separately tracked signed Java example, reserve at least 16 GiB of free scratch space for two fresh store/state pairs and one perturbed rebuild; the adapter's fixed-output archive guard permits at most 2 GiB per archive. A connected original-HTTPS fetch or a matching prefetched source bundle is required before the offline `SourceFetchOverridePlan` replay. The sandbox needs user namespaces, Bubblewrap, a declared static shell/utility, and signed store admission of the runtime loader, its shared libraries, the reviewed SDK/JDK/platform archives, source tree, and separately held signing materials. If any prerequisite or verifier is absent, record a blocker and produce no success receipt. No APK proof is a device/emulator install, launch, or runtime-behavior claim.

Set `TMPDIR` to spacious private scratch **before** entering the locked Nix devshell; launch the fresh-build runner once inside that shell. It does not reevaluate the full read-only snapshot flake for each SDK command or verifier invocation. Invoke the snapshot tool with Python and `b3sum` from locked Nixpkgs outputs on `PATH`; pass the same interpreter and explicit `--python`/`--b3sum` paths to the runner. Pass `--bwrap` from the locked, signed `pkgs.bubblewrap` output: the rail checks the full tool output signature and invokes that executable directly for preflight, test-only keytool and reviewed `apksigner verify`. The historical durable-publication receipt binds `flake.nix`, so APK proof tools are passed explicitly instead of changing the shared devshell. The Cargo target must be **fresh for each immutable proof run**, private, and outside the source snapshot; reusing one can leave read-only vendored `bzip2-sys` headers that fail on the next compile.

Run `scripts/prove-android-apk.py --help` for reviewed-archive and private-target arguments, including `--busybox` from locked `pkgs.pkgsStatic.busybox` and `--python`/`--b3sum`/`--bwrap` from that locked cohort. Verify signatures and retain Nix GC roots under private scratch for the first independently SHA-256-pinned Mantle executable and signed verifier tools **before** the long proof; `nix-store -r <output> --add-root <scratch-root>` prevents host GC from deleting an input between build and receipt recheck. Launch the proof **detached with pueue**, capture its full task output, and preserve its scratch run directory. The rail imports or connects to the original-HTTPS SDK/JDK source bundle, proves its pinned offline `SourceFetchOverridePlan` replay, exports `examples/android-minimal.ncl`, generates one **test-only** keystore, and runs an ignored real-Bubblewrap proof in three independent store/state directories. It writes a checked-in success receipt only after APK structure, reviewed `apksigner verify`, a signed-region tamper rejection, two identical BLAKE3 digests and a changed-Java digest are all observed. On any missing prerequisite or mismatch it writes `evidence/official-build-blocker.json` and no new success receipt. Private production signing keys must never be placed in a content-addressed store.

The official rail MUST run from a separately materialized, read-only full-source snapshot, not a shared root while other owners edit `crunch-build` or `Cargo.lock`. Capture only after `BACKEND TEST SNAPSHOT DONE`, CA final source-ready, the backend's one serialized manifest/lock/vendor transaction with stable lock, and the S0 B2/root integrator second-source stability barrier. Then `scripts/snapshot-android-apk-source.py --source . --snapshot <fresh-scratch-source> --manifest <scratch-manifest.json>` copies tracked edits and untracked owner source files with per-file BLAKE3 and pre/post mode/content checks. It rejects escaping symlinks and active vendored-source replacement rather than silently omitting an ignored dependency. The canonical source-profile BLAKE3 and manifest BLAKE3 are Mantle's content identities; the additionally printed NAR SHA-256 is **only** the Nix tree-interoperability check. Invoke the proof script **from the copied script in that snapshot**, passing `--evidence-dir <original-change-evidence-directory>`, `--source-snapshot-manifest`, `--expected-source-profile-blake3`, `--expected-source-manifest-blake3`, `--expected-source-nar-sha256`, and `--expected-mantle-sha256`. The first Mantle binary and prefetched source bundle are separately pinned; use `--expected-bundle-blake3` for a prefetched bundle. The rail rechecks the source tree, manifest, binary and bundle after actual SDK verification and refuses a success receipt on drift.

If all three SDK builds and the ignored test finished but post-build verification was interrupted, `scripts/verify-android-apk-proof.py --help` identifies the independently pinned inputs for an offline verification-only run. Execute it from a **second read-only full-source snapshot**, with the completed build's immutable snapshot and manifest pinned separately. It rechecks both complete source trees, the first executable, original reviewed HTTPS archives and bundle, signed Python/BLAKE3/BusyBox/Bubblewrap/native-runtime roots, three fresh archive outcomes, all three distinct store/state outputs and the exact passing real-build log. It runs the reviewed extracted `apksigner` in signed Bubblewrap for each APK, flips one signed DEX payload byte and requires verification failure; matching clean and differing Java-perturbed BLAKE3 digests are mandatory. Its receipt records both source snapshots and the verification mode. A missing or failed build cannot be recovered by this verifier.
The proof rehashes every included source member, including `Cargo.lock` and the APK, build and store owner crates, by BLAKE3 before and after compilation. Ignored `vendor-deps/` is recorded as excluded only because the proof Cargo command uses `--offline` without opt-in `.cargo/vendor-config.toml`, and the active `.cargo/config.toml` does not replace registry sources; an active replacement fails snapshot capture, and any needed omitted dependency fails the proof with a blocker rather than being silently treated as verified. NAR SHA-256 never substitutes for these Mantle-owned source identities.

The signed example enables only APK Signature Scheme v2 and declares Android `minSdkVersion="24"`, the first API level supporting that scheme; the proof keystore is not a release credential.

## Native libraries and assets

Both published and reviewed plans now accept optional `assets` (default `[]`)
and `native` (default `null`). An asset is `{ source_member, destination }`,
where `destination` is relative to APK `assets/`. A native package declares
`abi`, `api_level`, `ndk_revision`, `ndk_sysroot`, and `libraries`; each library
declares a signed producer output `store_file`, APK `filename`, and
`required_jni_exports`. These are explicit inputs, not host search paths.
Signing remains the existing final, separate stage; native compilation,
inspection, linking and packaging never receive signing inputs.

The initial declaration matrix is **arm64-v8a / aarch64-linux-android**, API
**24..35**, NDK **r28c / 28.2.13676358**, Linux x86-64 execution host. This is a
bounded source-level contract, not a statement that every matrix row has been
executed. The minimal proof cohort selects API 24 and Rust **1.90.0**, with
separately pinned x86_64-unknown-linux-gnu host and aarch64-linux-android std
archives in `tests/fixtures/android-native-minimal/cohort.json`. The existing
SDK cohort remains build-tools 35.0.0, platform 35 r02 and Temurin 17.0.17+10;
neither source-v1 nor reviewed-record-v1 identities or timestamp endpoints
change. NDK acquisition uses an ordinary independently SHA-256-pinned FOD,
not a forged fourth member of either existing SDK cohort.

`android.native.fetchNdk` pins
`https://dl.google.com/android/repository/android-ndk-r28c-linux.zip` to SHA-256
`dfb20d396df28ca02a8c708314b814a4d961dc9074f9a161932746f815aa552f`.
Google's [NDK archive table](https://github.com/android/ndk/wiki/Unsupported-Downloads#r28c)
records revision, URL, 722261334-byte size and SHA-1; the SHA-256 is corroborated
by the [F-Droid SDK transparency log](https://gitlab.com/fdroid/android-sdk-transparency-log/-/raw/master/checksums.json).
The Rust archive hashes come from the official
[host checksum](https://static.rust-lang.org/dist/rust-1.90.0-x86_64-unknown-linux-gnu.tar.xz.sha256)
and [Android std checksum](https://static.rust-lang.org/dist/rust-std-1.90.0-aarch64-linux-android.tar.xz.sha256).
Versions and path labels are never substitutes for those bytes, the signed
store content, or the actual producer/config/source identities.

### Stages and independent inspection

1. Acquire the pinned NDK and Rust archives explicitly; offline builds require
   their existing admitted source bundle, never a user Android or Cargo home.
   `android.native.extractNdk { archive, utility_root, utility_member }`
   rechecks the exact archive digest and lowers to an ordinary extraction
   derivation. Realize it before naming its actual exported output.
2. `android.native.mkLinker { ndk_tree, glibc, libgcc, utility_root,
   utility_member }` creates an ordinary declared tool closure. It wraps both
   clang **and ld.lld** with the explicit glibc loader: wrapping clang alone
   leaves its child linker requesting absent host `/lib64`. It also exports
   llvm-ar, llvm-ranlib and llvm-readelf wrappers. `ndk_tree`, glibc, libgcc and
   utility_root name complete independently admitted store objects; the NDK
   tree retains its `android-ndk-r28c/` archive root.
3. Feed `android.native.targetLinker { wrapper, ndk_tree, glibc, libgcc,
   api_level }` into the generic Cargo unit-plan declaration **and** its
   immutable config. Its exact arguments select the API target, declared
   sysroot, and max/common ELF page sizes of 16384 bytes. Keep `linker` as the
   separate host linker; `target_linker` applies only to target units. Pin the
   installed Rust host/target std closure, Cargo.lock, workspace/path packages,
   vendor roots, features, `no_default_features`, profile, native-input roles,
   flags, and host/runtime inputs in that config. The existing role declaration
   contract and generic producer handle build.rs and proc macros; there is no
   Android scheduler or second Cargo resolver. See the reusable
   `tests/fixtures/android-native-minimal/native.ncl` function.
4. Select the real admitted plan's cdylib root through
   `cargo-unit-bindings.json`, then read its compiler output's
   `share/mantle/unit-dependencies-v1.json`; never guess an output path or
   select a host proc macro by a similar filename. The proof checks host
   build.rs/proc-macro bindings and actual x86_64 ELF artifacts separately
   from the Android root. Bind that root's declared cdylib through the
   parameterized
   `tests/fixtures/android-native-minimal/apk.ncl` function and invoke the
   **existing** `crunch-android-apk` renderer and `mantle build` flow.

Both APK adapters inspect ELF64 little-endian AArch64 ET_DYN inputs before
rendering executable APK steps. Inspection checks dynamic metadata against
the loader's mapped tables, rejects interpreters/RPATH/RUNPATH/text relocations,
checks declared JNI exports and SONAME, walks DT_NEEDED, and resolves strong
imports against that root's dependency group and the declared API-specific NDK
stubs. A sibling library's exports cannot satisfy an import unless the loader
will reach it through DT_NEEDED. NDK stubs are API evidence only when their
complete tree is bound to the pinned extraction by ordinary signed build
admission; a caller-provided revision string or arbitrary signed tree is not
official NDK provenance.
Every declared library's mapped Android API note must also equal the selected
API. Missing notes and a higher/lower stamped API fail rather than being
inferred from a filename. The note is not compiler-content authority.
Native artifact and NDK sysroot store objects must also remain distinct from
the keystore and both password objects, even when their member paths differ.
Only the final signing derivation receives the separate secret objects.


Every native PT_LOAD must have power-of-two alignment at least 16384 and
congruent file/virtual offsets. The existing package stage installs declared
libraries as uncompressed `lib/arm64-v8a/<filename>`, adds declared assets,
then runs **`zipalign -P 16 -f 4`** and **`zipalign -c -P 16 4`**. Four-byte
ZIP alignment alone does not establish 16 KiB native-library alignment. Native
API level also reaches `aapt2 link --min-sdk-version` and D8 `--min-api`.
Published packaging still uses fixed file timestamps and jar-create without
`--date`; reviewed packaging retains its bounded jar-update `--date` renderer.

### Real remote proof driver and required run manifest

`scripts/prove-android-native.py --manifest <private-run-manifest.json>
--output <fresh-private-proof-directory>` runs cold/warm native and existing
APK stages, a changed-feature native rebuild, an independently admitted changed
asset source, independent ZIP-offset/ELF/JNI/API/dependency inspections,
`zipalign` checks and `aapt2 dump badging`. If `apk_signing` is supplied, it
also runs the declared `apksigner verify` and requires verification to reject
a tampered DEX. It creates no key or credentials and performs no installation.
Launch the real proof detached on the parent's immutable source snapshot;
no mid-flight SDK/build/test execution was used to justify this extension.

The run manifest requires concrete, pre-admitted values (no placeholder store
paths):

- `mantle`, `apk_adapter`, `b3sum`, `store`, `store_prefix`, `state_dir`,
  `store_signing_key`, `store_trusted_public_key`; this driver requires the same
  logical and physical store namespace. Bootstrap/signature-check the BLAKE3
  tool independently, as for the existing official APK rail.
- `native_recipe`, `changed_native_recipe`, `apk_plan_factory`, `apk_inputs`,
  `changed_asset_inputs`, `ndk_sysroot`, `artifact_stem`, `library_filename`,
  `required_jni_exports`, `assets`, `api_level`, `workload_identity`; optional
  `apk_signing` contains only the existing declared signing metadata, never
  password/key bytes. The plan factory accepts the same parameters as the
  fixture's `apk.ncl`. The minimal artifact stem is
  `libmantle_android_native_minimal`; library filename adds `.so`.
- `tools` maps `readelf`, `zipalign`, `aapt2` and, when signing, `apksigner` to
  exact argv prefixes using pinned executable/loader/library/JAR paths. No
  ambient executable lookup or network acquisition belongs in the driver.
- `pinned_files` is an array of `{ path, blake3 }` covering executables,
  verifier members, recipes, all imported configs/manifests and source snapshot
  files. Pin complete runtime/store roots through ordinary admission too;
  these file checks do not replace store signatures or extraction provenance.
- `negative_native_recipes` has exactly `wrong-toolchain`,
  `missing-dependency`, `undeclared-input`, `network` entries, each
  `{ recipe, diagnostic }`. Use a Rust closure without Android std for the
  first, omit the fixture's path macro package for the second, and the
  `try-undeclared` / `try-network` feature variants for the latter two.
  Diagnostics for these pinned fixture variants are respectively
  `can't find crate for`, `failed to load manifest`,
  `undeclared input denied by native sandbox`, and
  `network attempt denied by native sandbox`.
  The driver requires both failure and the declared diagnostic in the actual
  structured failure/current saved build log. It also directly exercises wrong
  ABI/API/NDK declarations and an otherwise supported API whose actual ELF note
  disagrees. For undeclared-input isolation it exclusively creates a real
  host-only marker outside the store, then removes only that marker; a
  pre-existing marker is a blocker. The network negative uses a real
  host-loopback listener at 127.0.0.1:41237 and first proves the host can connect
  to it, avoiding any assumption that public Internet connectivity works.
  The sandboxed build script must be unable to reach that listener; an occupied
  port blocks the proof rather than becoming a fake network-denial success.

For the minimal fixture, default features return 42 from JNI; `answer-two`
returns 43. Its build.rs checks distinct host/target triples, and a real local
proc macro executes on the host. These source declarations and unexecuted
proof driver are **not** a successful pinned-toolchain build receipt. Run the
driver and retain its reports before claiming the minimal acceptance item.
Its receipt distinguishes `build_structure: observed` from
`device_execution: not-performed`; JNI symbol inspection is not device JNI
execution or a production-readiness claim.

The representative Herdr workload is an explicit remaining prerequisite:
provide the authoritative repository and immutable revision, Android/native
source, Cargo.lock and dependency acquisition/vendor closure, actual native
dependencies, required JNI exports and asset manifest. The inspected local
`ogulcancelik/herdr` checkout describes a terminal agent multiplexer; its
README/Cargo manifest and inspected Onixpkgs recipes do not supply the requested
Android application/JNI wrapper and native/asset cohort. That checkout, unrelated
public projects, and generated large assets must not be cited as representative
Android success. No native SDK execution,
representative build, signature observation or acceptance closure is claimed
by this source change alone.
