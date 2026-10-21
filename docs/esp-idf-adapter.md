# ESP-IDF firmware adapter

The adapter is limited to **ESP-IDF 5.5.1**, commit
`fcae32885b0296b32044cb99ecbdc50d98dddb83` (annotated release tag object
`9d5ecb2edeb901c879b131e2b6a57b97e6ea4665`), on `x86_64-linux`, targeting
`esp32s3` (Xtensa) and `esp32c3` (RISC-V). It does not install an SDK, resolve
components, flash hardware, provision devices, or handle signing/encryption keys.

## Authority and acquisition

`crunch-esp-idf-core` is an alloc/no_std plan validator. `crunch-esp-idf`
lowers a plan to an ordinary input-addressed derivation; its admitted
`mantle-esp-idf run` executable invokes the pinned Python closure. The generic
Mantle input/output admission and scheduler remain the only store/execution
trust boundary. Neither the adapter nor version checks authorize input bytes.
Every role is a **whole admitted immutable store directory**, accompanied by a
BLAKE3 content inventory; all role roots appear in the derivation's inputs.
The runner itself must be a role. Runtime libraries/interpreters must also be
explicit roles, not ambient `/lib`, `/usr`, `PATH`, or another store view.

A role identity is BLAKE3 of UTF-8 canonical JSON, `sort_keys=True`,
`separators=(",", ":")`, `ensure_ascii=True`, containing a sorted array of
`[relative_path, kind, payload, executable]` entries. Directories have kind
`directory`, empty payload and false executable. Files have kind `file`, their
BLAKE3 hex digest and whether any execute bit is set. Symlinks have kind
`symlink`, their exact link text and false executable. Symlink targets must
resolve inside declared role roots. Special files and oversized inputs fail.
Root location and mtimes are not tree authority; file bytes, executable bits,
layout, empty directories, and link targets are. The plan additionally binds
logical store roots, source/configuration paths, ordering and policy.

### Exact supported tool cohort

| Input | Required identity / preparation |
| --- | --- |
| SDK | The exact commit above, **all recursive gitlinks initialized**, clean checkout. Snapshot helper writes fixed `version.txt` and `mantle-source.json`, removes Git administrative directories. All source/submodule bytes are bound by the admitted tree identity. |
| Xtensa | `esp-14.2.0_20241119`, `xtensa-esp-elf-14.2.0_20241119-x86_64-linux-gnu.tar.xz`, SHA-256 `e3e6dcf3d275c3c9ab0e4c8a9d93fd10e7efc035d435460576c9d95b4140c676` |
| RISC-V | `esp-14.2.0_20241119`, `riscv32-esp-elf-14.2.0_20241119-x86_64-linux-gnu.tar.xz`, SHA-256 `7faaa86d272f3e43c233f8a5ffeba327673224a752c2eb72394655d5e7950000` |
| CMake / Ninja | 3.30.2 / 1.12.1; independently acquired, admitted directory trees, including their complete host runtime closures |
| Python | An admitted self-contained interpreter/environment with `bin/python` and all ESP-IDF `tools/requirements/requirements.core.txt` dependencies and transitive dependencies installed. Pin the complete resulting tree and every external runtime role. Interpreter must support the SDK and the exact manager below; interpreter/version/distribution inventory belongs in the acquisition record, not an ambient virtualenv. |
| Component manager | Exactly `idf-component-manager==2.4.2` in that Python closure. Frozen execution consumes this API, not a floating manager installation. |
| Hashing | Python `blake3`, installed into the same admitted Python closure; pin its exact bytes with the closure inventory. |
| Python constraints | Acquired `espidf.constraints.v5.5.txt`, represented by explicit `python_constraints` member; bytes bound by its role. Copied to private IDF_TOOLS_PATH for upstream offline dependency checks. No constraints fetch during execution. |
| Utilities | Any tools needed by SDK/component CMake, including Git and shell, supplied through explicit `path` directory members; all ELF interpreter/shared-library closures admitted separately. No system utility fallback. |
| Runner | `mantle-esp-idf` built from this source, admitted with its complete host runtime closure; executable selected with `{ role, member }`. |

Toolchain archive URLs are under
`https://github.com/espressif/crosstool-NG/releases/download/esp-14.2.0_20241119/`.
SHA-256 is only the upstream download interoperability boundary. Verify archive
bytes **before** unpacking or any interpreter/RPATH relocation. Record relocation
commands and original archive checksums with acquisition evidence, then compute
BLAKE3 inventories of the final admitted trees. Include both compiler closures
when the selected SDK configuration uses the RISC-V ULP compiler on an Xtensa
application. Executable path/version alone is never a content pin.

Preparation is a separate operator step. For example, with explicitly selected
acquisition Git/Python executables, recursively check out the exact SDK commit,
then:

```sh
python scripts/prepare-esp-idf-inputs.py snapshot-sdk --git /absolute/acquisition/git SDK_CHECKOUT SDK_SNAPSHOT
python scripts/prepare-esp-idf-inputs.py inventory complete-plan.json
```

The helper refuses dirty, wrong-commit or incomplete checkouts. Review and pin
its printed role inventories; it does not silently update a build plan. Package
SDK snapshot under a role member such as `sdk/`. Package the real checked-in
fixture under a role containing `project/` and `component/`. Admit these trees,
toolchain trees, the runner, and **every runtime/import closure** using Mantle's
existing signed cache admission. `mantle store pull --from CACHE --all
--trusted-public-keys KEY` is the proof driver's admission path. A missing root
must fail; do not mount a host SDK or untracked host `/nix/store` as a fallback.

There is deliberately no checked-in fabricated tree hash, wheel lock, NAR hash,
or successful firmware receipt. A real cohort record must supply the acquired
complete closures and exact content identities. Without those inputs the
real-toolchain proof is unavailable, not passed. No network installation or
wheel resolution happens in the build driver.

## Configuration and components

Plan `common_defaults` are applied in order, then `target_defaults` in order,
then explicit `sdkconfig`, if any. Later assignments win; an explicit
`# CONFIG_X is not set` is an assignment of `n`. Files are project-relative and
must be declared by the project tree. Defaults with adjacent automatic target
suffixes are **not** implicitly read: name all target defaults in the ordered
list. The merged configuration lives in the private workspace, never the input
project. Upstream Kconfig resolves it with an explicitly empty defaults file.
Conflicting target, reproducibility, hidden-path or custom-partition settings
fail instead of being silently overridden. Adapter policy requires upstream
`APP_REPRODUCIBLE_BUILD` and `COMPILER_HIDE_PATHS_MACROS`. The declared partition
CSV must exist; generated partition configuration and binaries are retained.

Project-local components are part of the project role. Additional local or
managed components are `{name, source={role,member}, managed}` entries. They
cannot shadow project-local content. Managed component names use the SDK's
`namespace__component` directory form. Source roles contain actual verified
component contents, not download URLs.

Acquisition may use upstream component management to prepare **one lock for each
target** and complete component trees, including upstream component hash files.
Declare the original `idf_component.yml` files, exact `dependencies.lock`
member, and component contents together. Do not reuse a lock for another target.
During execution an admitted Python wrapper replaces only upstream
`download_project_dependencies` with a frozen consumer. It uses upstream
`ProjectRequirements`, `LockManager`, target checks and upstream content-hash
validation, then returns upstream `DownloadedComponent` records. Upstream CMake
registration and manifest requirements injection remain intact. It never calls
the version solver, registry version checks, or component fetcher. This is a
release-specific offline policy adapter, not a generic CMake substitute or a
second resolver.

The native adapter's exported offline CMake guard is reused strictly as policy:
preloaded with `-C` before project code, with disconnected FetchContent/CMP0170
and an ExternalProject rejection module. The current guard also rejects
`file(LOCK)` rather than silently changing lock lifetime semantics. These
compatibility guards do not replace the generic sandbox's network boundary.

The frozen consumer rejects missing/stale manifest hash, direct dependency set
changes, target/SDK mismatch, unsupported local dependency sources, missing
component hashes/content, incompatible targets, content hash drift and extra
managed directories. It does not rewrite the lock. Build configuration also
rejects secret/provisioning options; sandboxed projects receive no device or
user-secret capability. Generic sandbox networking remains disabled.

## Nickel authoring

After the embedded stdlib registration, author an ordinary firmware root:

```nickel
let esp = import "esp_idf.ncl" in
let plan = import "complete-plan.json" in
{
  firmware = esp.mkFirmware {
    plan,
    runner = { role = "runner", member = "bin/mantle-esp-idf" },
    store_prefix = "/nix/store",
  },
}
```

`complete-plan.json` is the serialized `crunch_esp_idf_core::Plan`: name, host,
release, SDK commit, target; `roles={role:{root,tree_blake3}}`; project/SDK/Python/
Python-constraints/CMake/Ninja/compiler member references; ordered `path` directory
members; nullable sdkconfig and dependencies_lock; ordered common/target defaults;
partition CSV; component entries; source_date_epoch and explicit project_version.
No arbitrary environment or extra build arguments are accepted. Compiler member
must end in `xtensa-esp32s3-elf-gcc` or `riscv32-esp-elf-gcc` for the corresponding
target; PATH must resolve that name to the declared compiler. The core validates
before invocation; Nickel rejects unsupported cohorts/undeclared role references.
`mantle-esp-idf lower PLAN.json STORE_PREFIX RUNNER_ROLE RUNNER_MEMBER` produces
the same ordinary derivation envelope for tooling clients.

The Rust core's `validate(Plan)` consumes and returns the admitted plan without
copying its input graph. `validate_member(String)` similarly returns the admitted
relative member or a typed error. Lexical borrowing stays private to the core.
The shell's `validate`, `lower`, and `run` consume the plan; `lower` also consumes
the runner member. Serialized plans and derivation/provenance formats are
unchanged; neither ownership nor lexical validation grants store authority.

For the checked-in fixture set `project.member="project"`, local component
`{name="mantle_message",source={role="fixture",member="component"},managed=false}`,
`common_defaults=["config/common.defaults"]`,
`target_defaults=["config/esp32s3.defaults"]` or the c3 counterpart,
`partition_csv="partitions.csv"`, sdkconfig/dependencies_lock null,
`project_version="1.0.0"`, `source_date_epoch=1700000000`.

## Output and reproducibility evidence

```
out/
  firmware/app.elf
  firmware/app.map
  firmware/app.bin
  firmware/bootloader.bin
  firmware/partition-table.bin
  firmware/extra-OFFSET.bin       (additional upstream flash images, if any)
  sdkconfig
  sdkconfig.json
  flash-layout.json
  provenance.json
```

`flash-layout.json` preserves upstream offsets, settings and write-flash
arguments, translating paths to **artifact-relative** firmware paths. Every
referenced image must resolve within the build output. The manifest records the
complete plan, canonical plan BLAKE3, and raw BLAKE3 hashes of every other emitted
artifact. No flashing occurs.

Environment is cleared at both the Rust/Python and Python/tool boundaries.
HOME, temporary files, component cache and IDF_TOOLS_PATH are private; IDF_PATH,
IDF_TARGET and tools come only from the plan. Python user-site loading is
suppressed; upstream dependency checking uses explicit constraints and the
admitted interpreter environment. SOURCE_DATE_EPOCH, UTC, C locale, Python hash
seed, explicit project version and SDK version.txt control metadata. Upstream
reproducible-build flags plus file-prefix maps remove workspace/SDK path effects;
`-Werror=date-time` rejects application timestamp macros. Cache/ccache inheritance
is disabled. Source/configuration/layout/SDK/toolchain/component role identities
and policy all affect the generic derivation identity.

`prove-esp-idf.py` executes real `mantle build --no-substitute` jobs from two
independent signed-input-only stores and different working directories. It
requires both builds to execute, inspects ELF32 machine identity (Xtensa=94,
RISC-V=243), verifies signed outputs, compares **raw bytes** of ELF/application/
bootloader/partition firmware, and checks unchanged cache reuse. Map files and
logs are diagnostic outputs: they are not substituted for raw firmware equality;
any later path normalization of diagnostics must be a separately labelled
observation. No normalized comparison can rescue failed raw ELF/BIN equality.

The proof cohort JSON schema is `mantle-esp-idf-proof-cohort-v1`, with absolute
`input_cache`, `trusted_key`, `signing_key`, `store_prefix="/nix/store"`, runner
member, and `targets` containing both target keys. Each target supplies `plan`
and `managed_plan` (JSON filenames relative to cohort), and `changed_plans` with all six keys `source`,
`configuration`, `partition`, `sdk`, `toolchain`, `component`, and `negative_plans`
with all three keys `lock-target-mismatch`, `stale-manifest`,
`missing-offline-component`. The managed positive must contain actual acquired
managed component content; both clean managed builds must execute and pass raw
artifact equality too. All changed/negative fixture roots must have been
acquired and admitted into the cache beforehand. Changed SDK/toolchain scenarios
must remain within the supported release, with changed admitted bytes (for
example a reviewed source comment / toolchain acquisition provenance change),
not an unsupported tool version. Managed-lock negatives must come from a real
managed-component acquisition, not mocked lock-parser responses.

```sh
python scripts/prove-esp-idf.py --mantle /absolute/pinned/mantle \
  --cohort /absolute/acquired-cohort.json --work /scratch/new-esp-idf-proof
```

The driver also creates digest-drift, path-escape, undeclared-role,
unsupported-target, toolchain-mismatch and missing-component negatives.
`receipt.json` is written only after all required positive, cache, invalidation
and negative scenarios succeed. Contract tests in the no_std crate,
`crates/crunch-esp-idf/tests/nickel_contract.rs`, and
`crates/crunch-esp-idf/src/test_driver.py` are distinct from this integration
proof; none constitutes firmware compilation evidence. Real compilation and
byte equality must be reported as unobserved until that driver actually succeeds.

Upstream references: [build system](https://docs.espressif.com/projects/esp-idf/en/v5.5.1/esp32/api-guides/build-system.html),
[pinned tool metadata](https://github.com/espressif/esp-idf/blob/v5.5.1/tools/tools.json),
[reproducibility](https://docs.espressif.com/projects/esp-idf/en/v5.5.1/esp32/api-guides/reproducible-builds.html),
[manager 2.4.2 frozen-policy boundary](https://github.com/espressif/idf-component-manager/blob/v2.4.2/idf_component_manager/dependencies.py).
