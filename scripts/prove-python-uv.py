#!/usr/bin/env python3
"""Real-tool, signed Mantle proof for the bounded Python cohort. Never run implicitly.

All downloads are prohibited. --inputs supplies independently reviewed tools,
complete Nix runtime/native roots and package wheel SHA-256 pins. No fake runner,
synthetic store receipt, ambient uv/pip cache, or host compiler discovery is used.
"""
import argparse
import base64
import copy
import csv
import hashlib
import io
import json
import os
import gzip
from pathlib import Path
import re
import shutil
import subprocess
import tarfile
import zipfile

REPO = Path(__file__).resolve().parents[1]
FIXTURES = REPO / "tests/fixtures/python-uv"
EPOCH = 946684800


def require(value, message):
    if not value:
        raise RuntimeError(message)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def q(value):
    return json.dumps(str(value))


def root(path):
    path = Path(path)
    require(path.parts[:3] == ("/", "nix", "store") and len(path.parts) >= 4, f"not a logical store path: {path}")
    return str(Path(*path.parts[:4]))


class Proof:
    def __init__(self, arguments):
        self.a = arguments
        self.tools = json.loads(arguments.inputs.read_text())
        self.directory = arguments.run_dir
        self.directory.mkdir(parents=True, exist_ok=False)
        self.index = 0
        self.evidence = {}
        self.nix_roots = sorted(row["path"] for row in self.tools["roots"])
        require(len(set(self.nix_roots)) == len(self.nix_roots), "duplicate declared root")
        for row in self.tools["roots"]:
            observed = self.command([arguments.nix, "hash", "path", "--algo", "sha256", "--format", "base16", row["path"]]).strip()
            require(observed == row["nar_sha256"], f"declared NAR mismatch: {row['path']}")
        closure = self.command([arguments.nix, "path-info", "--offline", "--recursive", *self.nix_roots]).splitlines()
        require(set(closure) == set(self.nix_roots), "inputs.roots must list the COMPLETE tool/native closure")
        for tool in ["python", "uv"]:
            require(root(self.tools[tool]) in self.nix_roots, f"undeclared tool: {tool}")
        require(set(self.tools["native_inputs"]["inputs"].values()) <= set(self.nix_roots), "native role not in independently pinned closure")
        require(sha(arguments.adapter) == self.tools["adapter_sha256"], "adapter binary differs from independent input pin")
        self.cache = self.directory / "signed-inputs"
        self.command([arguments.nix, "copy", "--offline", "--to",
                      f"file://{self.cache}?compression=none&secret-key={arguments.signing_key}", *self.nix_roots])
        self.artifacts = []
        pins = {row["filename"]: row for row in json.loads((FIXTURES / "upstream-wheels.json").read_text()).values()}
        for row in self.tools["artifacts"]:
            require(sha(row["path"]) == row["sha256"], f"upstream wheel pin mismatch: {row['filename']}")
            require(row["filename"] in pins and {key: row[key] for key in ["filename", "url", "sha256"]} == pins[row["filename"]],
                    "proof backend provenance differs from repository cohort")
            self.artifacts.append(dict(row))
        require({row["filename"] for row in self.artifacts} == {
            "packaging-25.0-py3-none-any.whl", "setuptools-80.9.0-py3-none-any.whl", "wheel-0.45.1-py3-none-any.whl"},
            "proof requires exact packaging/setuptools/wheel cohort")
        self.pure_wheel = self.make_pure_wheel()
        self.artifacts.append({"filename": self.pure_wheel.name, "url": "https://mantle-fixture.invalid/" + self.pure_wheel.name,
                               "path": str(self.pure_wheel), "sha256": sha(self.pure_wheel)})
        self.native_payload = self.directory / "native-payload"
        shutil.copytree(FIXTURES / "native", self.native_payload)
        (self.native_payload / "PKG-INFO").write_text("Metadata-Version: 2.3\nName: mantle-native-answer\nVersion: 1.0.0\nRequires-Python: ==3.12.*\n\n")
        self.native_sdist = self.make_native_sdist(self.native_payload, "native-sdist")
        self.artifacts.append({"filename": self.native_sdist.name, "url": "https://python-proof.invalid/" + self.native_sdist.name,
                               "path": str(self.native_sdist), "sha256": sha(self.native_sdist)})

    def command(self, arguments, negative=False):
        self.index += 1
        completed = subprocess.run([str(arg) for arg in arguments], cwd=REPO, text=True,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                   env={**os.environ, "TMPDIR": str(self.directory)})
        prefix = self.directory / f"command-{self.index:04d}"
        prefix.with_suffix(".argv.json").write_text(json.dumps([str(arg) for arg in arguments]) + "\n")
        prefix.with_suffix(".stdout").write_text(completed.stdout)
        prefix.with_suffix(".stderr").write_text(completed.stderr)
        require((completed.returncode != 0) if negative else (completed.returncode == 0),
                f"unexpected command exit {completed.returncode}: {arguments}\n{completed.stderr[-6000:]}")
        return completed.stdout

    def cli(self, arguments, negative=False):
        text = self.command([self.a.mantle, "--json", "--nix-compat", "--state-dir", self.state,
                             "--store", self.store, *arguments], negative)
        return json.loads(text)

    def build(self, text, label, negative=False, offline=False):
        recipe = self.directory / f"{label}.ncl"
        recipe.write_text(text)
        options = ["build", recipe, "--no-substitute", "--strict-hermetic", "--signing-key", self.a.signing_key,
                   "--trusted-public-keys", self.a.trusted_public_key, "--import-path", REPO / "lib"]
        if offline:
            options.append("--offline-source-preflight")
        report = self.cli(options, negative)
        (self.directory / f"{label}.report.json").write_text(json.dumps(report, indent=2) + "\n")
        if negative:
            require(report["counts"]["failed_total"] > 0, f"negative did not fail a build: {label}")
            logs = "\n".join(Path(row["saved_log_path"]).read_text() for row in report["failed"] if row.get("saved_log_path"))
            return report, logs
        rows = [row for row in report["outcomes"] if row["label"] == "python-proof"]
        require(len(rows) == 1, f"expected unique proof root: {label}")
        outputs = [row["path"] for row in rows[0]["outputs"] if row["name"] == "out"]
        require(len(outputs) == 1, f"expected unique output: {label}")
        physical = Path(outputs[0])
        self.verify(physical)
        return physical, report

    def verify(self, physical):
        text = self.command([self.a.mantle, "--nix-compat", "--state-dir", self.state, "--store", self.store,
                             "store", "verify", physical.name, "--trusted-public-keys", self.a.trusted_public_key])
        require(re.search(r"trusted_signatures=[1-9][0-9]*/[1-9][0-9]*", text), "no trusted output signature")

    def logical(self, physical):
        return "/nix/store/" + physical.name

    def fetch(self, path, label):
        text = 'let mantle = import "lib.ncl" in mantle.fetchurl {'
        text += f'name = "python-proof", url = {q(Path(path).as_uri())}, hash = {q(sha(path))} }}'
        return self.acquire(text, label)

    def acquire(self, text, label):
        recipe = self.directory / (label + "-acquisition.ncl")
        recipe.write_text(text)
        bundle = self.directory / (label + "-sources.bundle")
        self.cli(["source", "bundle", "export", "--build-root", recipe, "--import-path", REPO / "lib",
                  "--fetch-missing", "--to", bundle])
        self.cli(["source", "bundle", "import", "--from", bundle, "--pin"])
        preflight = self.cli(["source", "bundle", "preflight", "--build-root", recipe, "--import-path", REPO / "lib"])
        require(preflight["ready_class"] == "ready", "generic source preflight did not admit pinned payload")
        physical, _ = self.build(text, label, offline=True)
        self.cli(["source", "bundle", "import-store-path", "--logical", self.logical(physical),
                  "--physical", physical, "--trusted-public-keys", self.a.trusted_public_key, "--pin"])
        return self.logical(physical)

    def source(self, path, label):
        archive = self.directory / (label + ".tar.gz")
        with tarfile.open(archive, "w:gz") as writer:
            writer.add(path, arcname="source", recursive=True)
        nar = self.command([self.a.nix, "hash", "path", "--algo", "sha256", "--format", "base16", path]).strip()
        text = 'let mantle = import "lib.ncl" in mantle.fetchTarball {'
        text += f'name = "python-proof", url = {q(archive.as_uri())}, hash = {q(nar)} }}'
        return self.acquire(text, label)

    def make_pure_wheel(self):
        directory = "mantle_fixture-1.0.0.dist-info"
        entries = {
            "mantle_fixture/__init__.py": b"def answer():\n    return 42\n",
            directory + "/METADATA": b"Metadata-Version: 2.3\nName: mantle-fixture\nVersion: 1.0.0\nRequires-Python: ==3.12.*\n\n",
            directory + "/WHEEL": b"Wheel-Version: 1.0\nGenerator: mantle-owned-fixture\nRoot-Is-Purelib: true\nTag: py3-none-any\n\n",
        }
        records = io.StringIO()
        writer = csv.writer(records, lineterminator="\n")
        for name, content in sorted(entries.items()):
            encoded = base64.urlsafe_b64encode(hashlib.sha256(content).digest()).rstrip(b"=").decode()
            writer.writerow([name, "sha256=" + encoded, len(content)])
        writer.writerow([directory + "/RECORD", "", ""])
        entries[directory + "/RECORD"] = records.getvalue().encode()
        wheel = self.directory / "mantle_fixture-1.0.0-py3-none-any.whl"
        with zipfile.ZipFile(wheel, "w", zipfile.ZIP_DEFLATED) as archive:
            for name, content in sorted(entries.items()):
                info = zipfile.ZipInfo(name, (2000, 1, 1, 0, 0, 0))
                info.external_attr = 0o100644 << 16
                archive.writestr(info, content)
        return wheel

    def make_native_sdist(self, tree, label):
        destination = self.directory / label
        destination.mkdir()
        path = destination / "mantle_native_answer-1.0.0.tar.gz"
        def normalize(info):
            info.uid = info.gid = 0
            info.uname = info.gname = ""
            info.mtime = EPOCH
            return info
        with path.open("wb") as stream:
            with gzip.GzipFile(fileobj=stream, mode="wb", filename="", mtime=EPOCH) as compressed:
                with tarfile.open(fileobj=compressed, mode="w") as archive:
                    archive.add(tree, arcname="mantle_native_answer-1.0.0", filter=normalize)
        return path

    def pure_project(self, directory, artifact, distribution="mantle-fixture"):
        directory.mkdir()
        coordinate = artifact["url"]
        requirement = distribution + " @ " + coordinate
        archive_row = f'{{ url = {q(coordinate)}, hash = "sha256:{artifact["sha256"]}", size = {Path(artifact["original_path"]).stat().st_size} }}'
        archive_field = f'wheels = [{archive_row}]' if artifact["filename"].endswith(".whl") else f'sdist = {archive_row}'
        (directory / "pyproject.toml").write_text(
            '[project]\nname = "mantle-pure-root"\nversion = "1.0.0"\nrequires-python = "==3.12.*"\n'
            f'dependencies = [{q(requirement)}]\n'
            '[project.optional-dependencies]\nfeature = []\n[dependency-groups]\nproof = []\n[tool.uv]\npackage = false\n')
        (directory / "uv.lock").write_text(
            'version = 1\nrevision = 3\nrequires-python = "==3.12.*"\n\n'
            '[[package]]\nname = "mantle-pure-root"\nversion = "1.0.0"\nsource = { virtual = "." }\n'
            f'dependencies = [{{ name = {q(distribution)} }}]\n'
            '[package.optional-dependencies]\nfeature = []\n[package.dev-dependencies]\nproof = []\n'
            '[package.metadata]\n'
            f'requires-dist = [{{ name = {q(distribution)}, url = {q(coordinate)} }}]\n'
            'provides-extras = ["feature"]\n[package.metadata.requires-dev]\nproof = []\n\n'
            f'[[package]]\nname = {q(distribution)}\nversion = "1.0.0"\n'
            f'source = {{ url = {q(coordinate)} }}\n{archive_field}\n')
        return directory

    def config(self, project, mode="environment", native=False):
        return {"schema": 1, "mode": mode, "store_prefix": "/nix/store", "python": self.tools["python"],
                "uv": self.tools["uv"], "project": project,
                "target": {"python": self.tools["python_version"], "implementation": "cpython", "abi": "cp312",
                           "platform": "linux_x86_64", "wheel_tags": ["py3-none-any", "cp312-cp312-linux_x86_64"]},
                "extras": [], "groups": [], "members": [], "artifacts": self.admitted_artifacts,
                "packaging": "packaging-25.0-py3-none-any.whl",
                "build_requirements": ["setuptools==80.9.0", "wheel==0.45.1", "packaging==25.0"] if native else [],
                "dynamic_requirements": [], "build_packages": ["mantle-native-answer"] if native else [],
                "source_date_epoch": EPOCH, "native_inputs": self.tools["native_inputs"] if native else {"inputs": {}, "env": {}, "path": []},
                "runtime_inputs": self.nix_roots, "git_sources": {}}

    def recipe(self, config, label):
        path = self.directory / (label + ".json")
        path.write_text(json.dumps(config, sort_keys=True))
        admitted_config = self.fetch(path, label + "-config")
        native = config["native_inputs"]
        args = {"name": "python-proof", "adapter": self.adapter, "config": admitted_config,
                "python": root(config["python"]), "uv": root(config["uv"]), "project": config["project"],
                "artifacts": sorted({root(row["path"]) for row in config["artifacts"] if row["path"]}),
                "native_inputs": native, "runtime_inputs": config["runtime_inputs"]}
        # JSON is valid Nickel record syntax only after parsing through an explicit
        # JSON import; avoid custom text substitution of arbitrary values.
        params = self.directory / (label + "-params.json")
        params.write_text(json.dumps(args))
        return f'let python = import "python.ncl" in python (import {q(params)})\n'

    def negative(self, config, label, diagnostic):
        report, logs = self.build(self.recipe(config, label), label, True)
        require(diagnostic in logs, f"missing negative diagnostic {diagnostic}: {logs[-6000:]}")
        self.evidence[label] = {"failed_total": report["counts"]["failed_total"], "diagnostic": diagnostic}

    def native_variant(self, config, label, filename, old, new):
        tree = self.directory / (label + "-payload")
        shutil.copytree(self.native_payload, tree)
        path = tree / filename
        text = path.read_text()
        require(old in text, "native fixture mutation did not match")
        path.write_text(text.replace(old, new, 1))
        archive = self.make_native_sdist(tree, label + "-archive")
        admitted = self.fetch(archive, label + "-archive-admission")
        result = copy.deepcopy(config)
        row = next(row for row in result["artifacts"] if row["filename"] == archive.name)
        row.update(path=admitted, sha256=sha(archive))
        project = self.directory / (label + "-project")
        self.pure_project(project, {**row, "original_path": str(archive)}, "mantle-native-answer")
        result["project"] = self.source(project, label + "-project-admission")
        return result


    def run(self, cohort):
        self.state = self.directory / (cohort + "-state")
        self.store = self.directory / (cohort + "-store")
        self.store.mkdir()
        self.command([self.a.mantle, "--nix-compat", "--state-dir", self.state, "--store", self.store,
                      "store", "pull", "--from", self.cache, "--trusted-public-keys", self.a.trusted_public_key, "--all"])
        for declared in self.nix_roots:
            self.verify(Path(declared))
            self.cli(["source", "bundle", "import-store-path", "--logical", declared,
                      "--physical", self.store / Path(declared).name,
                      "--trusted-public-keys", self.a.trusted_public_key, "--pin"])
        raw_adapter = self.fetch(self.a.adapter, cohort + "-adapter-source")
        code = "import os,shutil,sys;shutil.copyfile(sys.argv[1],os.environ['out']);os.chmod(os.environ['out'],0o755)"
        text = 'let mantle = import "lib.ncl" in {'
        text += f'name="python-proof", builder={q(self.tools["python"])}, args=["-I","-S","-c",{q(code)},{q(raw_adapter)}], '
        text += f'inputs={json.dumps([raw_adapter, *self.nix_roots])}, outputs=["out"], sandbox=\'native, addressing_mode=\'input-addressed }} | mantle.Derivation'
        physical, _ = self.build(text, cohort + "-adapter-executable")
        self.adapter = self.logical(physical)
        self.cli(["source", "bundle", "import-store-path", "--logical", self.adapter,
                  "--physical", physical, "--trusted-public-keys", self.a.trusted_public_key, "--pin"])
        self.admitted_artifacts = []
        for index, artifact in enumerate(self.artifacts):
            admitted = self.fetch(artifact["path"], f"{cohort}-artifact-{index}")
            self.admitted_artifacts.append({"filename": artifact["filename"], "url": artifact["url"], "path": admitted, "sha256": artifact["sha256"]})
        pure_artifact = {**next(row for row in self.admitted_artifacts if row["filename"] == self.pure_wheel.name), "original_path": str(self.pure_wheel)}
        pure_tree = self.pure_project(self.directory / (cohort + "-pure-source"), pure_artifact)
        pure_source = self.source(pure_tree, cohort + "-pure-source-admission")
        config = self.config(pure_source)
        recipe = self.recipe(config, cohort + "-pure")
        pure_output, _ = self.build(recipe, cohort + "-pure")
        expected = "mantle-pure-answer=42"
        require(self.command([pure_output / "python", FIXTURES / "pure/run.py"]).strip() == expected, "pure environment did not run")
        warm_output, warm = self.build(recipe, cohort + "-pure-warm")
        require(warm_output == pure_output and warm["counts"]["built_total"] == 0 and warm["counts"]["cached_total"] >= 1,
                "trusted warm reuse not observed")
        extra = copy.deepcopy(config)
        extra["extras"] = ["feature"]
        extra_output, _ = self.build(self.recipe(extra, cohort + "-extra"), cohort + "-extra")
        pure_manifest = json.loads((pure_output / "manifest.json").read_text())
        require(json.loads((extra_output / "manifest.json").read_text())["plan_identity"] != pure_manifest["plan_identity"],
                "extras did not invalidate identity")
        grouped = copy.deepcopy(config)
        grouped["groups"] = ["proof"]
        group_output, _ = self.build(self.recipe(grouped, cohort + "-group"), cohort + "-group")
        require(json.loads((group_output / "manifest.json").read_text())["plan_identity"] != pure_manifest["plan_identity"],
                "groups did not invalidate identity")
        planned = copy.deepcopy(config)
        planned["mode"] = "plan"
        next(row for row in planned["artifacts"] if row["filename"] == self.pure_wheel.name)["path"] = ""
        plan_output, _ = self.build(self.recipe(planned, cohort + "-plan"), cohort + "-plan")
        plan_manifest = json.loads((plan_output / "manifest.json").read_text())
        require(not plan_manifest["offline_ready"] and any(not row["available"] for row in plan_manifest["acquisition"]),
                "missing source acquisition plan claimed readiness")
        native_tree = self.directory / (cohort + "-native-source")
        sdist_row = {**self.admitted_artifacts[-1], "original_path": str(self.native_sdist)}
        self.pure_project(native_tree, sdist_row, "mantle-native-answer")
        native_source = self.source(native_tree, cohort + "-native-source-admission")
        native = self.config(native_source, "wheel", True)
        native_output, _ = self.build(self.recipe(native, cohort + "-native"), cohort + "-native")
        native_wheels = list((native_output / "wheels").glob("mantle_native_answer-*.whl"))
        require(len(native_wheels) == 1, "native wheel missing")
        native_wheel = native_wheels[0]
        # A real uv installation followed by a real import of the compiled module.
        native_env_tree = self.directory / (cohort + "-native-env-source")
        native_row = {"filename": native_wheel.name, "url": "https://python-proof.invalid/" + native_wheel.name,
                      "path": self.logical(native_output) + "/wheels/" + native_wheel.name,
                      "sha256": sha(native_wheel), "original_path": str(native_wheel)}
        self.pure_project(native_env_tree, native_row, "mantle-native-answer")
        native_env_source = self.source(native_env_tree, cohort + "-native-env-source-admission")
        native_env = self.config(native_env_source)
        native_env["artifacts"] = [*self.admitted_artifacts[:-1], {key: value for key, value in native_row.items() if key != "original_path"}]
        env_output, _ = self.build(self.recipe(native_env, cohort + "-native-env"), cohort + "-native-env")
        require(self.command([env_output / "python", FIXTURES / "native/run.py"]).strip() == "mantle-native-answer=42",
                "compiled native extension did not run")
        self.evidence[cohort] = {"pure_output": str(pure_output), "native_output": str(native_output),
                                 "native_wheel_sha256": sha(native_wheel), "pure_plan_identity": pure_manifest["plan_identity"],
                                 "native_manifest": json.loads((native_output / "manifest.json").read_text()),
                                 "warm_counts": warm["counts"]}
        if cohort == "first":
            bad = copy.deepcopy(config)
            bad["target"]["abi"] = "cp311"
            self.negative(bad, "wrong-abi", "python-target-unsupported")
            bad = copy.deepcopy(config)
            bad["target"]["platform"] = "win_amd64"
            self.negative(bad, "wrong-platform", "python-target-unsupported")
            bad = copy.deepcopy(config)
            bad["target"]["python"] = "3.12.999"
            self.negative(bad, "wrong-interpreter", "python-interpreter-version-mismatch")
            bad = copy.deepcopy(config)
            bad["artifacts"][-1]["sha256"] = "0" * 64
            self.negative(bad, "wrong-artifact-hash", "python-artifact-hash-mismatch")
            bad = copy.deepcopy(native)
            bad["artifacts"] = [row for row in bad["artifacts"] if not row["filename"].startswith("setuptools-")]
            self.negative(bad, "missing-build-dependency", "python-build-dependency-missing")
            bad = self.native_variant(native, "dynamic", "setup.py", "setup(ext_modules=",
                                      'setup(setup_requires=["missing-build-package==1.0.0"], ext_modules=')
            self.negative(bad, "undeclared-dynamic-dependency", "python-backend-dynamic-requirements-mismatch")
            bad = self.native_variant(native, "backend-pin", "pyproject.toml", "setuptools==80.9.0", "setuptools==80.8.0")
            self.negative(bad, "changed-backend-pin", "python-backend-build-requirements-mismatch")
            stale = self.directory / "stale-source"
            shutil.copytree(pure_tree, stale)
            p = stale / "pyproject.toml"
            p.write_text(p.read_text().replace('dependencies = [', 'dependencies = ["not-locked==1.0.0", '))
            bad = copy.deepcopy(config)
            bad["project"] = self.source(stale, "stale-source-admission")
            self.negative(bad, "stale-lock", "python-tool-failed")
            missing_hash = self.directory / "missing-hash-source"
            shutil.copytree(pure_tree, missing_hash)
            p = missing_hash / "uv.lock"
            p.write_text(re.sub(r', hash = "sha256:[0-9a-f]{64}"', '', p.read_text()))
            bad = copy.deepcopy(config)
            bad["project"] = self.source(missing_hash, "missing-hash-source-admission")
            self.negative(bad, "missing-lock-hash", "python-lock-missing-hash")
            bad = self.native_variant(native, "download", "setup.py", "from setuptools",
                                      'import urllib.request\nurllib.request.urlopen("https://pypi.org/simple/", timeout=2)\nfrom setuptools')
            self.negative(bad, "backend-download-denied", "python-tool-failed")
            altered = self.native_variant(native, "changed-native", "answer.c", "FromLong(42)", "FromLong(43)")
            altered_output, _ = self.build(self.recipe(altered, "changed-native"), "changed-native")
            altered_manifest = json.loads((altered_output / "manifest.json").read_text())
            original_manifest = self.evidence[cohort]["native_manifest"]
            require(altered_manifest["plan_identity"] != original_manifest["plan_identity"], "source change reused identity")
            require(altered_manifest["wheels"][0]["sha256"] != original_manifest["wheels"][0]["sha256"], "changed C source reused wheel")
            lock_changed = self.directory / "changed-lock-project"
            shutil.copytree(pure_tree, lock_changed)
            path = lock_changed / "uv.lock"
            path.write_text(path.read_text() + "\n# independently changed lock bytes\n")
            lock_config = copy.deepcopy(config)
            lock_config["project"] = self.source(lock_changed, "changed-lock-admission")
            lock_output, _ = self.build(self.recipe(lock_config, "changed-lock"), "changed-lock")
            require(json.loads((lock_output / "manifest.json").read_text())["plan_identity"] != pure_manifest["plan_identity"],
                    "lock byte change reused identity")
        return native_wheel


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mantle", type=Path, required=True)
    parser.add_argument("--adapter", type=Path, required=True)
    parser.add_argument("--nix", type=Path, required=True)
    parser.add_argument("--inputs", type=Path, required=True)
    parser.add_argument("--signing-key", type=Path, required=True)
    parser.add_argument("--trusted-public-key", required=True)
    parser.add_argument("--run-dir", type=Path, required=True)
    arguments = parser.parse_args()
    require(arguments.run_dir.is_absolute(), "run directory must be absolute")
    proof = Proof(arguments)
    first = proof.run("first")
    second = proof.run("clean")
    require(first.read_bytes() == second.read_bytes(), "clean native wheels differ")
    a = proof.evidence["first"]["native_manifest"]["wheels"]
    b = proof.evidence["clean"]["native_manifest"]["wheels"]
    require(a == b, "clean wheel metadata/timestamps differ")
    proof.evidence["clean_comparison"] = {"wheel_bytes_equal": True, "metadata_timestamps_equal": True,
                                           "non_claim": "bounded fixture only; independent physical stores, same logical prefix/tool closure"}
    (proof.directory / "evidence.json").write_text(json.dumps(proof.evidence, indent=2, sort_keys=True) + "\n")
    print(proof.directory / "evidence.json")


if __name__ == "__main__":
    main()
