#!/usr/bin/env python3
"""Real Node/pnpm proof through Mantle; never downloads or invokes a host pnpm store.
Requires pre-admitted adapter/tool/runtime roots and explicit signing authority.
See docs/pnpm-workspaces.md. This is a driver, not an ordinary test-suite fixture.
"""
import argparse
import base64
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import tarfile


def encode(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


def write_json(path, value):
    path.write_text(encode(value) + "\n")


def archive(files):
    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w", format=tarfile.USTAR_FORMAT) as tar:
        for name, content in sorted(files.items()):
            data = content.encode()
            item = tarfile.TarInfo("package/" + name)
            item.size, item.mode, item.mtime = len(data), 0o644, 1
            tar.addfile(item, io.BytesIO(data))
    return gzip.compress(buf.getvalue(), mtime=0)


def fixture(root, helper):
    source, mirror = root / "source", root / "mirror"
    shutil.copytree(Path(__file__).resolve().parents[1] / "tests/fixtures/pnpm/workspace", source)
    mirror.mkdir()
    packages = []
    definitions = {
        "peer": ({"name": "@fixture/peer", "version": "1.0.0", "main": "index.js"}, "module.exports = 'peer';\n"),
        "consumer": ({"name": "@fixture/consumer", "version": "1.0.0", "main": "index.js", "peerDependencies": {"@fixture/peer": "1.0.0"}}, "module.exports = require('@fixture/peer') + ':unpatched';\n"),
        "darwin-only": ({"name": "@fixture/darwin-only", "version": "1.0.0", "os": ["darwin"]}, "module.exports = 'darwin';\n"),
        "git-archive": ({"name": "@fixture/git-archive", "version": "1.0.0", "main": "index.js"}, "module.exports = 'git-archive';\n"),
    }
    inventory = {}
    for name, (manifest, code) in definitions.items():
        data = archive({"package.json": encode(manifest), "index.js": code})
        path = mirror / (name + ".tgz")
        path.write_bytes(data)
        integrity = "sha512-" + base64.b64encode(hashlib.sha512(data).digest()).decode()
        key = manifest["name"] + "@1.0.0"
        packages.append({"lock_key": key, "archive": path.name, "integrity": integrity,
                         "blake3": helper("blob-digest", path), "mirror": "fixture-pnpm-mirror", "git_revision": None})
        inventory[key] = {"resolution": {"integrity": integrity}}
        if name == "git-archive":
            revision = "0123456789abcdef0123456789abcdef01234567"
            packages[-1]["git_revision"] = revision
            inventory[key]["resolution"]["tarball"] = "https://fixture.invalid/git/" + revision + "/package.tgz"
        if "peerDependencies" in manifest:
            inventory[key]["peerDependencies"] = manifest["peerDependencies"]
        if "os" in manifest:
            inventory[key]["os"] = manifest["os"]
    patch = (source / "patches/consumer.patch").read_bytes().replace(b"\r\n", b"\n")
    # pnpm 9.15.9 packages/crypto.base32-hash: MD5/Base32 is interop only.
    patch_hash = base64.b32encode(hashlib.md5(patch).digest()).decode().rstrip("=").lower()
    patched = f"1.0.0(patch_hash={patch_hash})(@fixture/peer@1.0.0)"
    lock = {"lockfileVersion": "9.0", "settings": {"autoInstallPeers": True, "excludeLinksFromLockfile": False},
            "patchedDependencies": {"@fixture/consumer@1.0.0": {"hash": patch_hash, "path": "patches/consumer.patch"}},
            "importers": {".": {}, "packages/lib": {"dependencies": {
                "@fixture/consumer": {"specifier": "1.0.0", "version": patched},
                "@fixture/git-archive": {"specifier": "1.0.0", "version": "1.0.0"},
                "@fixture/peer": {"specifier": "1.0.0", "version": "1.0.0"}}},
                "packages/app": {"dependencies": {"@fixture/lib": {"specifier": "workspace:*", "version": "link:../lib"}},
                    "optionalDependencies": {"@fixture/darwin-only": {"specifier": "1.0.0", "version": "1.0.0"}}}},
            "packages": inventory, "snapshots": {
                "@fixture/peer@1.0.0": {},
                "@fixture/git-archive@1.0.0": {},
                "@fixture/consumer@" + patched: {"dependencies": {"@fixture/peer": "1.0.0"}},
                "@fixture/darwin-only@1.0.0": {"optional": True}}}
    write_json(source / "pnpm-lock.yaml", lock)  # JSON is a strict YAML subset.
    return source, mirror, packages


class Driver:
    def __init__(self, args):
        self.args = args
        self.root = args.work_dir.resolve()
        self.root.mkdir(exist_ok=False)
        self.roles = json.loads(args.inputs_json.read_text())
        self.base = [str(args.mantle.resolve()), "--store", str(args.store.resolve()),
                     "--state-dir", str(args.state_dir.resolve()), "--store-prefix", args.store_prefix]
        self.evidence = []

    def helper(self, command, value):
        value = str(value)
        result = subprocess.run([str(self.args.adapter.resolve()), command, value], check=True, capture_output=True, text=True)
        return result.stdout.strip()

    def command(self, label, args, success=True):
        result = subprocess.run(args, capture_output=True, text=True, env={**os.environ, "HOME": str(self.root / "empty-home")})
        (self.root / (label + ".stdout")).write_text(result.stdout)
        (self.root / (label + ".stderr")).write_text(result.stderr)
        self.evidence.append({"label": label, "argv": args, "exit": result.returncode})
        if success and result.returncode:
            raise RuntimeError(f"{label} failed; see {self.root}/{label}.stderr")
        if not success and result.returncode == 0:
            raise RuntimeError(f"negative {label} unexpectedly succeeded")
        return result

    def build(self, label, text, success=True):
        path = self.root / (label + ".ncl")
        path.write_text(text)
        result = self.command(label, self.base + ["--json", "build", str(path), "--import-path", str(self.args.lib.resolve()),
            "--no-substitute", "--strict-hermetic", "--signing-key", str(self.args.signing_key.resolve()),
            "--trusted-public-keys", self.args.trusted_public_key], success)
        try:
            report = json.loads(result.stdout)
        except json.JSONDecodeError:
            if success:
                raise
            report = {}
        return report, result

    def admit(self, label, root):
        script = ["set -eu", "umask 022", "/bin/busybox mkdir -p \"$out\""]
        for path in sorted(root.rglob("*")):
            relative = str(path.relative_to(root))
            target = '"$out"/' + shlex.quote(relative)
            if path.is_symlink():
                script.append("/bin/busybox ln -s " + shlex.quote(os.readlink(path)) + " " + target)
            elif path.is_dir():
                script.append("/bin/busybox mkdir -p " + target)
            else:
                payload = base64.b64encode(path.read_bytes()).decode()
                script.append("printf '%s' " + shlex.quote(payload) + " | /bin/busybox base64 -d > " + target)
        drv = {"name": label, "builder": "/bin/sh", "args": ["-c", "\n".join(script)], "addressing_mode": "input-addressed"}
        report, _ = self.build(label, "std.deserialize 'Json " + json.dumps(encode(drv)))
        output = report["outcomes"][0]["outputs"][0]
        logical = self.args.store_prefix.rstrip("/") + "/" + Path(output["path"]).name
        return {"input": logical, "path": logical}

    def recipe(self, name, plan, source, mirror):
        plan_files = self.root / (name + "-plan-files")
        plan_files.mkdir()
        write_json(plan_files / "request.json", plan)
        plan_role = self.admit(name + "-plan", plan_files)
        plan_role["path"] += "/request.json"
        plan_role["blake3"] = self.helper("blob-digest", plan_files / "request.json")
        value = {"name": name, "plan": plan_role, "source": source, "mirror": mirror,
                 "adapter": {k: self.roles["adapter"][k] for k in ("input", "path")},
                 "tools": {k: self.roles["tools"][k] for k in ("input", "path")},
                 "runtime": [{k: r[k] for k in ("input", "path")} for r in self.roles["runtime"]]}
        return "let pnpm = import \"pnpm.ncl\" in pnpm.mkWorkspace (std.deserialize 'Json " + json.dumps(encode(value)) + ")"

    def scenario(self, name, original, mutate=None, success=True):
        source, mirror, packages = original
        path = self.root / name
        path.mkdir()
        shutil.copytree(source, path / "source")
        shutil.copytree(mirror, path / "mirror")
        source, mirror = path / "source", path / "mirror"
        if mutate:
            mutate(source, mirror)
        plan = {"schema": "mantle-pnpm-workspace-v1", "node_version": "22.14.0", "pnpm_version": "9.15.9", "platform": "linux-x64",
                "source_blake3": self.helper("digest", source), "tools_blake3": self.roles["tools"]["blake3"],
                "packages": packages, "runtime": [{k: r[k] for k in ("path", "blake3")} for r in self.roles["runtime"]],
                "targets": [{"workspace": "packages/lib", "scripts": ["build"], "outputs": ["dist"], "package_output": "package"},
                            {"workspace": "packages/app", "scripts": ["build"], "outputs": ["dist"], "package_output": None}]}
        src_role, mirror_role = self.admit(name + "-source", source), self.admit(name + "-mirror", mirror)
        recipe = self.recipe(name, plan, src_role, mirror_role)
        report, result = self.build(name, recipe, success)
        return plan, src_role, mirror_role, recipe, report, result

    def run(self):
        (self.root / "empty-home").mkdir()
        (self.root / "fixture").mkdir()
        original = fixture(self.root / "fixture", self.helper)
        plan, source, mirror, recipe, cold, _ = self.scenario("pnpm-cold", original)
        cold_out = Path(cold["outcomes"][0]["outputs"][0]["path"])
        assert cold["outcomes"][0]["cached"] is False
        assert (cold_out / "packages/app/dist/index.html").read_text() == "<h1>peer:patched</h1>\n"
        tarballs = list((cold_out / "packages/lib/package").glob("*.tgz"))
        assert len(tarballs) == 1
        with tarfile.open(tarballs[0], "r:gz") as package:
            assert package.extractfile("package/dist/index.js").read() == b'module.exports = "peer:patched";\n'
        warm, _ = self.build("pnpm-warm", recipe)
        assert warm["outcomes"][0]["cached"] is True
        assert warm["outcomes"][0]["outputs"][0]["path"] == str(cold_out)
        clean, _ = self.build("pnpm-clean", self.recipe("pnpm-clean", plan, source, mirror))
        assert clean["outcomes"][0]["cached"] is False
        assert self.helper("digest", cold_out) == self.helper("digest", Path(clean["outcomes"][0]["outputs"][0]["path"]))
        changed = self.scenario("pnpm-changed", original, lambda s, m: (s / "packages/app/build.cjs").write_text(
            (s / "packages/app/build.cjs").read_text().replace("<h1>", "<h2>").replace("</h1>", "</h2>")))
        changed_out = Path(changed[4]["outcomes"][0]["outputs"][0]["path"])
        assert changed[4]["outcomes"][0]["cached"] is False and changed_out != cold_out
        assert json.loads((changed_out / "provenance.json").read_text())["identity_blake3"] != json.loads((cold_out / "provenance.json").read_text())["identity_blake3"]
        assert (changed_out / "packages/app/dist/index.html").read_text() == "<h2>peer:patched</h2>\n"
        def stale(s, m):
            p = s / "packages/lib/package.json"
            value = json.loads(p.read_text()); value["dependencies"]["@fixture/peer"] = "2.0.0"; write_json(p, value)
        def prohibited(s, m):
            p = s / "packages/lib/package.json"
            value = json.loads(p.read_text()); value["scripts"]["preinstall"] = "node build.cjs"; write_json(p, value)
        negatives = {
            "stale-lock": (stale, "OUTDATED_LOCKFILE"),
            "integrity-drift": (lambda s, m: (m / "peer.tgz").write_bytes(b"corrupted"), "archive integrity drift"),
            "escaping-symlink": (lambda s, m: (s / "escape").symlink_to("/etc/passwd"), "symlinks prohibited"),
            "missing-package": (lambda s, m: (m / "peer.tgz").unlink(), "No such file"),
            "prohibited-script": (prohibited, "prohibited lifecycle script"),
            "undeclared-tool": (lambda s, m: (s / "packages/app/build.cjs").write_text(
                "require('node:child_process').execFileSync('mantle-undeclared-native-tool', []);\n"), "ENOENT"),
            "network": (lambda s, m: (s / "packages/app/build.cjs").write_text(
                "const net=require('node:net'); const s=net.connect(80,'192.0.2.1');\n"
                "s.on('error',e=>{console.error('NETWORK_DENIED:'+e.code);process.exit(44)});\n"
                "s.on('connect',()=>{console.error('NETWORK_UNEXPECTEDLY_ALLOWED');process.exit(45)});\n"
                "setTimeout(()=>{console.error('NETWORK_UNPROVEN_TIMEOUT');process.exit(46)},2000);\n"), "NETWORK_DENIED:ENETUNREACH"),
        }
        for name, (mutate, expected) in negatives.items():
            scenario = self.scenario("pnpm-negative-" + name, original, mutate, False)
            report, result = scenario[4], scenario[5]
            logs = result.stdout + result.stderr
            for failure in report.get("failed", []):
                for field in ("saved_log_path", "log_file"):
                    path = failure.get(field)
                    if path and Path(path).is_file():
                        logs += Path(path).read_text()
            if expected not in logs:
                # Persisted failure envelopes may nest log references. Read only this
                # derivation's label-matching log, never arbitrary state payloads.
                for path in (self.args.state_dir / "logs").glob("*pnpm-negative-" + name + ".drv.log"):
                    logs += path.read_text()
            assert expected in logs, f"{name}: failure lacked expected diagnostic {expected!r}"
        verified = self.command("pnpm-signatures", self.base + ["store", "verify", "--trusted-public-keys", self.args.trusted_public_key])
        row = next(line for line in verified.stdout.splitlines() if cold_out.name in line)
        assert row.startswith("OK ") and re.search(r"trusted_signatures=[1-9][0-9]*/", row), row
        write_json(self.root / "evidence.json", {"schema": "mantle-pnpm-real-proof-v1", "commands": self.evidence,
            "tools": self.roles, "cold_output": str(cold_out), "changed_output": str(changed_out),
            "observed": ["real offline frozen two-workspace build", "patch and peer", "platform optional skip", "workspace link", "pnpm pack",
                         "signed warm reuse", "clean output equality", "changed source invalidation", *negatives.keys()],
            "non_claims": ["synthetic local package corpus, not the public npm registry", "not universal pnpm compatibility", "no external network acquisition exercised"]})
        print(self.root / "evidence.json")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for option in ("mantle", "adapter", "inputs-json", "store", "state-dir", "signing-key", "work-dir", "lib"):
        parser.add_argument("--" + option, required=True, type=Path)
    parser.add_argument("--trusted-public-key", required=True)
    parser.add_argument("--store-prefix", default="/nix/store")
    Driver(parser.parse_args()).run()


if __name__ == "__main__":
    main()
