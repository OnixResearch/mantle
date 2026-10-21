"""Release-specific driver; executed only by the admitted Mantle runner."""
import configparser
import importlib.metadata
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys
import tempfile
from blake3 import blake3

MAX_FILES = 65536
MAX_FILE_BYTES = 2 * 1024**3


def fail(message):
    raise RuntimeError("esp-idf: " + message)


def json_bytes(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode()


def file_digest(path):
    if path.stat().st_size > MAX_FILE_BYTES:
        fail("file exceeds inventory bound: " + str(path))
    h = blake3()
    with path.open("rb") as stream:
        for data in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(data)
    return h.hexdigest()


def inside(path, roots):
    resolved = path.resolve(strict=True)
    if not any(resolved == root or root in resolved.parents for root in roots):
        fail("path escapes declared closure: " + str(path))
    return resolved


def inventory(root, roots):
    """BLAKE3(canonical JSON array [relative, kind, payload, executable])."""
    rows = []
    for parent, directories, files in os.walk(root, followlinks=False):
        for name in sorted(directories + files):
            path = Path(parent) / name
            relative = path.relative_to(root).as_posix()
            metadata = path.lstat()
            if stat.S_ISLNK(metadata.st_mode):
                inside(path, roots)
                rows.append([relative, "symlink", os.readlink(path), False])
            elif stat.S_ISREG(metadata.st_mode):
                rows.append([relative, "file", file_digest(path), bool(metadata.st_mode & 0o111)])
            elif stat.S_ISDIR(metadata.st_mode):
                rows.append([relative, "directory", "", False])
            else:
                fail("special file in declared input: " + relative)
            if len(rows) > MAX_FILES:
                fail("input exceeds inventory bound")
    return blake3(json_bytes(sorted(rows))).hexdigest()


def resolve(plan, member):
    root = Path(plan["roles"][member["role"]]["root"])
    path = root / member["member"]
    inside(path, [Path(role["root"]) for role in plan["roles"].values()])
    # argv[0] can select an Xtensa target; Python environment prefixes also
    # depend on the invoked path. Validate links without rewriting invocation.
    return path


def parse_config(paths):
    values = {}
    for path in paths:
        for line in path.read_text().splitlines():
            match = re.fullmatch(r"(CONFIG_[A-Z0-9_]+)=(.*)", line)
            unset = re.fullmatch(r"# (CONFIG_[A-Z0-9_]+) is not set", line)
            if match:
                values[match[1]] = match[2]
            elif unset:
                values[unset[1]] = "n"
            elif line.strip() and not line.startswith("#"):
                fail("malformed configuration in " + str(path))
    return values


def mandatory_config(values, target, partition):
    required = {
        "CONFIG_IDF_TARGET": json.dumps(target),
        "CONFIG_APP_REPRODUCIBLE_BUILD": "y",
        "CONFIG_COMPILER_HIDE_PATHS_MACROS": "y",
        "CONFIG_PARTITION_TABLE_CUSTOM": "y",
        "CONFIG_PARTITION_TABLE_CUSTOM_FILENAME": json.dumps(partition),
    }
    for key, value in required.items():
        if key in values and values[key] != value:
            fail("configuration conflicts with mandatory policy: " + key)
        values[key] = value
    for key, value in values.items():
        if value not in ("n", '""') and ("SIGNING_KEY" in key or key in {
            "CONFIG_SECURE_BOOT", "CONFIG_SECURE_BOOT_V2_ENABLED", "CONFIG_SECURE_FLASH_ENC_ENABLED",
            "CONFIG_SECURE_BOOT_BUILD_SIGNED_BINARIES"}):
            fail("secret/provisioning configuration outside adapter scope: " + key)


# Installed in a private executable wrapper, not in ambient sitecustomize.
# Replace ONLY acquisition with a strict frozen consumer of upstream records.
# No resolver and no alternate dependency graph is introduced.
FROZEN_MANAGER = r'''
import os, runpy, sys
REAL_PYTHON = __REAL_PYTHON__
if len(sys.argv) >= 3 and sys.argv[1:3] == ["-m", "idf_component_manager.prepare_components"]:
    from pathlib import Path
    import importlib.metadata
    if importlib.metadata.version("idf-component-manager") != "2.4.2":
        raise RuntimeError("unsupported frozen manager API")
    import idf_component_manager.core as core
    from idf_component_manager.dependencies import DownloadedComponent, check_manifests_targets
    from idf_component_tools.lock import LockManager
    from idf_component_tools.hash_tools.validate import validate_hash_eq_hashfile, validate_hashfile_eq_hashdir
    from idf_component_tools.build_system_tools import build_name, get_idf_version
    def frozen(project_requirements, lock_path, managed_components_path):
        requirements = project_requirements
        solution = LockManager(lock_path).load()
        check_manifests_targets(requirements)
        if not solution.manifest_hash or solution.manifest_hash != requirements.manifest_hash:
            raise RuntimeError("frozen manifest/lock mismatch; reacquire dependencies")
        if solution.target != requirements.target or str(solution.idf_version) != get_idf_version():
            raise RuntimeError("frozen lock target/SDK mismatch")
        if set(solution.direct_dependencies or []) != set(requirements.direct_dep_names):
            raise RuntimeError("frozen direct-dependency mismatch")
        local = {manifest.real_name for manifest in requirements.manifests}
        result = set()
        expected = set()
        for component in solution.dependencies:
            if component.name == "idf":
                continue
            name = build_name(component.name)
            if name in local or name.split("__")[-1] in local:
                continue
            directory = Path(managed_components_path) / name
            if not component.source.downloadable:
                raise RuntimeError("undeclared local dependency: " + component.name)
            if not component.component_hash or not directory.is_dir():
                raise RuntimeError("missing frozen dependency: " + component.name)
            if component.targets and requirements.target not in component.targets:
                raise RuntimeError("frozen component target mismatch")
            validate_hash_eq_hashfile(directory, component.component_hash)
            validate_hashfile_eq_hashdir(directory)
            expected.add(name)
            result.add(DownloadedComponent(str(directory), component.targets, str(component.version)))
        actual = {entry.name for entry in Path(managed_components_path).iterdir()}
        if actual != expected:
            raise RuntimeError("extra/missing frozen managed component")
        return result
    core.download_project_dependencies = frozen
    sys.argv = ["idf_component_manager.prepare_components"] + sys.argv[3:]
    runpy.run_module("idf_component_manager.prepare_components", run_name="__main__")
else:
    os.execv(REAL_PYTHON, [REAL_PYTHON, "-I"] + sys.argv[1:])
'''


def prepare(plan, workspace):
    roots = [Path(role["root"]) for role in plan["roles"].values()]
    for root in roots:
        if root.is_symlink() or not root.is_dir():
            fail("role is not a whole immutable directory: " + str(root))
    for role in plan["roles"].values():
        if inventory(Path(role["root"]), roots) != role["tree_blake3"]:
            fail("input digest drift: " + role["root"])
    if importlib.metadata.version("idf-component-manager") != "2.4.2":
        fail("Python closure must contain idf-component-manager 2.4.2")
    sdk = resolve(plan, plan["sdk"])
    if (sdk / "version.txt").read_text().strip() != "v5.5.1":
        fail("SDK version.txt mismatch")
    provenance = json.loads((sdk / "mantle-source.json").read_text())
    if provenance["commit"] != plan["sdk_commit"] or provenance["release"] != "5.5.1":
        fail("SDK acquisition provenance mismatch")
    # Recursive checkout recipe records every gitlink. Absence is never a stub.
    submodules = provenance["submodules"]
    if not submodules:
        fail("SDK submodule closure absent")
    for member, commit in submodules.items():
        if not re.fullmatch(r"[0-9a-f]{40}", commit):
            fail("invalid SDK submodule commit")
        if member.startswith("/") or ".." in Path(member).parts:
            fail("SDK submodule path escape")
        directory = inside(sdk / member, [sdk])
        if not directory.is_dir() or not any(directory.iterdir()):
            fail("missing SDK submodule: " + member)
    modules = configparser.ConfigParser()
    modules.read(sdk / ".gitmodules")
    if not {modules[s]["path"] for s in modules.sections()}.issubset(submodules):
        fail("SDK gitlink not declared by acquisition")
    project = workspace / "project"
    shutil.copytree(resolve(plan, plan["project"]), project, symlinks=False)
    excluded_inputs = ["build", "managed_components", "sdkconfig.old", ".git"]
    if plan["sdkconfig"] != "sdkconfig":
        excluded_inputs.append("sdkconfig")
    for excluded in excluded_inputs:
        path = project / excluded
        if path.exists():
            fail("generated/ambient project input forbidden: " + excluded)
    config_members = plan["common_defaults"] + plan["target_defaults"]
    if plan["sdkconfig"] is not None:
        config_members.append(plan["sdkconfig"])
    values = parse_config([project / path for path in config_members])
    mandatory_config(values, plan["target"], plan["partition_csv"])
    if not (project / plan["partition_csv"]).is_file():
        fail("partition CSV missing")
    (workspace / "sdkconfig").write_text("\n".join(k + "=" + v for k, v in sorted(values.items())) + "\n")
    (workspace / "empty-defaults").write_text("")
    components = project / "components"
    managed = project / "managed_components"
    components.mkdir(exist_ok=True)
    managed.mkdir()
    # Existing local components belong to the project tree; external additions
    # cannot silently replace one of those components.
    for component in plan["components"]:
        destination = (managed if component["managed"] else components) / component["name"]
        if destination.exists():
            fail("component shadows project-local content")
        shutil.copytree(resolve(plan, component["source"]), destination, symlinks=False)
    manifests = sorted(project.rglob("idf_component.yml"))
    if manifests and plan["dependencies_lock"] is None:
        fail("manifest has no target-specific frozen lock")
    lock = None
    if plan["dependencies_lock"] is not None:
        lock = (project / plan["dependencies_lock"]).read_bytes()
        (project / "dependencies.lock").write_bytes(lock)
    wrapper = workspace / "python"
    real_python = str(resolve(plan, plan["python"]))
    wrapper.write_text("#!" + real_python + " -I\n" + FROZEN_MANAGER.replace("__REAL_PYTHON__", repr(real_python)))
    wrapper.chmod(0o755)
    return sdk, project, wrapper, lock


def execute(plan):
    # Sandbox /build is private and writable. Different physical workspaces are
    # supported; prefix maps, not diagnostic rewriting, control artifact paths.
    workspace = Path(tempfile.mkdtemp(prefix="esp-idf-", dir=Path.cwd()))
    sdk, project, python, lock = prepare(plan, workspace)
    build = workspace / "build"
    home = workspace / "home"
    home.mkdir()
    tools = home / "idf-tools"
    tools.mkdir()
    shutil.copyfile(resolve(plan, plan["python_constraints"]), tools / "espidf.constraints.v5.5.txt")
    modules = workspace / "modules"
    modules.mkdir()
    (modules / "ExternalProject.cmake").write_text(
        'message(FATAL_ERROR "esp-idf: ExternalProject must be acquired as a separate admitted input")\n')
    guard = workspace / "offline-guard.cmake"
    guard.write_text(os.environ["MANTLE_ESP_IDF_CMAKE_GUARD"])
    env = {
        "PATH": ":".join(str(resolve(plan, member)) for member in plan["path"]),
        "HOME": str(home), "TMPDIR": str(workspace), "LC_ALL": "C", "TZ": "UTC",
        "SOURCE_DATE_EPOCH": str(plan["source_date_epoch"]), "PYTHONNOUSERSITE": "1",
        "PYTHONDONTWRITEBYTECODE": "1", "PYTHONHASHSEED": "0",
        "IDF_PATH": str(sdk), "IDF_TARGET": plan["target"], "IDF_TOOLCHAIN": "gcc",
        "IDF_VERSION": "5.5.1",
        "IDF_TOOLS_PATH": str(home / "idf-tools"), "IDF_COMPONENT_MANAGER": "1",
        "IDF_PYTHON_ENV_PATH": str(resolve(plan, plan["python"])).rsplit("/bin/", 1)[0],
        "IDF_COMPONENT_CHECK_NEW_VERSION": "0", "IDF_COMPONENT_CACHE_PATH": str(home / "component-cache"),
        "CCACHE_DISABLE": "1", "ZERO_AR_DATE": "1",
    }
    compiler = resolve(plan, plan["compiler"])
    selected = shutil.which(compiler.name, path=env["PATH"])
    if selected is None or Path(selected).resolve() != compiler.resolve():
        fail("PATH does not select declared target compiler")
    version = subprocess.check_output([str(compiler), "--version"], env=env, text=True)
    if "esp-14.2.0_20241119" not in version:
        fail("unsupported compiler release")
    observed_tools = {"compiler": version.strip(), "python": sys.version,
        "python_distributions": sorted((dist.metadata["Name"], dist.version)
                                       for dist in importlib.metadata.distributions())}
    for key, expected in (("cmake", "3.30.2"), ("ninja", "1.12.1")):
        version = subprocess.check_output([str(resolve(plan, plan[key])), "--version"], env=env, text=True)
        if expected not in version.splitlines()[0].split():
            fail("unsupported " + key + " release")
        observed_tools[key] = version.strip()
    flags = " ".join("-ffile-prefix-map=" + str(path) + "=" + replacement
                     for path, replacement in ((workspace, "/mantle-build"), (sdk, "/esp-idf")))
    flags += " -Werror=date-time"
    configure = [str(resolve(plan, plan["cmake"])), "-C", str(guard),
        "-S", str(project), "-B", str(build), "-G", "Ninja",
        "-DFETCHCONTENT_FULLY_DISCONNECTED=ON", "-DFETCHCONTENT_UPDATES_DISCONNECTED=ON",
        "-DCMAKE_POLICY_DEFAULT_CMP0170=NEW", "-DCMAKE_MODULE_PATH=" + str(modules),
        "-DCMAKE_MAKE_PROGRAM=" + str(resolve(plan, plan["ninja"])),
        "-DPYTHON=" + str(python), "-DIDF_TARGET=" + plan["target"],
        "-DSDKCONFIG=" + str(workspace / "sdkconfig"),
        "-DSDKCONFIG_DEFAULTS=" + str(workspace / "empty-defaults"),
        "-DPROJECT_VER=" + plan["project_version"], "-DCCACHE_ENABLE=0",
        "-DCMAKE_C_FLAGS=" + flags, "-DCMAKE_CXX_FLAGS=" + flags, "-DCMAKE_ASM_FLAGS=" + flags]
    subprocess.run(configure, env=env, check=True)
    subprocess.run([str(resolve(plan, plan["ninja"])), "-C", str(build), "-j1", "all"], env=env, check=True)
    if lock is not None and (project / "dependencies.lock").read_bytes() != lock:
        fail("upstream rewrote frozen lock")
    resolved = parse_config([workspace / "sdkconfig"])
    for key in ("CONFIG_IDF_TARGET", "CONFIG_APP_REPRODUCIBLE_BUILD", "CONFIG_COMPILER_HIDE_PATHS_MACROS",
                "CONFIG_PARTITION_TABLE_CUSTOM", "CONFIG_PARTITION_TABLE_CUSTOM_FILENAME"):
        if key not in resolved:
            fail("required resolved configuration missing: " + key)
    mandatory_config(resolved, plan["target"], plan["partition_csv"])
    output = Path(os.environ["out"])
    output.mkdir()
    firmware = output / "firmware"
    firmware.mkdir()
    description = json.loads((build / "project_description.json").read_text())
    app = description["project_name"]
    for source, destination in ((app + ".elf", "app.elf"), (app + ".map", "app.map"),
                                (app + ".bin", "app.bin"), ("bootloader/bootloader.bin", "bootloader.bin"),
                                ("partition_table/partition-table.bin", "partition-table.bin")):
        shutil.copyfile(inside(build / source, [build]), firmware / destination)
    flash = json.loads((build / "flasher_args.json").read_text())
    names = {app + ".bin": "app.bin", "bootloader/bootloader.bin": "bootloader.bin",
             "partition_table/partition-table.bin": "partition-table.bin"}
    layout = {}
    for offset, source in flash["flash_files"].items():
        if not re.fullmatch(r"0x[0-9a-fA-F]+", offset):
            fail("invalid flash offset")
        source_path = inside(build / source, [build])
        relative = source_path.relative_to(build).as_posix()
        destination = names.get(relative, "extra-" + str(int(offset, 16)) + ".bin")
        shutil.copyfile(source_path, firmware / destination)
        layout[offset] = "firmware/" + destination
    (output / "flash-layout.json").write_bytes(json_bytes({
        "target": plan["target"], "flash_files": layout,
        "flash_settings": flash.get("flash_settings", {}),
        "write_flash_args": flash.get("write_flash_args", [])}))
    shutil.copyfile(workspace / "sdkconfig", output / "sdkconfig")
    shutil.copyfile(build / "config/sdkconfig.json", output / "sdkconfig.json")
    artifacts = {path.relative_to(output).as_posix(): file_digest(path)
                 for path in sorted(output.rglob("*")) if path.is_file()}
    (output / "provenance.json").write_bytes(json_bytes({
        "schema": "mantle-esp-idf-provenance-v1", "plan": plan,
        "plan_blake3": blake3(json_bytes(plan)).hexdigest(), "artifacts_blake3": artifacts,
        "observed_tools": observed_tools,
        "network": "denied-by-mantle-sandbox", "normalization": "none; raw artifact hashes",
        "non_claims": ["no flashing", "no signing/provisioning", "no compiler correctness proof"]}))


if __name__ == "__main__":
    execute(json.loads(os.environ["MANTLE_ESP_IDF_PLAN"]))
