"""Effectful Python worker, always launched with the declared interpreter -I -S.

Not a resolver/downloader. uv owns freshness/PEP 508 export. packaging owns PEP
440/508/tag interpretation. The surrounding Mantle native sandbox denies network.
"""
import base64
import csv
import hashlib
import io
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import urllib.parse
import zipfile

MAX_BYTES = 64 * 1024 * 1024
MAX_ENTRIES = 16384
UV_VERSION = "0.9.5"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def bounded(path):
    path = Path(path)
    require(path.is_file() and not path.is_symlink(), f"python-input-kind: {path}")
    require(path.stat().st_size <= MAX_BYTES, f"python-input-size: {path}")
    return path.read_bytes()


def digest(path):
    return hashlib.sha256(bounded(path)).hexdigest()


def run(args, **kwargs):
    result = subprocess.run(args, check=False, text=True, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, **kwargs)
    require(result.returncode == 0,
            f"python-tool-failed: {args[0]}: {result.stderr[-16384:]}")
    require(len(result.stdout) <= MAX_BYTES, "python-tool-output-limit")
    return result.stdout


def safe_member(name):
    p = Path(name)
    require(name and p.parts and not p.is_absolute() and ".." not in p.parts and "\\" not in name
            and "//" not in name and name.rstrip("/") == p.as_posix()
            and all(ord(character) >= 32 for character in name), f"python-archive-path: {name}")
    return p


def validate_wheel(path, expected_name, expected_version, tags, fixed_epoch=None):
    name, version, _, wheel_tags = parse_wheel_filename(path.name)
    require(name == canonicalize_name(expected_name) and str(version) == str(Version(expected_version)),
            f"python-wheel-name-version: {path.name}")
    require(bool(wheel_tags & tags), f"python-wheel-abi-platform: {path.name}")
    with zipfile.ZipFile(path) as archive:
        rows = archive.infolist()
        require(len(rows) <= MAX_ENTRIES and sum(row.file_size for row in rows) <= MAX_BYTES,
                "python-wheel-size-limit")
        require(len({safe_member(row.filename).as_posix() for row in rows}) == len(rows), "python-wheel-duplicate")
        for row in rows:
            safe_member(row.filename)
            require((row.external_attr >> 16) & 0o170000 != 0o120000, "python-wheel-symlink")
        metadata = [row.filename for row in rows if row.filename.endswith(".dist-info/METADATA")]
        wheel = [row.filename for row in rows if row.filename.endswith(".dist-info/WHEEL")]
        records = [row.filename for row in rows if row.filename.endswith(".dist-info/RECORD")]
        require(len(metadata) == len(wheel) == len(records) == 1, "python-wheel-metadata-missing")
        from email.parser import BytesParser
        meta = BytesParser().parsebytes(archive.read(metadata[0]))
        require(canonicalize_name(meta["Name"]) == name and Version(meta["Version"]) == version,
                "python-wheel-metadata-identity")
        if meta.get("Requires-Python"):
            require(Version(MARKER_ENV["python_full_version"]) in SpecifierSet(meta["Requires-Python"]),
                    "python-wheel-interpreter-mismatch")
        declared_tags = BytesParser().parsebytes(archive.read(wheel[0])).get_all("Tag", [])
        metadata_tags = set()
        from packaging.tags import parse_tag
        for tag in declared_tags:
            metadata_tags.update(parse_tag(tag))
        require(metadata_tags == wheel_tags, "python-wheel-metadata-tags")
        record_rows = list(csv.reader(io.StringIO(archive.read(records[0]).decode())))
        require(len({row[0] for row in record_rows}) == len(record_rows), "python-wheel-record-duplicate")
        by_name = {row[0]: row for row in record_rows}
        for row in rows:
            if row.is_dir():
                continue
            require(row.filename in by_name, f"python-wheel-record-missing: {row.filename}")
            _, encoded, size = by_name[row.filename]
            if row.filename == records[0]:
                require(encoded == size == "", "python-wheel-record-self")
                continue
            raw = archive.read(row.filename)
            wanted = "sha256=" + base64.urlsafe_b64encode(hashlib.sha256(raw).digest()).rstrip(b"=").decode()
            require(encoded == wanted and size == str(len(raw)), f"python-wheel-record-hash: {row.filename}")
        if fixed_epoch is not None:
            import time
            expected = time.gmtime(fixed_epoch)[:6]
            expected = (*expected[:5], expected[5] // 2 * 2)
            require(all(row.date_time == expected for row in rows), "python-wheel-timestamps")
        return {"filename": path.name, "sha256": digest(path),
                "metadata_sha256": hashlib.sha256(archive.read(metadata[0])).hexdigest(),
                "wheel_sha256": hashlib.sha256(archive.read(wheel[0])).hexdigest(),
                "timestamps": sorted({str(row.date_time) for row in rows}),
                "requires_dist": meta.get_all("Requires-Dist", [])}


def artifact_map(config):
    result = {}
    for artifact in config["artifacts"]:
        coordinate = urllib.parse.urlparse(artifact["url"])
        require(coordinate.scheme == "https" and coordinate.hostname and not coordinate.username
                and not coordinate.password, "python-artifact-url-credentials-or-scheme")
        available = bool(artifact["path"])
        require(available or config["mode"] == "plan", f"python-source-missing: {artifact['filename']}")
        if available:
            require(digest(artifact["path"]) == artifact["sha256"],
                    f"python-artifact-hash-mismatch: {artifact['filename']}")
        result[artifact["filename"]] = {**artifact, "available": available}
    packaging = config["packaging"]
    require(packaging == "packaging-25.0-py3-none-any.whl" and packaging in result and result[packaging]["available"],
            "python-packaging-pin: requires packaging==25.0 verified wheel")
    # Bootstrap only the explicitly hash-verified pure-Python standards parser.
    sys.path.insert(0, result[packaging]["path"])
    return result


def marker_applies(text, extras):
    if text is None:
        return True
    require(re.search(r"\b(platform_release|platform_version)\b", text) is None,
            "python-marker-kernel-unsupported")
    marker = Marker(text)
    return any(marker.evaluate({**MARKER_ENV, "extra": extra}) for extra in ["", *extras])


def pinned_requirement(text):
    requirement = Requirement(text)
    require(requirement.url is None, f"python-build-requirement-url: {text}")
    pins = list(requirement.specifier)
    require(len(pins) == 1 and pins[0].operator == "==" and "*" not in pins[0].version
            and not requirement.extras and requirement.marker is None,
            f"python-build-requirement-unpinned: {text}")
    return canonicalize_name(requirement.name), str(Version(pins[0].version))


def validate_lock(config, project):
    lock = tomllib.loads(bounded(project / "uv.lock").decode())
    require(lock.get("version") == 1 and lock.get("revision") == 3,
            "python-lock-schema: requires uv.lock version=1 revision=3")
    allowed = {"version", "revision", "requires-python", "resolution-markers", "supported-environments",
               "required-environments", "conflicts", "options", "manifest", "package"}
    require(set(lock) <= allowed, "python-lock-unknown-field")
    require(not lock.get("conflicts"), "python-lock-conflicts-unsupported")
    require(Version(config["target"]["python"]) in SpecifierSet(lock["requires-python"]),
            "python-lock-interpreter-mismatch")
    for key in ["supported-environments", "required-environments"]:
        if lock.get(key):
            require(any(marker_applies(text, []) for text in lock[key]), f"python-lock-{key}-mismatch")
    packages = lock.get("package", [])
    require(0 < len(packages) <= 256, "python-lock-package-limit")
    by_name = {}
    for package in packages:
        allowed_package = {"name", "version", "source", "dependencies", "optional-dependencies", "dev-dependencies",
                           "sdist", "wheels", "metadata", "resolution-markers"}
        require(set(package) <= allowed_package, f"python-lock-package-field: {package.get('name')}")
        archives = list(package.get("wheels", []))
        require(len(archives) <= 128, "python-lock-wheel-limit")
        if package.get("sdist"):
            archives.append(package["sdist"])
        for archive in archives:
            require(re.fullmatch(r"sha256:[0-9a-f]{64}", archive.get("hash", "")) is not None,
                    f"python-lock-missing-hash: {package['name']}")
        if package.get("resolution-markers") and not any(marker_applies(text, config["extras"]) for text in package["resolution-markers"]):
            continue
        name = canonicalize_name(package["name"])
        require(name not in by_name, f"python-lock-ambiguous-target: {name}")
        by_name[name] = package
    return lock, by_name


def export_selection(config, project, workspace):
    # uv validates pyproject/lock freshness. --locked is deliberate, NOT --frozen.
    args = [config["uv"], "export", "--locked", "--offline", "--no-config", "--project", str(project),
            "--python", config["python"], "--format", "requirements-txt", "--no-header", "--no-annotate",
            "--no-emit-workspace", "--no-default-groups"]
    for extra in config["extras"]:
        args += ["--extra", extra]
    for group in config["groups"]:
        args += ["--group", group]
    # Workspace selection is bounded to all declared members or the root project;
    # partial workspace selection is not silently broadened.
    if workspace:
        args.append("--all-packages")
    exported = run(args, cwd=project)
    exported = exported.replace("\\\n", " ")
    requirements = []
    for line in exported.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        require(not line.startswith("-"), f"python-export-unsupported-option: {line}")
        parts = re.split(r"\s+--hash=", line)
        requirement = Requirement(parts[0].strip())
        hashes = {part.strip().removeprefix("sha256:") for part in parts[1:]}
        require(all(re.fullmatch("[0-9a-f]{64}", item) for item in hashes), "python-export-hash-format")
        if requirement.marker and not marker_applies(str(requirement.marker), config["extras"]):
            continue
        requirements.append((requirement, hashes))
    require(len(requirements) <= 256, "python-export-limit")
    return requirements


def selected_artifact(package, hashes, artifacts, tags, allow_source):
    candidates = []
    for wheel in package.get("wheels", []):
        coordinate = wheel.get("url", wheel.get("path", ""))
        filename = urllib.parse.unquote(urllib.parse.urlparse(coordinate).path.rsplit("/", 1)[-1])
        if filename not in artifacts:
            continue
        require(artifacts[filename]["url"] == coordinate, f"python-lock-provenance-mismatch: {filename}")
        expected = wheel.get("hash", "").removeprefix("sha256:")
        require(re.fullmatch("[0-9a-f]{64}", expected) is not None, f"python-lock-missing-hash: {filename}")
        require(expected == artifacts[filename]["sha256"] and (not hashes or expected in hashes),
                f"python-lock-hash-mismatch: {filename}")
        try:
            _, _, _, wheel_tags = parse_wheel_filename(filename)
        except Exception as error:
            raise ValueError(f"python-wheel-name: {filename}") from error
        if wheel_tags & tags:
            candidates.append(filename)
    if candidates:
        # Explicit deterministic selection; no platform preference inferred from host caches.
        filename = sorted(candidates)[0]
        return "wheel", filename
    if allow_source and package.get("sdist"):
        sdist = package["sdist"]
        coordinate = sdist.get("url", sdist.get("path", ""))
        filename = urllib.parse.unquote(urllib.parse.urlparse(coordinate).path.rsplit("/", 1)[-1])
        expected = sdist.get("hash", "").removeprefix("sha256:")
        require(filename in artifacts, f"python-source-missing: {filename}")
        require(artifacts[filename]["url"] == coordinate, f"python-lock-provenance-mismatch: {filename}")
        require(re.fullmatch("[0-9a-f]{64}", expected) is not None and expected == artifacts[filename]["sha256"]
                and (not hashes or expected in hashes), f"python-source-hash: {filename}")
        return "sdist", filename
    raise ValueError(f"python-wheel-missing-or-abi-platform: {package['name']}")


def check_workspace(project, pyproject, config):
    tool_uv = pyproject.get("tool", {}).get("uv", {})
    require(not (project / "uv.toml").exists(), "python-project-uv-toml-unsupported")
    require(not tool_uv.get("index") and not tool_uv.get("default-groups"), "python-project-index-or-default-groups-unsupported")
    workspace = tool_uv.get("workspace", {})
    require(set(workspace) <= {"members"} and not workspace.get("exclude"), "python-workspace-exclude-unsupported")
    members = workspace.get("members", [])
    require(sorted(members) == sorted(config["members"]), "python-workspace-members-mismatch")
    for member in members:
        safe_member(member)
        require(not any(character in member for character in "*?[]"), "python-workspace-glob-unsupported")
        require((project / member / "pyproject.toml").is_file(), "python-workspace-member-missing")
    for name, source in tool_uv.get("sources", {}).items():
        require(isinstance(source, dict) and source == {"workspace": True}, f"python-project-source-unsupported: {name}")
    return members


def unpack_sdist(artifact, destination):
    require(artifact["filename"].endswith(".tar.gz"), "python-sdist-format: only tar.gz")
    with tarfile.open(artifact["path"], "r:gz") as archive:
        members = archive.getmembers()
        require(len(members) <= MAX_ENTRIES and sum(row.size for row in members) <= MAX_BYTES,
                "python-sdist-limit")
        require(len({safe_member(row.name).as_posix() for row in members}) == len(members), "python-sdist-duplicate")
        for row in members:
            safe_member(row.name)
            require(row.isdir() or row.isfile(), "python-sdist-link-or-special")
        archive.extractall(destination, filter="data")
    roots = list(destination.iterdir())
    require(len(roots) == 1 and roots[0].is_dir(), "python-sdist-single-root")
    return roots[0]


def install(config, wheels, destination, scratch):
    destination.mkdir(parents=True, exist_ok=False)
    requirements = scratch / (destination.name + "-requirements.txt")
    requirements.write_text("".join(f"{path.as_uri()} --hash=sha256:{digest(path)}\n" for path in wheels))
    if not wheels:
        return
    run([config["uv"], "pip", "install", "--offline", "--no-config", "--no-cache", "--no-index", "--no-deps",
         "--no-build", "--no-installer-metadata", "--require-hashes", "--python", config["python"], "--target", str(destination),
         "--requirements", str(requirements)])


HOOK = r'''
import importlib, json, os, sys
sys.path[:0] = [sys.argv[1]]
os.chdir(sys.argv[2])
backend = importlib.import_module("setuptools.build_meta")
if sys.argv[3] == "requirements":
    result = backend.get_requires_for_build_wheel({})
else:
    result = backend.build_wheel(sys.argv[4], {})
print("MANTLE_PEP517_RESULT=" + json.dumps(result, sort_keys=True))
'''


def backend_call(config, site, source, operation, destination):
    text = run([config["python"], "-I", "-S", "-c", HOOK, str(site), str(source), operation, str(destination)])
    results = [line.removeprefix("MANTLE_PEP517_RESULT=") for line in text.splitlines() if line.startswith("MANTLE_PEP517_RESULT=")]
    require(len(results) == 1, "python-backend-result")
    return json.loads(results[0])


def build_source(config, source, site, wheelhouse, name, version, tags):
    pyproject = tomllib.loads(bounded(source / "pyproject.toml").decode())
    build = pyproject.get("build-system", {})
    require(set(build) <= {"requires", "build-backend"} and build.get("build-backend") == "setuptools.build_meta",
            "python-backend-unsupported: setuptools.build_meta without backend-path only")
    expected = sorted(pinned_requirement(row) for row in config["build_requirements"])
    actual = sorted(pinned_requirement(row) for row in build.get("requires", []))
    require(actual == expected and ("setuptools", "80.9.0") in expected and ("wheel", "0.45.1") in expected,
            "python-backend-build-requirements-mismatch")
    # Setuptools setup_requires is never silently installed. Hook results must
    # equal the explicitly pinned dynamic closure before invoking build_wheel.
    dynamic = backend_call(config, site, source, "requirements", wheelhouse)
    expected_dynamic = sorted(pinned_requirement(row) for row in config["dynamic_requirements"])
    require(sorted(pinned_requirement(row) for row in dynamic) == expected_dynamic,
            "python-backend-dynamic-requirements-mismatch")
    result = backend_call(config, site, source, "wheel", wheelhouse)
    require(isinstance(result, str) and Path(result).name == result and result.endswith(".whl"), "python-backend-wheel-result")
    wheel = wheelhouse / result
    validate_wheel(wheel, name, version, tags, config["source_date_epoch"])
    return wheel


def main():
    config = json.loads(bounded(sys.argv[1]))
    require(sys.implementation.name == "cpython" and sys.version.split()[0] == config["target"]["python"],
            "python-interpreter-version-mismatch")
    require(sys.platform == "linux" and os.uname().machine == "x86_64", "python-platform-mismatch")
    require(sys.flags.isolated and sys.flags.no_site, "python-interpreter-not-isolated")
    artifacts = artifact_map(config)
    global Requirement, Marker, SpecifierSet, Version, canonicalize_name, parse_wheel_filename, MARKER_ENV
    from packaging.requirements import Requirement
    from packaging.markers import Marker
    from packaging.specifiers import SpecifierSet
    from packaging.version import Version
    from packaging.utils import canonicalize_name, parse_wheel_filename
    from packaging.tags import sys_tags, parse_tag
    MARKER_ENV = {"implementation_name": "cpython", "implementation_version": config["target"]["python"],
                  "os_name": "posix", "platform_machine": "x86_64", "platform_python_implementation": "CPython",
                  "platform_system": "Linux", "platform_release": "", "platform_version": "",
                  "python_full_version": config["target"]["python"], "python_version": "3.12", "sys_platform": "linux"}
    tags = set()
    for text in config["target"]["wheel_tags"]:
        tags.update(parse_tag(text))
    require(tags <= set(sys_tags()), "python-interpreter-abi-mismatch")
    output = Path(os.environ["out"])
    output.mkdir(parents=True)
    scratch = Path(tempfile.mkdtemp(prefix="mantle-python-"))
    try:
        home = scratch / "home"
        home.mkdir()
        os.environ.update(HOME=str(home), XDG_CONFIG_HOME=str(home), XDG_CACHE_HOME=str(scratch / "cache"),
                          UV_CACHE_DIR=str(scratch / "uv-cache"), UV_NO_MANAGED_PYTHON="1", UV_PYTHON_DOWNLOADS="never",
                          UV_OFFLINE="1", UV_NO_CONFIG="1", UV_NO_CACHE="1", UV_PYTHON=config["python"],
                          PYTHONNOUSERSITE="1", PYTHONHASHSEED="0",
                          PIP_NO_INDEX="1", PIP_CONFIG_FILE="/dev/null", SOURCE_DATE_EPOCH=str(config["source_date_epoch"]),
                          TMPDIR=str(scratch), SETUPTOOLS_USE_DISTUTILS="local")
        observed_uv = run([config["uv"], "--version"]).strip()
        require(re.fullmatch(r"uv 0\.9\.5(?: \([A-Za-z0-9 ._-]+\))?", observed_uv) is not None,
                "python-uv-version-mismatch")
        project = Path(config["project"])
        pyproject = tomllib.loads(bounded(project / "pyproject.toml").decode())
        members = check_workspace(project, pyproject, config)
        _, packages = validate_lock(config, project)
        selected = export_selection(config, project, members)
        require(config["mode"] != "environment" or not config["build_packages"], "python-environment-source-build-unsupported: build wheel first")
        wheelhouse = scratch / "wheels"
        wheelhouse.mkdir()
        # Normalize admitted arbitrary store filenames without modifying their bytes.
        local = {}
        for filename, artifact in artifacts.items():
            if not artifact["available"]:
                continue
            target = wheelhouse / filename
            shutil.copyfile(artifact["path"], target)
            local[filename] = target
        build_wheels = []
        build_pins = config["build_requirements"] + config["dynamic_requirements"]
        for text in build_pins:
            name, version = pinned_requirement(text)
            matching = []
            for filename in artifacts:
                if filename.endswith(".whl"):
                    parsed_name, parsed_version, _, _ = parse_wheel_filename(filename)
                    if parsed_name == name and str(parsed_version) == version:
                        if filename in local:
                            validate_wheel(local[filename], name, version, tags)
                        matching.append(filename)
            require(len(matching) == 1, f"python-build-dependency-missing-or-ambiguous: {text}")
            if matching[0] in local:
                build_wheels.append(local[matching[0]])
        build_versions = dict(pinned_requirement(text) for text in build_pins)
        for wheel in set(build_wheels):
            name, version, _, _ = parse_wheel_filename(wheel.name)
            metadata = validate_wheel(wheel, name, str(version), tags)
            for text in metadata["requires_dist"]:
                dependency = Requirement(text)
                if dependency.marker and not marker_applies(str(dependency.marker), []):
                    continue
                dependency_name = canonicalize_name(dependency.name)
                require(dependency.url is None and dependency_name in build_versions
                        and Version(build_versions[dependency_name]) in dependency.specifier,
                        f"python-build-dependency-closure-missing: {text}")
        site = scratch / "build-site"
        if config["build_packages"]:
            require(config["mode"] in ["wheel", "plan"], "python-source-mode")
            require(config["native_inputs"]["inputs"], "python-native-inputs-missing")
            require({"CC", "LDSHARED", "AR"} <= set(config["native_inputs"]["env"]),
                    "python-native-compiler-linker-archiver-missing")
            if config["mode"] == "wheel":
                install(config, sorted(set(build_wheels)), site, scratch)
        wheels = []
        acquisition = []
        selected_names = set()
        selected_package_extras = {}
        for requirement, hashes in selected:
            name = canonicalize_name(requirement.name)
            selected_package_extras.setdefault(name, set()).update(requirement.extras)
            require(name in packages, f"python-lock-package-missing: {name}")
            package = packages[name]
            require(Version(package["version"]) in requirement.specifier, f"python-lock-version: {name}")
            source = package["source"]
            require(set(source) <= {"registry", "url", "git", "editable", "virtual", "directory"} and len(source) == 1,
                    f"python-lock-source-unsupported: {name}")
            for key in ["registry", "url", "git"]:
                if key in source:
                    coordinate = urllib.parse.urlparse(source[key])
                    require(coordinate.scheme == "https" and coordinate.hostname and not coordinate.username
                            and not coordinate.password, "python-source-url-credentials-or-scheme")
            selected_names.add(name)
            if "git" in source:
                revision = urllib.parse.urlparse(source["git"]).fragment
                require(re.fullmatch("[0-9a-f]{40}", revision) is not None, "python-git-unpinned")
                declared = config["git_sources"].get(name)
                require(declared and declared["revision"] == revision, "python-git-source-missing")
                require(name in config["build_packages"], "python-git-needs-explicit-build")
                row = {"name": name, "source": source, "revision": revision, "kind": "git-snapshot", "path": declared["path"]}
                acquisition.append(row)
                if config["mode"] == "wheel":
                    copied = scratch / (name + "-source")
                    shutil.copytree(declared["path"], copied)
                    wheels.append(build_source(config, copied, site, wheelhouse, name, package["version"], tags))
                continue
            kind, filename = selected_artifact(package, hashes, artifacts, tags, name in config["build_packages"])
            acquisition.append({"name": name, "version": package["version"], "source": source, "kind": kind,
                                "filename": filename, "url": artifacts[filename]["url"], "sha256": artifacts[filename]["sha256"],
                                "available": artifacts[filename]["available"]})
            if kind == "wheel":
                if filename in local:
                    validate_wheel(local[filename], name, package["version"], tags)
                    wheels.append(local[filename])
            elif config["mode"] == "wheel":
                extracted = scratch / (name + "-unpacked")
                extracted.mkdir()
                source_root = unpack_sdist(artifacts[filename], extracted)
                wheels.append(build_source(config, source_root, site, wheelhouse, name, package["version"], tags))
        # uv excludes workspace projects from exported requirements. They are
        # admitted source roots, not omitted build dependencies.
        for name in config["build_packages"]:
            if name in selected_names:
                continue
            require(name in packages, f"python-build-package-not-locked: {name}")
            package = packages[name]
            source = package["source"]
            relative = source.get("editable", source.get("directory"))
            require(relative is not None, f"python-build-package-not-selected: {name}")
            if relative != ".":
                safe_member(relative)
                require(relative in members, "python-workspace-source-not-declared")
            source_root = project / relative
            selected_names.add(name)
            acquisition.append({"name": name, "source": source, "kind": "workspace", "path": str(source_root)})
            if config["mode"] == "wheel":
                copied = scratch / (name + "-source")
                shutil.copytree(source_root, copied)
                wheels.append(build_source(config, copied, site, wheelhouse, name, package["version"], tags))
        require(set(config["build_packages"]) <= set(packages), "python-build-package-unknown")
        # A wheel's runtime closure must be present in the selected lock graph.
        for wheel in wheels:
            parsed_name, parsed_version, _, _ = parse_wheel_filename(wheel.name)
            info = validate_wheel(wheel, parsed_name, str(parsed_version), tags)
            for text in info["requires_dist"]:
                dep = Requirement(text)
                if dep.marker and not marker_applies(str(dep.marker), selected_package_extras.get(parsed_name, set())):
                    continue
                dep_name = canonicalize_name(dep.name)
                require(dep.url is None and dep_name in selected_names and Version(packages[dep_name]["version"]) in dep.specifier,
                        f"python-wheel-undeclared-runtime-dependency: {text}")
        if config["mode"] == "environment":
            install(config, wheels, output / "site-packages", scratch)
            # An isolated launcher, not a relocatable virtualenv. Source path and
            # interpreter store closure remain explicit absolute references.
            launcher = output / "python"
            launcher.write_text(f"#!{config['python']} -IS\nimport os,sys\nsys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), 'site-packages'))\nif len(sys.argv) < 2: raise SystemExit('usage: python SCRIPT [ARGS...]')\np=sys.argv.pop(1)\nsys.argv[0]=p\nexec(compile(open(p,'rb').read(),p,'exec'), {{'__name__':'__main__','__file__':p}})\n")
            launcher.chmod(0o755)
        elif config["mode"] == "wheel":
            (output / "wheels").mkdir()
            for wheel in wheels:
                shutil.copyfile(wheel, output / "wheels" / wheel.name)
        identity = os.environ["MANTLE_PYTHON_IDENTITY"]
        manifest = {"schema": "mantle-python-v1", "plan_identity": identity, "target": config["target"],
                    "uv_version": UV_VERSION, "mode": config["mode"], "extras": config["extras"], "groups": config["groups"],
                    "members": members, "acquisition": acquisition, "native_inputs": json.loads(os.environ["MANTLE_PYTHON_RESOLVED_NATIVE"]),
                    "build_requirements": build_pins, "source_date_epoch": config["source_date_epoch"],
                    "offline_ready": config["mode"] != "plan",
                    "build_artifacts": [{"filename": row["filename"], "url": row["url"], "sha256": row["sha256"], "available": row["available"]}
                                        for row in artifacts.values() if row["filename"] != config["packaging"]],
                    "wheels": [validate_wheel(path, *[str(value) for value in parse_wheel_filename(path.name)[:2]], tags) for path in wheels],
                    "adapter": {"adapter": "python-uv-v1", "lock_identity": identity,
                                "offline_control": "uv-offline-no-index-no-build; mantle-network-denied-native-sandbox",
                                "generated_source_boundary": "only-output-wheels-or-isolated-site-packages; no-cache-export",
                                "extra": {"mirror-kind": "python-wheel-sdist", "interpreter": config["python"],
                                          "relocation": "launcher pins interpreter store path; native wheels may embed native store roots"}},
                    "non_claims": ["not an acquisition receipt or trust grant", "not universal Python/uv compatibility",
                                   "no claim of binary reproducibility until independent clean comparison"]}
        (output / "manifest.json").write_text(json.dumps(manifest, sort_keys=True, indent=2) + "\n")
    finally:
        shutil.rmtree(scratch)


try:
    main()
except Exception as error:
    print(f"python-adapter: {error}", file=sys.stderr)
    sys.exit(1)
