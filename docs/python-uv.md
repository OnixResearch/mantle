# Python/uv lock and PEP 517 adapter

## Boundary and supported cohort

`crunch-python-core` is a no-std admission core. `crunch-python` builds the
`mantle-python` std process adapter, with an embedded worker launched using the
**declared** interpreter's `-I -S` flags. `lib/python.ncl` lowers explicit inputs
to the ordinary Mantle native derivation. There is no Python scheduler, store,
receipt authority, dependency resolver, downloader, or pip repair path.

The Rust admission API consumes owned values: `admit_target(Target)` returns the
admitted `Target`, `admit_artifacts(Vec<Artifact>)` returns the admitted artifact
vector, and `admit_selections(Vec<String>)` returns the admitted selection vector.
Each returns `Result<_, String>` with the existing `python-*` diagnostics on
failure. The launcher moves configuration fields through admission without
cloning them; their JSON representation and declared ordering are unchanged.

The supported fragment is intentionally bounded:

| Input | Supported |
|---|---|
| uv | Exactly 0.9.5, independently admitted tool/runtime closure |
| Lock | `uv.lock` version 1, revision 3; freshness checked by real `uv export --locked --offline`, never `--frozen` |
| Interpreter | CPython, exact declared `3.12.<patch>`, Linux x86_64; no managed-Python lookup/download |
| Tags | Explicit nonempty subset of `py3-none-any`, `cp312-cp312-linux_x86_64`; no implicit manylinux, musllinux, abi3, cross-compilation or host-kernel tag widening |
| Markers | PEP 508 via hash-verified `packaging==25.0`, with a fixed declared Linux/CPython marker environment; `platform_release` and `platform_version` are rejected |
| Project | PEP 621 `pyproject.toml`, explicit extras and PEP 735 groups; no default groups, conflicting extras, index credentials or `uv.toml` |
| Workspace | Exact top-level member-directory names, no globs/exclusions; all declared members selected; `tool.uv.sources` accepts only `{ workspace = true }` |
| Package sources | HTTPS registry/direct archives with SHA-256; Git with a full 40-hex locked commit and an explicitly admitted immutable snapshot; workspace source roots |
| Source archives | Regular-file/directory `.tar.gz`, one top-level directory; no traversal, symlinks, hardlinks, devices or ambiguous names |
| Backend | `setuptools.build_meta`, `setuptools==80.9.0`, `wheel==0.45.1`, explicit exact-pinned static and dynamic build closure; no backend-path or backend config-settings |
| Native inputs | Existing `NativeInputDeclaration` roles/markers, no discovery; explicit `CC`, `LDSHARED`, `AR` and complete tool/header/link/runtime roots |

At most 256 lock packages/artifacts, 64 selections per selection class, 128 wheel
rows per lock package, 16,384 entries per source/output tree and 64 MiB per input
file/archive expansion are admitted. The configuration is bounded to 1 MiB.
Ambiguous versions for the selected target fail rather than choosing a resolver
fork. Unsupported shapes fail with `python-*` diagnostics.

`uv` owns lock freshness and dependency graph export; Mantle never resolves a
second graph. `packaging` owns version/requirement/tag parsing. Wheel filenames,
METADATA name/version/Requires-Python, WHEEL tags, RECORD SHA-256/size rows and
ZIP paths are validated before installation. Runtime wheel metadata dependencies
must belong to the selected lock closure; backend dependencies must belong to
the separately declared pinned build closure.

The pinned historical backend cohort is **not a blanket security endorsement**.
The published wheel 0.45.1 advisory concerns its CLI `unpack` path (not called:
uv installs prevalidated wheels); setuptools 80.9.0's published Unicode exclusion
advisory concerns macOS sdist creation (this cohort builds wheels on Linux).
Untrusted backend/source execution still requires the native sandbox and an
explicit operator source trust decision. These pins grant no arbitrary remote
code trust and support no publishing operation.

## Acquisition, planning and offline execution

1. Declare upstream coordinates and hashes through existing `mantle.fetchurl`,
   `fetchTarball` or `fetchGit`. The fixture's
   `tests/fixtures/python-uv/acquire.ncl` uses the generic fetcher for the three
   pinned standards/backend wheels. Its `upstream-wheels.json` contains exact
   PyPI artifact URLs and SHA-256 values, not a package-index credential.
2. Export/import/pin source bundles using the existing generic source commands.
   A wheel/sdist directory is `package-mirror:<identity>:<directory>`, not a new
   source kind or private download cache. A Git snapshot must be acquired with
   its locked revision **and independently reviewed recursive content hash** via
   the existing Git source transport. A path string alone is not Git proof.
3. Admit tools, configuration, project, mirror payloads and all runtime/native
   roots through the existing store boundary. The Nickel recipe binds their
   exact store roots into derivation inputs. Mantle's normal PathInfo/signature
   checks remain the authority; the adapter's manifest is not an admission or
   trust receipt.
4. `mode = "plan"` validates the lock/export and emits acquisition declarations
   without installing packages or running a backend. An artifact's `path = ""`
   means declared-but-not-acquired and is allowed **only** in this mode; its
   filename/HTTPS URL/hash remain mandatory. The verified packaging wheel and
   declared interpreter/uv/project must be present to run the planner. A plan
   always reports `offline_ready = false` and does not probe dynamic hooks.
5. `mode = "environment"` accepts selected wheels only. `uv pip install` receives
   only local verified wheel URLs, `--offline --no-index --no-deps --no-build
   --require-hashes --no-cache --no-installer-metadata`. No lock URL is handed to
   the installer. The output contains isolated `site-packages/` and an executable
   `python SCRIPT [ARGS...]` launcher using the declared interpreter's `-IS`.
6. `mode = "wheel"` additionally builds explicitly named `build_packages` from
   sdist/Git/workspace source using the declared backend. Static
   `[build-system].requires` must equal `build_requirements`; each requirement
   must be an exact `name==version` pin with one matching verified wheel. The
   backend environment is a fresh, separate target directory installed offline.
   Its metadata dependency closure is checked, not recursively downloaded.
   `get_requires_for_build_wheel({})` is run in that environment inside the
   network-denied sandbox. Its result must exactly equal the separately
   exact-pinned `dynamic_requirements` before `build_wheel` is invoked. Unpinned,
   URL, marker-bearing, extra-bearing or missing dynamic requirements are
   rejected; setuptools `setup_requires` never gets an installer fallback.

Wheel-only environments omit workspace project installations (uv's
`--no-emit-workspace`), not their dependency graph. Run source scripts through
the isolated launcher, or build the desired workspace packages as wheels first
and feed those wheels into a second locked environment. There are no editable
installs. Workspace wheel dependencies must all be selected/built explicitly;
the adapter does not infer a build order or execute an undeclared build backend.

The Rust launcher clears its environment. The worker creates private HOME,
XDG config/cache and uv cache directories; credentials, Python user sites,
PYTHONPATH, user pip config and ambient uv configuration are absent. Backend
Python processes use `-I -S` and only the verified backend site is prepended.
`.pth` files are **not executed**. This is not a general virtualenv or entry-point
launcher contract. `UV_PYTHON_DOWNLOADS=never` and explicit interpreter arguments
prevent interpreter discovery/download. An ordinary Mantle native sandbox is
required to deny network for arbitrary backend child processes; uv's offline
flag alone is not such a sandbox. Calling the adapter directly on an unrestricted
host does not prove network isolation or source trust.

## Configuration and minimal Nickel recipe

The adapter reads `MANTLE_PYTHON_CONFIG` (JSON), `out`, and the evaluated input
bindings emitted by `lib/python.ncl`. Unknown configuration fields fail closed.
A config contains:

- `schema: 1`, `mode`, `store_prefix`, `python` and `uv` executable paths,
  `project` store root;
- `target: { python, implementation: "cpython", abi: "cp312", platform:
  "linux_x86_64", wheel_tags }`;
- explicit arrays `extras`, `groups`, `members`, `build_packages`,
  `build_requirements`, `dynamic_requirements`, `runtime_inputs`;
- `artifacts: [{ filename, url, sha256, path }]`, where `path` is a file inside
  an admitted root (or empty in acquisition-only planning);
- `packaging: "packaging-25.0-py3-none-any.whl"`, `source_date_epoch` (ZIP UTC
  range, at least 1980), and `native_inputs: { inputs, env, path }`;
- `git_sources: { normalized_package_name: { revision, path } }` for admitted
  Git snapshots, or `{}`.

The artifact URL must match its locked archive URL exactly. Build wheel origins
are separately explicit in `artifacts` because uv.lock is not a build-system
lockfile. Artifact/backend/interpreter/native roots are all inputs, not tool
versions looked up by name. The executable's bytes, embedded worker, normalized
config, project/lock bytes, Git source tree bytes, mirror payload bytes, complete
store-root declarations, interpreter version, selections and backend pins bind
the BLAKE3 plan identity. Derivation identity additionally binds the admitted
builder, config and input roots through the existing pipeline.

A minimal recipe, after preparing the two JSON files with actual admitted paths:

```nickel
let python = import "python.ncl" in
python (import "python-inputs.json")
```

`python-inputs.json` contains `name`, `adapter` (executable store path), `config`
(the JSON file path within an admitted root), `python` and `uv` (**tool roots**,
not executables), `project`,
`artifacts` (unique acquired **roots**), `native_inputs`, `runtime_inputs`, and
optional `git_sources` (snapshot root array). Its evaluated native/runtime
bindings must equal those in the config; all config paths must be under the
recipe's declared roots. `lib/python.ncl` uses the normal input-addressed native
sandbox, not dynamic plans or a helper store.

Native role values use the existing grammar, for example `CC =
"{{role:cc}}/bin/cc"`, `LDSHARED = "{{role:cc}}/bin/cc -shared"`, `AR =
"{{role:binutils}}/bin/ar"`, `CFLAGS = "-O2 -g0"`, and role-qualified include,
link and rpath flags. `native_inputs.path` can expose declared compiler,
binutils, pkg-config, CMake and Ninja `bin` directories, never `/usr/bin`.
CMake adapter outputs may supply `{{role:native}}/include`, `/lib` and
`/lib/pkgconfig`, using its normal admitted output layout. The adapter does not
rewrite Python's native closure or run an ambient CMake/Cargo repair.

## Provenance and reproducibility limits

`manifest.json` records the BLAKE3 input identity, target/selections, original
source coordinates, hashes and acquired state, pinned backend/native inputs,
output BLAKE3 file digests, wheel SHA-256, METADATA/WHEEL SHA-256 and ZIP timestamp
sets. Its `adapter` object has **exactly** the existing
`SourceAdapterMetadata { adapter, lock_identity, offline_control,
generated_source_boundary, extra }` shape. A source-bundle API caller can bind it
without a Python-specific source transport:

```rust
let metadata: SourceAdapterMetadata = serde_json::from_value(manifest["adapter"].clone())?;
let mirror = SourceSpec {
    kind: SourceRecordKind::PackageMirror,
    identity: mirror_identity,
    path: acquired_mirror,
    adapter: Some(metadata),
};
```

This is generic metadata, not a fake store receipt. Existing CLI `--source`
syntax admits the same PackageMirror payload without adapter metadata; source
bundle readiness and adapter output do not assert build trust or reproducibility.

Native wheel ZIP entries must use the declared `SOURCE_DATE_EPOCH`, rounded to
ZIP's two-second precision. Clean comparison must compare **wheel bytes**, wheel
metadata and timestamp sets, not just the filename. Backend-produced debug
paths, generated source, compiler flags, library rpaths, locale and native
closure can still change wheel bytes. The fixture uses explicit `-g0` compiler
flags; no general binary rewriting or auditwheel relocation is performed.

The environment is not a portable virtualenv: its launcher pins an absolute
interpreter store path. Native wheels can embed native store rpaths and depend
on the declared Python/native runtime closure. Moving to another logical store
prefix, changing that closure, or expecting ambient libraries is unsupported.
The site directory can be moved only while preserving the interpreter and
native closure; build manifests retain the original provenance. No relocation
or cross-machine compatibility claim follows from a successful local import.

## Exact real proof, prerequisites and expected evidence

No real-tool acceptance is claimed by this change's source alone. After the
integrated build, run the driver detached on a spacious Linux host with bwrap,
user namespaces, a writable isolated physical store, and the normal Mantle
sandbox prerequisites:

```text
python3 scripts/prove-python-uv.py \
  --mantle /absolute/pinned/mantle \
  --adapter /absolute/pinned/mantle-python \
  --nix /absolute/pinned/nix \
  --inputs /absolute/reviewed-python-inputs.json \
  --signing-key /absolute/proof-signing-key \
  --trusted-public-key NAME:BASE64 \
  --run-dir /absolute/new-spacious-run-directory
```

The host Python is only the proof orchestrator; it is not an adapter tool input.
Nix is an explicit proof-only local NAR/hash and signed cache producer; the
adapter and normal build path do not invoke it. The caller supplies a real key
pair and must independently pin the Mantle binary/source revision as part of
the enclosing source-bound proof.

`reviewed-python-inputs.json` must contain:

- `python`: actual admitted Nix CPython executable, `python_version`: its exact
  `3.12.<patch>`, and `uv`: actual admitted uv 0.9.5 executable;
- `adapter_sha256`: independently measured built adapter file SHA-256;
- `roots`: **complete** list of `{ path, nar_sha256 }` for interpreter, uv,
  compiler/binutils, libc/headers/linker and all runtime/native dependencies.
  The driver checks every NAR against these pins and checks that offline
  recursive Nix closure enumeration equals this exact declared set; it never
  silently adds a tool or native dependency;
- `native_inputs`: the complete existing role declaration described above,
  including `CC`, `LDSHARED`, `AR` and deterministic compiler flags;
- `artifacts`: exactly three `{ filename, url, sha256, path }` rows, matching
  `upstream-wheels.json`, with prefetched local files. No path or cache fallback
  is attempted for a missing file.

The exact upstream artifact pins were read from
[packaging 25.0](https://pypi.org/pypi/packaging/25.0/json),
[setuptools 80.9.0](https://pypi.org/pypi/setuptools/80.9.0/json), and
[wheel 0.45.1](https://pypi.org/pypi/wheel/0.45.1/json). The lock schema and flags
are bound to [uv 0.9.5's lock implementation](https://github.com/astral-sh/uv/blob/0.9.5/crates/uv-resolver/src/lock/mod.rs)
and [CLI definitions](https://github.com/astral-sh/uv/blob/0.9.5/crates/uv-cli/src/lib.rs).

The proof builds an owned standards-valid pure wheel and a deterministic owned
native sdist from `tests/fixtures/python-uv/native`; their `.invalid` HTTPS
coordinates identify **only the owned fixtures**, not real upstream registry
publication. The backend wheels are the actual pinned upstream releases.

The driver uses real generic source export/import/pin/preflight and fixed-output
fetch builds, signs every resulting Mantle output, imports independently
hash-pinned complete tool roots through the signed cache boundary, and verifies
trusted signatures before consuming outputs. It then:

- builds and runs the pure environment (`mantle-pure-answer=42`);
- observes trusted warm reuse with zero new builds, and extras/groups identity
  invalidation;
- proves acquisition-only planning reports missing material without readiness;
- builds an actual CPython C extension wheel from the locked **sdist**, installs
  that wheel with real offline uv, and imports it (`mantle-native-answer=42`);
- rejects wrong ABI/platform/interpreter, stale lock, missing lock hashes,
  tampered payload hashes, missing build wheels, undeclared dynamic hook
  requirements, changed backend pins and backend network attempts;
- changes C source and lock bytes, requiring new identities and changed native
  wheel bytes;
- repeats from a second fresh physical store/state and compares clean native
  wheel bytes, metadata and timestamps.

Every command's argv/stdout/stderr and build reports remain in `--run-dir`.
Only a completed run writes `evidence.json` with `clean_comparison`. Any missing
prerequisite, real tool error, failed signature, unexpected cache behavior or
wheel difference aborts. The bounded result is not universal uv compatibility,
compiler correctness, trust in arbitrary Python code, or a release-parity claim.
