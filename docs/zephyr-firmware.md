# Pinned offline Zephyr firmware

Mantle's `crunch-zephyr-core` validates a bounded firmware plan;
`crunch-zephyr`/`mantle-zephyr` lowers it to an **ordinary admitted derivation**.
There is no firmware scheduler, west resolver, private store, downloader, or
special sandbox. `lib/zephyr.ncl` provides the corresponding typed Nickel helper.

Rust core entry points consume owned data: `validate(Plan)` returns the admitted
wire plan, `admit(Plan)` returns it with resolved tool/configuration/module paths,
and `admit_runner(RunnerRequest { plan, runner })` returns it with its builder
path. Borrowed role/member resolution remains private. `Member`, `Tool`, and
named `Rejection { code, detail }` records belong to this core. The shell's
`identity`, `lower`, and `build` consume the plan, and `lower` consumes the runner.
Plan JSON, identity hashing, derivation environments, and provenance bytes keep
the existing formats; resolved paths are pure data, not input admission.

**Evidence status:** the checked-in contracts, real application, and executable
proof driver are not a compiler receipt. The implementation session did not
have the prepared Zephyr SDK/source/Python closure and was explicitly prohibited
from builds or checks mid-flight. Do not mark the offline-build or byte-comparison
acceptance criteria complete until the real driver finishes against immutable
integrated source and its signed input cohort. Missing prerequisites fail; there
is no mock compiler or fallback output.

## Deliberately narrow cohort

| Input | Exact selection |
|---|---|
| Build host | `x86_64-linux` |
| Zephyr | `v3.7.0`, peeled commit `36940db938a8f4a1e919496793ed439850a221c2` |
| Board | `qemu_cortex_m3`, no board revision or qualifier |
| Application | `tests/fixtures/zephyr/app`, one C source, explicit `prj.conf` and `uart.overlay` |
| Resolved module set | CMSIS only: `4b96cbb174678dcd3ca86e11e1f24bc5f8726da0`, upstream manifest path `modules/hal/cmsis` |
| SDK/target | Zephyr SDK `0.16.8`, `arm-zephyr-eabi` toolchain |
| CMake / Ninja | `3.30.5` / `1.12.1` |
| Python | `3.11.10`, no west installed |
| Python packages | PyYAML `6.0.2`, pykwalify `1.8.0`, pyelftools `0.31`, packaging `24.1`, anytree `2.12.1`, intelhex `2.3.0` |
| Other host generators | dtc `1.7.0`, GNU gperf `3.1` |
| Time/parallelism | source epoch `1721992744`, one build job in the fixture |

Python's complete installed environment, including transitive distributions,
interpreter, loader and shared libraries, must be content-pinned and admitted.
The direct distribution versions above are checked before configure; transitive
packages are bound by the environment's whole-tree identity rather than resolved
at build time. Do not install west into that environment: explicit modules make
it unnecessary, and its workspace search would add no authority.

Selection sources:

- [Zephyr 3.7.0 board documentation](https://docs.zephyrproject.org/3.7.0/boards/qemu/cortex_m3/doc/index.html)
  names this Cortex-M3/LM3S6965 **emulation** board and explicitly disclaims
  physical-hardware suitability. No QEMU executable or emulator execution is
  needed to produce firmware artifacts.
- [Zephyr 3.7.0 source tag](https://github.com/zephyrproject-rtos/zephyr/releases/tag/v3.7.0)
  and [peeled tag object](https://api.github.com/repos/zephyrproject-rtos/zephyr/git/tags/1d761236dc4a3ec1d833f1773594891a334f19a6)
  bind the selected upstream source revision.
- [That release's west.yml](https://github.com/zephyrproject-rtos/zephyr/blob/v3.7.0/west.yml)
  supplies the exact CMSIS revision/path. This fixture disables module discovery
  and passes **only** CMSIS through `ZEPHYR_MODULES`; unrelated upstream modules
  are not silently downloaded or claimed as part of the supported build closure.
- [Zephyr 3.7.0 getting started guide](https://docs.zephyrproject.org/3.7.0/develop/getting_started/index.html)
  selects SDK 0.16.8; the [SDK release](https://github.com/zephyrproject-rtos/sdk-ng/releases/tag/v0.16.8)
  publishes its archives/checksums.
- [Release Python requirements](https://github.com/zephyrproject-rtos/zephyr/blob/v3.7.0/scripts/requirements-base.txt)
  explain the Python build dependencies. The exact newer patch versions above
  are an adapter cohort selection, not an upstream claim that arbitrary Python
  environments are equivalent.

No other board, revision, multi-image/sysbuild, shield, external module set,
Python environment, SDK release, host architecture or signing workflow is claimed.
Unsupported boards/qualifiers and changed upstream revisions reject early rather
than creating a plausibly named output. Patched content at an unchanged selected
upstream revision is still distinct content and must have a newly reviewed tree
identity; the recorded revision alone never authenticates it.

## Acquisition is a separate operation

Prepare sources, tools and their runtime closure **before** a Mantle build. The
adapter never runs `west update`, `git fetch`, `pip`, an SDK installer, or an
online dependency manager. Fetch exact Zephyr/CMSIS commits, verify them against
the selected upstream identities, export Git trees without `.git`/`.west`, and
preserve the acquisition receipt. Do not pass a working west workspace as source.

The conservative SDK source is the official full Linux x86_64 archive:

```text
https://github.com/zephyrproject-rtos/sdk-ng/releases/download/v0.16.8/zephyr-sdk-0.16.8_linux-x86_64.tar.xz
SHA256 cb4e4012751e4526aaf1ec1e8ab9b4ded5681e2e01711b64f7a1b519ff7dbc6a
```

For reduced acquisition/storage, the same release publishes these separately
pinned components (assemble them during acquisition, not configure):

```text
zephyr-sdk-0.16.8_linux-x86_64_minimal.tar.xz
SHA256 c323afcd22149ef0f217759a478bf8b3a42d2e36c89b653a0b875db88d7677e4

toolchain_linux-x86_64_arm-zephyr-eabi.tar.xz
SHA256 2f77a13e419e9b3bf113cb54eab6060869719477a459453434ec2f4ad536611d

hosttools_linux-x86_64.tar.xz (only if its host tools are included)
SHA256 9fc16936f1697341b3c64255f39aec2a78cc3e9a1d71b6edc9ebadb7eb781882
```

These SHA256 values are from the official
[0.16.8 `sha256.sum`](https://github.com/zephyrproject-rtos/sdk-ng/releases/download/v0.16.8/sha256.sum),
not invented expected results. They authenticate archive-byte selection, not
Mantle-owned identity. The selected SDK role is the unpacked SDK root containing
`sdk_version`, `cmake/Zephyr-sdkConfig.cmake`, and
`arm-zephyr-eabi/bin/arm-zephyr-eabi-gcc`.

On Nix-style hosts, upstream SDK executable loaders may reference `/lib64` or
host libraries that are deliberately absent from the sandbox. Acquire/package
them into a declared closure first, with explicit ELF interpreters/RPATHs pointing
to admitted runtime roots. If SDK setup/relocation or `patchelf` is required, do
it in the preparation derivation, record its exact input/tool identities, and
pin the **resulting** SDK tree. Merely admitting the SDK directory does not infer
its ELF loader. Do not mount host `/lib`, `/usr`, a whole Nix store, or inject
ambient `LD_LIBRARY_PATH` to make the build pass. Nix-packaged CMake/Python tools
likewise need their complete signed runtime closure.

Every role has `{ root, tree_blake3 }`. Roots must be whole logical store roots,
not host source paths or unchecked subdirectories. Independently record each
prepared role's content digest with the same immutable `mantle-zephyr` executable:

```sh
/pinned/bin/mantle-zephyr seal-tree /nix/store/<digest>-prepared-input
```

`mantle-zephyr-tree-v1` hashes sorted, length-delimited relative names and entry
kinds, file lengths/content/executable bits, directories, and literal symlink
targets. It deliberately ignores mtimes, uid/gid and write permission bits.
The root location is not part of this tree digest; the complete logical root
**is** part of the plan. Limits are 400,000 entries, depth 64 and 8 GiB of file
content per role. Symlinks must resolve within a separately declared role;
missing, special, non-UTF-8 or escaping inputs fail before any compiler executes.

A version file, URL, Git SHA, expected digest or store-looking string is not
admission authority. The operator must independently pin the prepared contents
and import signed PathInfo/NARs through Mantle's existing verified store path.
For a prepared Nix cohort, reuse the signed file-cache workflow:

```sh
nix copy --offline --to "file://$CACHE?compression=none&secret-key=$SIGNING_KEY" "${INPUT_ROOTS[@]}"
mantle --nix-compat --store "$INPUT_STORE" --state-dir "$INPUT_STATE" \
  store pull --from "$CACHE" --trusted-public-keys "$TRUSTED_PUBLIC_KEY" --all
```

Include each source, fixture, runner, tool and runtime root in `INPUT_ROOTS`.
Inspect the pull report for rejected signatures/hash mismatches/missing NARs;
never add `--trust-unsigned`. The input state used by the proof must contain
**inputs only**, not an already realized firmware root.

## Minimal Nickel authoring

Prepare `roles.ncl` as the record of the exact signed roots and independent tree
hashes. Required fixture roles are `zephyr`, `cmsis`, `sdk`, `fixture`, `runner`,
`python`, `cmake`, `ninja`, `dtc`, and `gperf`, plus every referenced runtime role.
`fixture` contains the complete `tests/fixtures/zephyr` directory, not just `app`;
`runner` contains the current-source `bin/mantle-zephyr`. Tool member names and
versions are explicit in the checked-in cohort recipe.

```nickel
let roles = import "/scratch/zephyr/roles.ncl" in
let bindings = (import "tests/fixtures/zephyr/cohort.ncl") roles in
(import "tests/fixtures/zephyr/firmware.ncl") bindings
```

`firmware.ncl` calls `zephyr.mkFirmware { plan, runner }`. A plan names the SDK,
Zephyr, project and resolved manifest roles, exact module revisions, board and
qualifiers, ordered Kconfig fragments/devicetree overlays, host tools, source
epoch and job bound. Authoring does not grant tool/network access. The helper
emits the generic derivation envelope; `mantle-zephyr lower PLAN.json runner
bin/mantle-zephyr` exposes the same envelope for other frontends. No new Mantle
CLI subcommand or scheduler dispatch is required.

Obtain the wire plan for the driver with the pinned Nickel CLI, using the real
bindings (this is evaluation, not firmware evidence):

```sh
nickel export --format json /scratch/zephyr/plan.ncl > /scratch/zephyr/plan.json
```

Here `plan.ncl` is `(import "bindings.ncl").plan`, and `bindings.ncl` evaluates
the checked-in `cohort.ncl` function over `roles.ncl`. Retain the selected fixture layout.
Nickel runtime `zephyr.ncl` module must be embedded alongside `cmake.ncl` in the
built Mantle binary; rebuild the integrated binary after changing either.

## Actual execution and output layout

The builder checks every input tree, resolves every symlink within the admitted
closure, compares the admitted `resolved-west.json` with the typed plan, and
checks CMSIS against **the admitted Zephyr source's** `west.yml`. It checks SDK
version/target and exact host/Python distribution versions before configure.
It creates a fresh `/build/mantle-zephyr`, clearing inherited environment;
only declared tool directories enter PATH, and no ambient `ZEPHYR_BASE`, SDK
registry, HOME, CMake cache, west workspace, compiler flags, CPATH or host library
path survives. `ZEPHYR_BASE`/SDK/config/overlay/module paths are then set from the
plan, explicitly. User and package registries/ccache are disabled. Generated
configuration and devicetree remain build outputs, not assumed inputs.

CMake/Ninja run inside Mantle's ordinary network-denied sandbox. CMake's shared
offline guard rejects `file(DOWNLOAD/UPLOAD)`; ExternalProject is rejected and
FetchContent is disconnected. These compatibility guards supplement, not replace,
the sandbox's network isolation.
The shared guard also rejects `file(LOCK)` rather than changing caller-function
lock lifetimes. A future source/tool cohort needing this operation is unsupported;
do not suppress that diagnostic to obtain a nominally successful build.

The single signed `out` root contains:

```text
zephyr.elf                 ELF32 little-endian ARM executable
zephyr.bin                 raw firmware
zephyr.hex                 Intel HEX firmware
zephyr.map                 linker map
.config                    resolved Kconfig
zephyr.dts                 resolved devicetree
include/generated/zephyr/devicetree_generated.h
manifest.json              mantle-zephyr-provenance-v1
```

`manifest.json` binds the entire normalized plan and its domain-separated BLAKE3
identity, exact observed host tool versions, all source/module/tool/runtime roots
and tree identities, and BLAKE3 for each artifact. Manifest bytes are part of the
ordinary signed output tree. It is not a substitute for signed output admission.

## Real build proof versus contract tests

Build the dedicated runner from the same locked, immutable integrated checkout
inside the pinned development shell with `cargo build --locked -p crunch-zephyr
--bin mantle-zephyr`. Package/admit it with its complete runtime closure before
the firmware proof; a root-only `cargo build -p mantle` does not build this binary.

After the parent integrates and checks immutable source, launch this driver
**detached** on a scratch filesystem with enough room for independent stores and
an additional SDK copy:

```sh
python3 scripts/prove-zephyr-firmware.py \
  --mantle /snapshot/bin/mantle --expected-mantle-blake3 "$MANTLE_BLAKE3" \
  --runner /snapshot/bin/mantle-zephyr --expected-runner-blake3 "$RUNNER_BLAKE3" \
  --plan /scratch/zephyr/plan.json --run-dir /scratch/zephyr/proof-new \
  --base-state "$INPUT_STATE" --trusted-public-key "$TRUSTED_PUBLIC_KEY" \
  --signing-key "$SIGNING_KEY" --b3sum /pinned/bin/b3sum
```

Use the pinned development shell's reviewed bubblewrap/static sandbox shell;
record their identities, observer/interpreter identities and source snapshot
with the external proof evidence. The provided runner must be identical to the
admitted runner. The script neither builds Mantle nor mutates lifecycle/Git or
acquires missing prerequisites.

Successful execution writes `proof.json` only after:

1. A first **real** compiler build reports one built root and valid ARM ELF,
   config, DTS, map and all firmware artifact hashes.
2. An unchanged run uses signed cache admission, with zero compiler builds.
3. An independent clean state/store, containing only the same signed inputs,
   executes the compiler again and compares **all seven artifacts byte for
   byte**, independently checking their manifest hashes.
4. Source, Kconfig, overlay, CMSIS content and SDK content variations are created
   through ordinary admitted Python derivations, not host edits to signed roots.
   With the derivation name held fixed, each changes the plan and generic output
   identity and rebuilds. Source/config/overlay controls must change BIN bytes.
5. Missing module, wrong revision, unsupported board/qualifier and escaping
   configuration members reject early. Content mismatch, mismatched acquired
   resolved manifest, escaping symlink and an actual attempted CMake download
   each fail with the expected diagnostic; there is no successful fallback.

Revision/board changes outside the finite cohort are explicitly rejected, not
silently reinterpreted as supported variants. Module/SDK content-only controls
exercise conservative content binding without pretending that an unchanged
revision/version string authenticates changed content.

Rust contract tests cover missing/misresolved modules, undeclared/escaping
members, unsupported board/qualifier/tool versions and canonical tree-content
invariants. They are distinct from the real compiler proof above. A compile-only
or skipped test is not evidence for steps 1–5.

## Reproducibility and scope limits

The proof compares clean builds with the **same logical input roots**, source
snapshot, tool/runtime closure, host cohort and store prefix, using a fixed work
path, source epoch, locale, timezone and Python hash seed. It does not prove
byte identity after changing store prefixes, source paths, SDK relocation,
compiler versions, arbitrary boards, host architectures or source/debug path
normalization. ELF/map/generated DTS may intentionally contain declared source
paths even when BIN/HEX are stable. The driver fails on any remaining clean
artifact difference instead of normalizing or suppressing it; retain failed
logs and investigate the specific difference before making a reproducibility
claim. No compiler-correctness or hardware-runtime guarantee follows.

This adapter never flashes, discovers devices, monitors serial ports, runs
QEMU, provisions hardware, accesses signing secrets, or signs firmware. Mantle
still signs its own normal store metadata. Hardware execution is a separate,
explicit workflow outside this artifact-only adapter.
