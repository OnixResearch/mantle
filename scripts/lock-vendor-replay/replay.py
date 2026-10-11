#!/usr/bin/env python3
"""Detached, fresh-state native lock-vendor replay. Run through run.sh inside nix develop.

No old binary, plan receipt, cache state or assembled output is accepted. The
historical registry cache and Git trees are *inputs* checked against this lock.
All claims are written under the caller's newly created --scratch directory.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tomllib

REPO = Path(__file__).resolve().parents[2]
FIXTURE = REPO / "fixtures/lock-vendor-two-crate"
DEFAULT_ROOT = Path.home() / "mantle-cairn-finish"
HEX = re.compile(r"[0-9a-f]{64}\Z")


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def run(args, *, stdout=None, cwd=None, env=None, expected=0):
    result = subprocess.run([str(x) for x in args], cwd=cwd, env=env, text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if stdout is not None:
        Path(stdout).write_text(result.stdout)
        Path(str(stdout) + ".stderr").write_text(result.stderr)
    require((result.returncode == 0) if expected == 0 else (result.returncode != 0),
            f"command exit {result.returncode}, expected {expected}: {args}\n{result.stderr[-3000:]}")
    return result.stdout.strip(), result.returncode


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def nar(path):
    digest, _ = run(["nix", "hash", "path", "--algo", "sha256", "--format", "base16", path])
    require(HEX.fullmatch(digest), f"invalid NAR hash: {digest}")
    return digest


def root(binary):
    binary = Path(binary).resolve()
    require(binary.parts[:3] == ("/", "nix", "store") and len(binary.parts) >= 4,
            f"not pinned Nix binary: {binary}")
    return Path(*binary.parts[:4])


def runtime(*binaries):
    paths = set()
    for binary in binaries:
        paths.add(str(root(binary)))
        with Path(binary).open("rb") as file:
            prefix = file.read(2)
            if prefix == b"#!":
                interpreter = file.readline(1024).decode().strip().split()[0]
                paths.add(str(root(interpreter)))
                continue
        observed = subprocess.run(["ldd", str(binary)], capture_output=True, text=True)
        if observed.returncode:
            require("not a dynamic executable" in observed.stdout + observed.stderr
                    or "statically linked" in observed.stdout + observed.stderr,
                    f"cannot inspect pinned runtime dependencies: {binary}: {observed.stderr}")
            continue
        for item in re.findall(r"/nix/store/[^/\s()]+", observed.stdout):
            paths.add(str(root(item)))
    return sorted(paths)


def q(text):
    return json.dumps(str(text))


def cli(a, state, store, args, report, *, negative=False):
    cmd = [a.cli, "--json", "--store", store, "--state-dir", state,
           "--nix-compat", *args]
    stdout, code = run(cmd, stdout=report, cwd=REPO,
                       env={**os.environ, "TMPDIR": str(a.scratch)},
                       expected=1 if negative else 0)
    if negative:
        try:
            return json.loads(stdout), code
        except json.JSONDecodeError:
            return {"stderr": Path(str(report) + ".stderr").read_text()}, code
    return json.loads(stdout), code


def build(a, state, store, file, label, *, offline=None, negative=False):
    args = ["build", "--no-substitute", "--signing-key", a.signing_key,
            "--trusted-public-keys", a.trusted_key, "-I", REPO / "lib"]
    if offline:
        args.append("--strict-hermetic")
        args += ["--offline-source-manifest", offline[0],
                 "--offline-source-manifest-blake3", offline[1]]
    args.append(file)
    return cli(a, state, store, args, a.scratch / f"{label}.json", negative=negative)


def verify(a, state, store, logical, label):
    require(str(logical).startswith("/nix/store/"), "expected logical /nix/store output")
    # `store verify` filters by PathInfo basename and emits text even under
    # --json; a full /nix/store selector silently matches zero records.
    basename = Path(logical).name
    report = a.scratch / f"verify-{label}.txt"
    stdout, _ = run([a.cli, "--json", "--store", store, "--state-dir", state,
                     "--nix-compat", "store", "verify", basename,
                     "--trusted-public-keys", a.trusted_key],
                    stdout=report, cwd=REPO,
                    env={**os.environ, "TMPDIR": str(a.scratch)})
    stderr = Path(str(report) + ".stderr").read_text()
    require(re.fullmatch(rf"OK {re.escape(basename)}  trusted_signatures=[1-9][0-9]*/[1-9][0-9]*", stdout)
            and "1 checked, 0 mismatches" in stderr,
            f"no matched signed PathInfo verification for {logical}: {stdout!r} {stderr!r}")
    return stdout


def create_key(a):
    directory = a.scratch / "keys"
    directory.mkdir(mode=0o700)
    secret = subprocess.run(["nix", "key", "generate-secret", "--key-name",
                             "mantle-rust-efficiency-lock-vendor-20261011"],
                            text=True, capture_output=True, check=True).stdout.strip()
    require(secret.startswith("mantle-rust-efficiency-lock-vendor-20261011:"),
            "wrong generated signing key name")
    key = directory / "signing-key"
    key.write_text(secret + "\n")
    key.chmod(0o600)
    public = subprocess.run(["nix", "key", "convert-secret-to-public"],
                            input=secret + "\n", text=True, capture_output=True, check=True).stdout.strip()
    require(public.count(":") == 1 and public.startswith("mantle-rust-efficiency-lock-vendor-20261011:"),
            "invalid derived public key")
    a.signing_key, a.trusted_key = key, public
    return {"public_name": public.split(":")[0], "public_sha256": hashlib.sha256(public.encode()).hexdigest()}


def import_signed_runtime(a, state, store, python, rust, linker, git, base_runtime):
    """No ambient /nix/store fallback: import the exact signed tool closures."""
    patch = root(shutil.which("patch"))
    roots = sorted(set([str(python), str(rust), str(linker), str(git),
                        str(patch), *base_runtime,
                        *runtime(git / "bin/git", patch / "bin/patch")]))
    cache = a.scratch / "input-cache"
    run(["nix", "copy", "--offline", "--to",
         f"file://{cache}?compression=none&secret-key={a.signing_key}", *roots],
        stdout=a.scratch / "signed-inputs-copy.txt", cwd=REPO)
    pull = a.scratch / "signed-inputs-pull.txt"
    run([a.cli, "--nix-compat", "--store", store, "--state-dir", state,
         "store", "pull", "--from", cache,
         "--trusted-public-keys", a.trusted_key, "--all"],
        stdout=pull, cwd=REPO, env={**os.environ, "TMPDIR": str(a.scratch)})
    text = Path(str(pull) + ".stderr").read_text()
    counts = re.search(r"imported=([0-9]+) skipped_present=([0-9]+) "
                       r"skipped_untrusted=([0-9]+) skipped_hash_mismatch=([0-9]+) "
                       r"skipped_missing_nar=([0-9]+) skipped_parse_error=([0-9]+)", text)
    require(counts is not None and int(counts[1]) >= len(roots)
            and all(int(counts[i]) == 0 for i in range(2, 7)),
            f"runtime closure did not enter the isolated signed store: {text[-2000:]}")
    return {"declared_roots": roots, "imported_paths": int(counts[1]),
            "input_cache": str(cache), "narinfo_count": len(list(cache.glob("*.narinfo")))}

def import_signed_roots(a, state, store, roots, label):
    """Admit exact Nix inputs through signed PathInfo before a Mantle consumer runs."""
    roots = sorted({str(path) for path in roots})
    require(roots and all(path.startswith("/nix/store/") and
                          (Path(path).is_dir() or Path(path).is_file())
                          for path in roots), f"missing reviewed {label} roots")
    cache = a.scratch / "input-cache"
    run(["nix", "copy", "--offline", "--to",
         f"file://{cache}?compression=none&secret-key={a.signing_key}", *roots],
        stdout=a.scratch / f"signed-{label}-inputs-copy.txt", cwd=REPO)
    pull = a.scratch / f"signed-{label}-inputs-pull.txt"
    run([a.cli, "--nix-compat", "--store", store, "--state-dir", state,
         "store", "pull", "--from", cache,
         "--trusted-public-keys", a.trusted_key, "--all"],
        stdout=pull, cwd=REPO, env={**os.environ, "TMPDIR": str(a.scratch)})
    text = Path(str(pull) + ".stderr").read_text()
    counts = re.search(r"imported=([0-9]+) skipped_present=([0-9]+) "
                       r"skipped_untrusted=([0-9]+) skipped_hash_mismatch=([0-9]+) "
                       r"skipped_missing_nar=([0-9]+) skipped_parse_error=([0-9]+)", text)
    require(counts is not None and int(counts[1]) + int(counts[2]) >= len(roots)
            and all(int(counts[i]) == 0 for i in range(3, 7)),
            f"{label} roots did not enter the isolated signed store: {text[-2000:]}")
    return {"admitted_roots": roots, "imported_paths": int(counts[1]),
            "already_signed_paths": int(counts[2]),
            "narinfo_count": len(list(cache.glob("*.narinfo")))}


def exact_output(report, name, output_name="out"):
    matches = [o["path"] for row in report["outcomes"] for o in row["outputs"]
               if row["label"] == name and o["name"] == output_name]
    require(len(matches) == 1, f"missing/ambiguous signed output {name}/{output_name}: {matches}")
    require(Path(matches[0]).exists(), f"unexported signed output: {matches[0]}")
    return matches[0]


def accepted(report, expected_fetches):
    plans = [row for row in report["native_dynamic_plans"] if row["mode"] == "native"]
    require(len(plans) == 1, f"expected exactly one native plan, got {len(plans)}")
    row = plans[0]
    ids = row["accepted_unit_ids"]
    fetches = [value for value in ids if value.startswith("fetch.")]
    require(row["scheduler_action"] == "registered-roots" and row["rejection_reason"] is None,
            f"native plan not admitted: {row}")
    require(len(fetches) == expected_fetches and len(set(ids)) == len(ids)
            and "vendor.complete" in ids, "fetch unit identity cardinality disagrees with Cargo.lock")
    digest = row["canonical_plan_digest"]
    require(isinstance(digest, str) and HEX.fullmatch(digest), "missing accepted canonical plan digest")
    bindings = [b for b in report["plan_output_bindings"] if b["root"] == "vendor.complete"]
    require(len(bindings) == 1 and bindings[0]["status"] == "bound"
            and bindings[0]["plan_digest"] == digest, "consumer did not bind accepted plan")
    parent = bindings[0]["producer_drv_path"]
    require(parent == row["producer_key"] and parent.startswith("/nix/store/")
            and parent.endswith(".drv"), "producer identity not from accepted report")
    vendor = bindings[0]["output_path"]
    require(isinstance(vendor, str) and vendor.startswith("/nix/store/"), "missing resolved vendor output")
    return {"parent_drv_path": parent, "canonical_plan_digest": digest,
            "vendor_path": vendor, "fetch_unit_ids": fetches,
            "plan_artifact_path": row["plan_artifact_path"]}


def unpacked_nar(archive, scratch):
    tree = scratch / "source-tree"
    tree.mkdir(parents=True)
    with tarfile.open(archive, "r:gz") as source:
        seen = set()
        for entry in source:
            require(entry.name and not Path(entry.name).is_absolute()
                    and ".." not in Path(entry.name).parts
                    and entry.name not in seen and not entry.islnk(),
                    f"unsafe source archive member: {entry.name}")
            seen.add(entry.name)
            if entry.issym():
                parts = Path(entry.name).parts
                checkout_root = Path(*parts[:4])
                target = Path(os.path.normpath(str(Path(entry.name).parent / entry.linkname)))
                require(len(parts) >= 5 and parts[:2] == ("git-cache", "checkouts")
                        and entry.linkname and not Path(entry.linkname).is_absolute()
                        and target.is_relative_to(checkout_root),
                        f"Git checkout symlink escapes pinned source: {entry.name}")
            require(entry.isfile() or entry.isdir() or entry.issym(),
                    "source bundle contains special entry")
        source.extractall(tree, filter="data")
    return nar(tree)


def source_archive(a, profile, workspace=False):
    name = f"{profile}-source.tar.gz"
    archive = a.scratch / name
    args = [sys.executable, REPO / "scripts/vendor-source-bundle.py", "--profile", profile,
            "--index", FIXTURE / "registry-index.tar.gz" if not workspace else a.index,
            "--to", archive]
    if workspace:
        args += ["--git-cache", a.git_cache, "--git-sidecars", a.git_sidecars,
                 "--reviewed-table", a.table]
    run(args, stdout=a.scratch / f"{profile}-archive-report.json", cwd=REPO)
    return archive, unpacked_nar(archive, a.scratch / profile)
def workspace_rows(a):
    import importlib.util
    spec = importlib.util.spec_from_file_location("vendor_dynamic", REPO / "scripts/vendor-dynamic-assemble.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    rows = module.rows(REPO / "Cargo.lock", a.table)
    require(900 <= len(rows) <= 4095, f"unexpected full-lock artifact count: {len(rows)}")
    return rows


def prepare_git_sidecars(a, rows):
    """Old Git/Nix trees are bounded immutable inputs; freshly signed outputs follow."""
    hints = json.loads(a.git_trees.read_text())
    packages = tomllib.loads((REPO / "Cargo.lock").read_text())["package"]
    sources = {(p["name"], p["version"]): p["source"] for p in packages
               if p.get("source", "").startswith("git+")}
    by_source = {}
    for row in rows:
        if row[3] == "recursive":
            source = sources[row[:2]]
            require(by_source.setdefault(source, row)[2:6] == row[2:6],
                    f"Git source conflict {source}")
    sidecars = {}
    for source, row in sorted(by_source.items()):
        original = source[4:].split("?", 1)[0].split("#", 1)[0]
        hint = hints.get(original)
        require(isinstance(hint, list) and len(hint) == 2 and hint[0] == row[2],
                f"no reviewed Nix Git tree input: {source}")
        tree = Path(hint[1])
        require(tree.is_dir() and str(tree).startswith("/nix/store/") and nar(tree) == row[2],
                f"Git NAR input mismatch: {source}")
        matches = []
        for db in sorted((a.git_cache / "db").iterdir()):
            checkouts = a.git_cache / "checkouts" / db.name
            checkout = checkouts / row[5][:7]
            if not checkout.is_dir() or not (checkout / ".cargo-ok").is_file():
                continue
            value, _ = run(["git", "-C", checkout, "rev-parse", "HEAD"])
            url, _ = run(["git", "-C", checkout, "remote", "get-url", "origin"])
            if value == row[5] and url == db.resolve().as_uri():
                matches.append((db, checkout))
        require(len(matches) == 1, f"ambiguous or missing pinned Git checkout: {source}")
        db, checkout = matches[0]
        private_mirrors = {
            "ssh://git@github.com/OnixResearch/onix-artifact.git":
                DEFAULT_ROOT / "inputs/onix-artifact.git",
            "https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git":
                DEFAULT_ROOT / "inputs/durable-file-publication.git",
        }
        if original in private_mirrors:
            mirror = private_mirrors[original]
            require(mirror.is_dir(), f"missing declared private Git mirror {mirror}")
            mirror_tree, _ = run(["git", "--git-dir", mirror, "rev-parse", row[5] + "^{tree}"])
            checkout_tree, _ = run(["git", "-C", checkout, "rev-parse", row[5] + "^{tree}"])
            require(mirror_tree == checkout_tree,
                    f"private mirror tree differs from locked Git revision: {original}")
        sidecars[source] = {"git_db": str(db.relative_to(a.git_cache)),
                            "checkout": str(checkout.relative_to(a.git_cache)),
                            "git_tree": str(tree)}
    a.git_sidecars = a.scratch / "git-sidecars.json"
    a.git_sidecars.write_text(json.dumps({"git": sidecars}, sort_keys=True) + "\n")
    return {source: (by_source[source][2],
                     Path(hints[source[4:].split("?", 1)[0].split("#", 1)[0]][1]))
            for source in by_source}


def publisher_batch(a, state, store, specs, index, python):
    """Each published output is signed by the current CLI, not a Nix-cache receipt."""
    named = {}
    parts = ['let mantle = import "lib.ncl" in {']
    for offset, (key, spec) in enumerate(specs):
        label = f"lv-cache-{index:02d}-{offset:03d}"
        named[f"r{offset}"] = key
        if spec[0] == "flat":
            parts.append(f'  r{offset} = mantle.fetchurl {{ name = "{label}", '
                         f'url = {q(spec[1])}, hash = {q(spec[2])} }},')
        else:
            old = spec[1]
            code = "import os,shutil,sys; shutil.copytree(sys.argv[1],os.environ['out'],symlinks=True)"
            parts.append(f'  r{offset} = {{ name = "{label}", builder = {q(str(python / "bin/python3"))}, '
                         f'args = [{q("-c")}, {q(code)}, {q(old)}], outputs = ["out"], '
                         f'inputs = [{q(old)}, {q(python)}], addressing_mode = '
                         "'input-addressed, sandbox = 'native } | mantle.Derivation,")
    parts.append("}")
    ncl = a.scratch / f"publisher-{index:02d}.ncl"
    ncl.write_text("\n".join(parts) + "\n")
    report, _ = build(a, state, store, ncl, f"publisher-{index:02d}")
    outputs = {}
    for label, key in named.items():
        physical = Path(exact_output(report, label))
        require(physical.parent == store, "publisher output outside isolated store")
        outputs[key] = "/nix/store/" + physical.name
    return outputs


def registry_source(cache, row):
    """A missing cache file is acquired by the signed fixed-output fetcher."""
    name = f"{row[0]}-{row[1]}.crate"
    source = cache / name
    if source.exists() or source.is_symlink():
        require(source.is_file() and not source.is_symlink() and sha(source) == row[2],
                f"wrong locked archive: {row[0]}@{row[1]}")
        return source.as_uri()
    return f"https://static.crates.io/crates/{row[0]}/{name}"


def signed_catalog(a, state, store, rows, archive, archive_nar, git_trees, python):
    cache = a.registry_cache
    require(cache.is_dir(), f"missing immutable registry cache: {cache}")
    publisher, aliases = [], {}
    for row in rows:
        if row[3] == "flat":
            source_url = registry_source(cache, row)
            key = row[:2]
            publisher.append((key, ("flat", source_url, row[2])))
            aliases[row[:2]] = key
        else:
            match = next(((s, tree) for s, (digest, tree) in git_trees.items()
                          if s.endswith("#" + row[5]) and digest == row[2]), None)
            require(match is not None, f"no reviewed Git input: {row[0]}@{row[1]}")
            source_name, source = match
            key = ("git", source_name)
            if key not in aliases.values():
                publisher.append((key, ("recursive", source, row[2])))
            aliases[row[:2]] = key
    outputs = {}
    for index, offset in enumerate(range(0, len(publisher), 48)):
        outputs.update(publisher_batch(a, state, store, publisher[offset:offset + 48], index, python))
    require(len(aliases) == len(rows) and len(outputs) == len(publisher),
            "publisher output inventory incomplete")
    for key, spec in publisher:
        path = store / Path(outputs[key]).name
        require((sha(path) if spec[0] == "flat" else nar(path)) == spec[2],
                f"new signed publisher output hash mismatch: {key}")
    source_root = a.scratch / "workspace-source.ncl"
    source_root.write_text('let mantle = import "lib.ncl" in mantle.fetchTarball '
                           f'{{ name = "lv-full-source-bundle", url = {q(archive.as_uri())}, '
                           f'hash = {q(archive_nar)} }}\n')
    source_report, _ = build(a, state, store, source_root, "publisher-source")
    source_path = exact_output(source_report, "lv-full-source-bundle")
    source_logical = "/nix/store/" + Path(source_path).name
    verify(a, state, store, source_logical, "workspace-source")
    require(nar(source_path) == archive_nar, "newly signed source bundle NAR mismatch")
    catalog = {"format": "mantle-lock-vendor-offline-cache-v1",
               "provenance": "local-integrity-observation-v1", "store_prefix": "/nix/store",
               "source_bundle_store_path": source_logical, "reviewed_table": a.table.read_text(),
               "artifacts": [{"name": row[0], "version": row[1], "sha256": row[2],
                              "hash_mode": row[3], "fetch_url": row[4],
                              "git_rev": None if row[5] == "-" else row[5],
                              "store_path": outputs[aliases[row[:2]]]} for row in rows]}
    seed = a.scratch / "catalog-seed.json"
    seed.write_text(json.dumps(catalog, sort_keys=True, separators=(",", ":")) + "\n")
    require(seed.stat().st_size <= 2 * 1024 * 1024, "catalog exceeds signed bounded-read limit")
    imported, _ = run(["nix", "store", "add-file", seed])
    require(imported.startswith("/nix/store/") and sha(imported) == sha(seed),
            "catalog seed did not enter immutable Nix store")
    import_signed_roots(a, state, store, [imported], "catalog-seed")
    verify(a, state, store, imported, "catalog-seed")
    root_file = a.scratch / "catalog-publisher.ncl"
    copy_code = "import shutil,os,sys; shutil.copyfile(sys.argv[1],os.environ['out'])"
    root_file.write_text('let mantle = import "lib.ncl" in '
                         '{ name = "lv-catalog-signed", '
                         f'builder = {q(str(python / "bin/python3"))}, '
                         f'args = ["-c", {q(copy_code)}, {q(imported)}], '
                         f'inputs = [{q(imported)}, {q(python)}], outputs = ["out"], '
                         "addressing_mode = 'input-addressed, sandbox = 'native } | mantle.Derivation\n")
    catalog_report, _ = build(a, state, store, root_file, "publisher-catalog")
    catalog_path = exact_output(catalog_report, "lv-catalog-signed")
    verify(a, state, store, "/nix/store/" + Path(catalog_path).name, "catalog")
    require(sha(catalog_path) == sha(seed), "signed catalog changed during publication")
    digest, _ = run([a.b3sum, "--no-names", catalog_path])
    catalog_blake3 = digest.split()[0]
    require(HEX.fullmatch(catalog_blake3), "invalid signed catalog BLAKE3")
    return (catalog_path, catalog_blake3), {"artifacts": len(rows),
            "publisher_outputs": len(outputs), "catalog": catalog_path,
            "catalog_blake3": catalog_blake3, "source_bundle": source_logical}



def prepare_inputs(a, state, store, profile, archive, digest, table, python, rust,
                   linker, runtime_roots, *, source_logical=None):
    """Realize each declared dependency first; Nickel receives signed store IDs."""
    if source_logical is None:
        source_file = a.scratch / f"{profile}-source.ncl"
        source_file.write_text('let mantle = import "lib.ncl" in mantle.fetchTarball '
                               f'{{ name = "{profile}-bundle", url = {q(archive.as_uri())}, '
                               f'hash = {q(digest)} }}\n')
        source_report, _ = build(a, state, store, source_file, f"{profile}-source")
        source_logical = "/nix/store/" + Path(exact_output(source_report, f"{profile}-bundle")).name
    verify(a, state, store, source_logical, f"{profile}-bundle")
    require(nar(store / Path(source_logical).name) == digest,
            f"{profile} source archive differs after signing")
    full_file = a.scratch / f"{profile}-full-table.ncl"
    full_file.write_text(f'(import {q(str(FIXTURE / "full-table.ncl"))}) '
                         f'{{ url = {q(table.as_uri())}, sha256 = {q(sha(table))} }}\n')
    full_report, _ = build(a, state, store, full_file, f"{profile}-full-table")
    full_path = "/nix/store/" + Path(exact_output(full_report, "lock-vendor-reviewed-full-table")).name
    producer_file = a.scratch / f"{profile}-producer.ncl"
    producer_file.write_text(f'(import {q(str(FIXTURE / "producer.ncl"))}) '
                             f'{{ bundle = {q(source_logical)}, python = {q(python)}, '
                             f'rust = {q(rust)}, linker = {q(linker)}, '
                             f'runtime = {json.dumps(runtime_roots)} }}\n')
    producer_report, _ = build(a, state, store, producer_file, f"{profile}-producer")
    producer = "/nix/store/" + Path(exact_output(producer_report, "lock-vendor-producer")).name
    selected_file = a.scratch / f"{profile}-selected.ncl"
    selected_file.write_text(f'(import {q(str(FIXTURE / "selected-table.ncl"))}) '
                             f'{{ bundle = {q(source_logical)}, producer = {q(producer)}, '
                             f'full_table = {q(full_path)} }}\n')
    selected_report, _ = build(a, state, store, selected_file, f"{profile}-selected")
    selected = "/nix/store/" + Path(exact_output(selected_report, "lock-vendor-selected-shared-hashes")).name
    for label, path in (("full-table", full_path), ("producer", producer), ("selected", selected)):
        verify(a, state, store, path, f"{profile}-{label}")
    return source_logical, producer, selected


def render_plan(a, profile, source, producer, selected, python, rust, git, patch, runtime_roots):
    """All producer inputs are exact signed logical paths, not Nickel records."""
    run_roots = runtime_roots + ([str(git)] if git else []) + ([str(patch)] if patch else [])
    source_text = f'''let mantle = import "lib.ncl" in
let plan = (import {q(str(FIXTURE / 'mantle.ncl'))}) {{ producer = {q(producer)}, bundle = {q(source)},
  selected_table = {q(selected)}, python = {q(python)}, rust = {q(rust)},
  runtime = {json.dumps(run_roots)} }} in
{{ name = "{profile}-vendor-consumer", builder = {q(str(python / "bin/python3"))}, system = 'x86_64-linux,
  args = ["-c", "import os,pathlib; pathlib.Path(os.environ['out']).write_text(os.environ['VENDOR']+'\\\\n')"],
  env = {{ VENDOR = "{{{{mantle-plan-output:vendor}}}}" }},
  inputs = [{{ name = "vendor", producer = plan, plan_output = "plan", root = "vendor.complete", unit_output = "out" }}, {q(python)}],
  addressing_mode = 'input-addressed, sandbox = 'native }} | mantle.Derivation
'''
    dest = a.scratch / f"{profile}.ncl"
    dest.write_text(source_text)
    plan_root = a.scratch / f"{profile}-plan-only.ncl"
    plan_root.write_text(source_text.split(f'\n{{ name = "{profile}-vendor-consumer"', 1)[0]
                         + "\nplan\n")
    return dest


def negative_plan(a, state, store, identity, python):
    """Adversarial native-plan derivations mutate the *signed producer output* in bwrap."""
    artifact = identity["plan_artifact_path"]
    require(isinstance(artifact, str) and artifact.startswith("/nix/store/")
            and (store / Path(artifact).name).is_file(), "no signed producer plan artifact")
    verify(a, state, store, artifact, "native-plan")
    mutations = {
        "undeclared-fetch": "p['units'][-1]['derivation']['inputs'].append({'kind':'unit_output','unit':'fetch.undeclared','output':'out'})",
        "duplicate-ids": "p['units'].append(p['units'][0].copy())",
        "unit-limit": "p['units'].extend([dict(p['units'][0], id='fetch.excess.%04d'%i) for i in range(4100)])",
        "incorrect-hash": "p['units'][0]['derivation']['fixed_output']['hash']='0'*64",
    }
    results = {}
    for mode, mutation in mutations.items():
        code = (f"import json,os,sys; p=json.load(open(sys.argv[1])); {mutation}; "
                "json.dump(p,open(os.environ['plan'],'w'),separators=(',',':'))")
        source = f'''let mantle = import "lib.ncl" in
let forged = {{ name = "negative-{mode}-producer", builder = {q(str(python / "bin/python3"))},
  args = ["-c", {q(code)}, {q(artifact)}], outputs = ["plan"],
  dynamic_plan_outputs = ["plan"], inputs = [{q(artifact)}, {q(python)}],
  addressing_mode = 'input-addressed, sandbox = 'native }} | mantle.Derivation in
{{ name = "negative-{mode}-consumer", builder = {q(str(python / "bin/python3"))},
  args = ["-c", "import os; open(os.environ['out'],'w').write(os.environ['VENDOR'])"],
  env = {{ VENDOR = "{{{{mantle-plan-output:vendor}}}}" }},
  inputs = [{{ name = "vendor", producer = forged, plan_output = "plan",
              root = "vendor.complete", unit_output = "out" }}, {q(python)}],
  addressing_mode = 'input-addressed, sandbox = 'native }} | mantle.Derivation
'''
        file = a.scratch / f"negative-{mode}.ncl"
        file.write_text(source)
        report, rc = build(a, state, store, file, f"negative-{mode}", negative=True)
        if mode != "incorrect-hash":
            plans = report.get("native_dynamic_plans", [])
            expected = {"undeclared-fetch": "unit output dependency",
                        "duplicate-ids": "unit id", "unit-limit": "units"}[mode]
            require(len(plans) == 1 and plans[0]["scheduler_action"] == "rejected"
                    and expected in plans[0]["rejection_reason"],
                    f"{mode}: native admission did not deny for expected reason")
        else:
            plans = report.get("native_dynamic_plans", [])
            require(len(plans) == 1 and plans[0]["scheduler_action"] == "registered-roots",
                    "wrong SHA failed before native plan admitted the fetch")
            logs = [Path(row["saved_log_path"]) for row in report.get("failed", [])
                    if isinstance(row.get("saved_log_path"), str)]
            require(any(path.is_file() and
                        "FOD hash mismatch for lock-fetch-" in path.read_text(errors="replace")
                        for path in logs),
                    "wrong SHA did not fail a selected fixed-output fetch")
        results[mode] = {"exit_code": rc, "reported_failure": report.get("failed", []),
                         "plan_action": (report.get("native_dynamic_plans") or [{}])[0].get("scheduler_action")}
    return results


def negative_assembler(a, state, store, identity, bundle, selected, python, rust):
    """Run actual assembler in native bwrap with signed fetches; deny partial output."""
    plan = json.loads((store / Path(identity["plan_artifact_path"]).name).read_text())
    units = {unit["id"]: unit for unit in plan["units"]}
    final = units["vendor.complete"]["derivation"]
    locked = {f"https://static.crates.io/crates/{p['name']}/{p['name']}-{p['version']}.crate": p
              for p in tomllib.loads((FIXTURE / "Cargo.lock").read_text())["package"]
              if p.get("source", "").startswith("registry+")}
    specs = []
    for url, p in sorted(locked.items()):
        archive = a.registry_cache / f"{p['name']}-{p['version']}.crate"
        require(archive.is_file() and sha(archive) == p["checksum"],
                f"missing exact-source input archive for assembler negative: {p['name']}")
        specs.append(((p["name"], p["version"]), ("flat", archive.as_uri(), p["checksum"])))
    published = publisher_batch(a, state, store, specs, 99, python)
    outputs = {}
    for unit_id, unit in units.items():
        if not unit_id.startswith("fetch."):
            continue
        url = unit["derivation"]["env"]["url"]
        require(url in locked, f"native fetch is not a two-crate locked URL: {url}")
        package = locked[url]
        outputs[unit_id] = published[(package["name"], package["version"])]
    require(len(outputs) == 2, "incorrect signed fetch cardinality in fixture")
    for index, logical in enumerate(sorted(outputs.values())):
        verify(a, state, store, logical, f"two-crate-fetch-{index}")
    args = [arg.replace("{{mantle-source:src.bundle}}", bundle)
            .replace("{{mantle-source:src.table}}", selected)
            .replace("{{mantle-source:src.rust}}", str(rust))
            for arg in final["args"]]
    for position, arg in enumerate(args):
        for unit_id, path in outputs.items():
            args[position] = args[position].replace("{{mantle-unit-output:" + unit_id + ":out}}", path)
    require(len([arg for arg in args if arg.startswith("/nix/store/") and arg in outputs.values()]) == 2,
            "cannot resolve two signed fetch artifact arguments")
    target = next(i for i, arg in enumerate(args) if arg in outputs.values())
    originals = sorted(outputs.values())
    denials = {}
    for label, replacement in (("missing-artifact", "/nix/store/" + "0" * 32 + "-missing.crate"),
                               ("wrong-unpacked-hash", next(p for p in originals if p != args[target]))):
        broken = args.copy()
        broken[target] = replacement
        source = ('let mantle = import "lib.ncl" in { '
                  f'name = "lv-assembler-{label}", builder = {q(str(python / "bin/python3"))}, '
                  f'args = {json.dumps(broken)}, outputs = ["out"], '
                  f'inputs = {json.dumps([bundle, selected, str(python), str(rust)] + originals)}, '
                  "addressing_mode = 'input-addressed, sandbox = 'native } | mantle.Derivation\n")
        file = a.scratch / f"assembler-{label}.ncl"
        file.write_text(source)
        failed, rc = build(a, state, store, file, f"assembler-{label}", negative=True)
        require(failed.get("counts", {}).get("failed_total", 0) > 0
                and not any(r.get("label") == f"lv-assembler-{label}" for r in failed.get("outcomes", [])),
                f"assembler {label} published output")
        logs = [Path(row["saved_log_path"]) for row in failed["failed"]
                if row.get("saved_log_path")]
        require(any(path.is_file() and "archive mismatch" in path.read_text(errors="replace")
                    for path in logs), f"assembler {label} did not name the offending archive")
        denials[label] = {"exit_code": rc, "failed_total": failed["counts"]["failed_total"],
                          "failure_logs": [str(path) for path in logs]}
    return denials

def two_crate(a, state, store, python, rust, linker, git, base_runtime):
    archive, digest = source_archive(a, "two-crate")
    fixture_lock = tomllib.loads((FIXTURE / "Cargo.lock").read_text())
    packages = [p for p in fixture_lock["package"] if p.get("source", "").startswith("registry+")]
    require(len(packages) == 2 and all(HEX.fullmatch(p.get("checksum", "")) for p in packages),
            "two-crate fixture changed")
    bundle, producer, table = prepare_inputs(a, state, store, "two-crate", archive, digest,
                                             FIXTURE / "shared-lock-hashes-v1", python, rust,
                                             linker, base_runtime)
    root_file = render_plan(a, "two-crate", bundle, producer, table,
                            python, rust, None, None, base_runtime)
    report, _ = build(a, state, store, root_file, "two-crate")
    identity = accepted(report, 2)
    vendor = store / Path(identity["vendor_path"]).name
    require(vendor.is_dir() and all((vendor / p["name"] / ".cargo-checksum.json").is_file()
                                   for p in packages), "signed vendor tree missing locked crates")
    require({entry.name for entry in vendor.iterdir()} == {p["name"] for p in packages},
            "assembler published unexpected package layout")
    for p in packages:
        package = vendor / p["name"]
        checksum = json.loads((package / ".cargo-checksum.json").read_text())
        require(checksum["package"] == p["checksum"], f"assembler source hash mismatch: {p['name']}")
        for relative, expected in checksum["files"].items():
            path = package / relative
            require(path.is_file() and not path.is_symlink()
                    and path.resolve().is_relative_to(package.resolve())
                    and sha(path) == expected, f"assembled payload checksum mismatch: {p['name']}/{relative}")
    identity["source_bundle_nar_sha256"] = digest
    identity["vendor_nar_sha256"] = nar(vendor)
    identity["lock_sha256"] = sha(FIXTURE / "Cargo.lock")
    identity["artifacts"] = [f"{p['name']}@{p['version']}:{p['checksum']}" for p in packages]
    verify(a, state, store, identity["vendor_path"], "two-crate-vendor")
    plan_report, _ = build(a, state, store, a.scratch / "two-crate-plan-only.ncl",
                           "two-crate-plan-only")
    exported_plan = exact_output(plan_report, "lock-vendor-two-crate-plan", "plan")
    require(Path(exported_plan).name == Path(identity["plan_artifact_path"]).name,
            "exported native plan is not the accepted binding's immutable artifact")
    verify(a, state, store, identity["plan_artifact_path"], "two-crate-plan-artifact")
    identity["negative_controls"] = negative_plan(a, state, store, identity, python)
    identity["negative_controls"].update(
        negative_assembler(a, state, store, identity, bundle, table, python, rust))
    (a.scratch / "two-crate-accepted.json").write_text(json.dumps(identity, indent=2) + "\n")
    return identity


def realize_provider(a, state, store):
    """Pin the real musl.cc legacy seed from its checked-in provider recipe."""
    recipe = REPO / "bootstrap/seed-legacy.ncl"
    require(recipe.is_file() and "sha256-ZtQZncMvugqmS7OMvMtdhY5MMtem4KXBhamxWbHUDkY="
            in recipe.read_text(), "legacy provider pin drift")
    # Intermediate PathInfo does not promise physical export; demand the raw
    # tarball as an explicit build root before consuming it as a profile input.
    raw_wrapper = a.scratch / "raw-provider-root.ncl"
    raw_wrapper.write_text('let mantle = import "lib.ncl" in { '
                           'raw = mantle.fetchTarball { name = "musl-gcc-raw", '
                           'url = "https://musl.cc/x86_64-linux-musl-native.tgz", '
                           'hash = "sha256-ZtQZncMvugqmS7OMvMtdhY5MMtem4KXBhamxWbHUDkY=" } }\n')
    raw_result, _ = build(a, state, store, raw_wrapper, "raw-provider-root")
    archive = Path(exact_output(raw_result, "raw"))
    wrapper = a.scratch / "real-provider.ncl"
    wrapper.write_text('let mantle = import "lib.ncl" in '
                       f'let seed = import {q(str(recipe))} in {{ '
                       'raw = mantle.fetchTarball { name = "musl-gcc-raw", '
                       'url = "https://musl.cc/x86_64-linux-musl-native.tgz", '
                       'hash = "sha256-ZtQZncMvugqmS7OMvMtdhY5MMtem4KXBhamxWbHUDkY=" }, '
                       'normalized = seed.toolchain }\n')
    result, _ = build(a, state, store, wrapper, "real-provider")
    require(exact_output(result, "raw") == str(archive),
            "normalization did not consume the separately exported pinned raw provider")
    normalized = Path(exact_output(result, "normalized"))
    for name, path in (("raw-provider", archive), ("normalized-provider", normalized)):
        verify(a, state, store, "/nix/store/" + path.name, name)
    manifest = normalized / "share/crunch-bootstrap/provider.json"
    facts = json.loads(manifest.read_text())
    require(facts["provider_id"] == "musl.cc-native-reduced-v1"
            and facts["raw"]["hash"] == "sha256-ZtQZncMvugqmS7OMvMtdhY5MMtem4KXBhamxWbHUDkY="
            and facts["reduction"]["raw_size_bytes"] > facts["reduction"]["reduced_size_bytes"],
            "normalized provider manifest differs from genuine reduction")
    a.provider_archive, a.provider_manifest = archive, manifest
    return {"raw_path": str(archive), "normalized_path": str(normalized),
            "manifest_sha256": sha(manifest)}

def hydrate_profile(a, state, store, identity, producer_path, selected_path):
    """Receipt metadata must agree with the accepted native report, never replace it."""
    require(a.provider_archive and a.provider_manifest
            and a.provider_archive.exists() and a.provider_manifest.is_file(),
            "real musl.cc normalized-provider archive and manifest must be supplied")
    excluded = a.excluded_vendor
    output = store / Path(identity["vendor_path"]).name
    require(excluded.is_dir() and output.is_dir() and nar(excluded) == nar(output),
            "new full-lock assembler differs from excluded immutable vendor tree")
    verify(a, state, store, identity["vendor_path"], "full-vendor")
    producer = store / Path(producer_path).name
    selected = store / Path(selected_path).name
    for label, path in (("producer", producer), ("selected", selected)):
        verify(a, state, store, "/nix/store/" + path.name, label)
    checkout = a.scratch / "fresh-checkout"
    run(["git", "clone", "--no-local", REPO, checkout], stdout=a.scratch / "fresh-clone.log")
    require(sha(checkout / "Cargo.lock") == sha(REPO / "Cargo.lock")
            and not (checkout / "vendor-deps").exists(), "fresh clone lock or vendor exclusion drift")
    missing_cmd = [a.cli, "--json", "--nix-compat", "--store", store, "--state-dir", state,
                   "self-build", "--no-substitute", "--signing-key", a.signing_key,
                   "--trusted-public-keys", a.trusted_key]
    missing, _ = run(missing_cmd, cwd=checkout, stdout=a.scratch / "self-build-without-hydration.json",
                     env={**os.environ, "TMPDIR": str(a.scratch)}, expected=1)
    stderr = (a.scratch / "self-build-without-hydration.json.stderr").read_text()
    require("required staged source entry missing:" in missing + stderr
            and "/vendor-deps" in missing + stderr
            and not (checkout / "vendor-deps").exists(),
            "self-build did not reject missing vendor-deps before hydration")
    manifest = a.scratch / "fresh-clone-profile.json"
    profile_args = ["source", "bundle", "bootstrap-profile", "--mode", "fresh-clone-lock-vendor-inputs",
                    "--provider-archive", a.provider_archive,
                    "--provider-manifest", a.provider_manifest, "--mantle-source", checkout,
                    "--lock-vendor-producer", producer, "--lock-vendor-selected-table", selected,
                    "--lock-vendor-output", output, "--excluded-vendor-deps", excluded,
                    "--lock-vendor-parent-drv-path", identity["parent_drv_path"],
                    "--lock-vendor-plan-digest", identity["canonical_plan_digest"],
                    "--to", manifest]
    profile, _ = cli(a, state, store, profile_args, a.scratch / "profile-report.json")
    require(profile["lock_vendor_parent_drv_path"] == identity["parent_drv_path"]
            and profile["lock_vendor_plan_digest"] == identity["canonical_plan_digest"],
            "profile disagrees with actual accepted native plan")
    manifest_hash = profile["manifest_blake3"]
    require(HEX.fullmatch(manifest_hash), "missing canonical profile digest")
    recorded = json.loads(manifest.read_text())
    require(not any(r["identity"] == "vendored-cargo-inputs" for r in recorded["records"])
            and any(r["identity"] == "lock-vendor-acquisition" for r in recorded["records"]),
            "excluded vendor tree leaked into fresh-clone profile")
    hydrate = ["source", "bundle", "hydrate-self-build", "--from", manifest,
               "--expected-manifest-blake3", manifest_hash, "--checkout", checkout,
               "--lock-vendor-output", output,
               "--lock-vendor-parent-drv-path", identity["parent_drv_path"],
               "--lock-vendor-plan-digest", identity["canonical_plan_digest"]]
    denials = {}
    cases = {
        "missing-output": hydrate[:hydrate.index("--lock-vendor-output")] +
                          hydrate[hydrate.index("--lock-vendor-parent-drv-path"):],
        "wrong-digest": [("0" * 64 if arg == identity["canonical_plan_digest"] else arg) for arg in hydrate],
        "forged-parent": [("/nix/store/" + "0" * 32 + "-forged.drv"
                           if arg == identity["parent_drv_path"] else arg) for arg in hydrate],
    }
    for name, args in cases.items():
        failure, rc = cli(a, state, store, args, a.scratch / f"hydrate-{name}.json", negative=True)
        require(not (checkout / "vendor-deps").exists(),
                f"{name} published partial vendor before refusing hydration")
        denials[name] = {"exit_code": rc, "diagnostic": failure}
    hydrated, _ = cli(a, state, store, hydrate, a.scratch / "hydrate-accepted.json")
    require((checkout / "vendor-deps").is_dir()
            and nar(checkout / "vendor-deps") == nar(output), "hydrated vendor tree differs from accepted output")
    require(hydrated["manifest_blake3"] == manifest_hash, "hydrate receipt source identity mismatch")
    package = next(p for p in sorted((checkout / "vendor-deps").iterdir())
                   if (p / ".cargo-checksum.json").is_file()
                   and json.loads((p / ".cargo-checksum.json").read_text())["files"])
    relative = sorted(json.loads((package / ".cargo-checksum.json").read_text())["files"])[0]
    selected_file = package / relative
    require(selected_file.is_file() and not selected_file.is_symlink(), "no checked vendor file for mismatch")
    previous = selected_file.read_bytes()
    original_mode = selected_file.stat().st_mode & 0o777
    try:
        selected_file.chmod(original_mode | 0o200)
        selected_file.write_bytes(previous + b"\n# hydration-integrity-negative\n")
        _, rc = run(missing_cmd, cwd=checkout, stdout=a.scratch / "self-build-mismatched-hydration.json",
                    env={**os.environ, "TMPDIR": str(a.scratch)}, expected=1)
        text = (a.scratch / "self-build-mismatched-hydration.json.stderr").read_text()
        require("checksum mismatch" in text or "vendor file" in text,
                "self-build did not refuse mismatched hydrated package")
        denials["self-build-mismatched-hydration"] = {"exit_code": rc,
                                                     "offending_artifact": str(selected_file)}
    finally:
        selected_file.write_bytes(previous)
        selected_file.chmod(original_mode)
    require(nar(checkout / "vendor-deps") == nar(output),
            "mismatch control did not restore exact hydrated vendor tree")
    return {"profile_manifest_blake3": manifest_hash,
            "excluded_vendor_bytes": profile["excluded_vendor_payload_bytes"],
            "hydrated_vendor_nar_sha256": nar(output), "denials": denials,
            "fresh_checkout": str(checkout)}


def workspace(a, state, store, python, rust, linker, git, base_runtime):
    provider = realize_provider(a, state, store)
    rows = workspace_rows(a)
    git_trees = prepare_git_sidecars(a, rows)
    signed_git_trees = import_signed_roots(
        a, state, store, [path for _, path in git_trees.values()], "git")
    archive, digest = source_archive(a, "workspace", workspace=True)
    offline, signed = signed_catalog(a, state, store, rows, archive, digest, git_trees, python)
    patch = root(shutil.which("patch"))
    extra_runtime = sorted(set(base_runtime + runtime(git / "bin/git", patch / "bin/patch")
                               ) - {str(git), str(patch), str(python), str(rust), str(linker)})
    bundle, producer, selected = prepare_inputs(a, state, store, "workspace", archive, digest,
                                                a.table, python, rust, linker, extra_runtime,
                                                source_logical=signed["source_bundle"])
    render = render_plan(a, "workspace", bundle, producer, selected,
                         python, rust, git, patch, extra_runtime)
    report, _ = build(a, state, store, render, "workspace", offline=offline)
    identity = accepted(report, len(rows))
    identity["provider"] = provider
    identity["signed_historical_git_trees"] = signed_git_trees
    identity["source_cache"] = signed
    identity["profile"] = hydrate_profile(a, state, store, identity, producer, selected)
    (a.scratch / "workspace-accepted.json").write_text(json.dumps(identity, indent=2) + "\n")
    return identity

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, default=DEFAULT_ROOT / "targets/rust-efficiency-20261011/release/mantle")
    parser.add_argument("--scratch", type=Path, default=DEFAULT_ROOT / "tmp/rust-efficiency-20261011/lock-vendor")
    parser.add_argument("--store-root", type=Path, default=DEFAULT_ROOT / "stores/rust-efficiency-20261011")
    parser.add_argument("--publisher-name", default="lock-vendor-publisher")
    parser.add_argument("--mode", choices=("two-crate", "workspace", "all"), default="all")
    parser.add_argument("--b3sum", type=Path, default=shutil.which("b3sum"),
                        help="pinned Nix BLAKE3 binary for out-of-band signed catalog digest")
    historical = DEFAULT_ROOT / "tmp/add-lock-driven-vendor-fetches"
    parser.add_argument("--index", type=Path, default=historical / "baseline/cargo-home/registry/index/index.crates.io-1949cf8c6b5b557f")
    parser.add_argument("--git-cache", type=Path, default=historical / "baseline/cargo-home/git")
    parser.add_argument("--git-trees", type=Path, default=historical / "exp/trees.json")
    parser.add_argument("--registry-cache", type=Path, default=historical / "baseline/cargo-home/registry/cache/index.crates.io-1949cf8c6b5b557f")
    parser.add_argument("--excluded-vendor", type=Path, default=REPO / "vendor-deps")
    parser.add_argument("--table", type=Path, default=REPO / "bootstrap/pins/cargo-shared-lock-hashes-v1")
    a = parser.parse_args()
    require(a.cli.is_file() and a.cli.is_relative_to(DEFAULT_ROOT / "targets/rust-efficiency-20261011"),
            "requires current cohort Mantle binary")
    require(a.mode == "two-crate" or
            (a.b3sum is not None and a.b3sum.is_file()
             and str(a.b3sum.resolve()).startswith("/nix/store/")),
            "full-lock replay needs pinned --b3sum /nix/store/.../bin/b3sum")
    require(not a.scratch.exists(), f"scratch already exists: {a.scratch}")
    require(a.scratch.parent.is_dir(), "dedicated scratch parent does not exist")
    require(re.fullmatch(r"lock-vendor-publisher(?:-[a-z0-9]+)?", a.publisher_name),
            "publisher name must remain in isolated cohort namespace")
    store = a.store_root / a.publisher_name
    state = a.store_root / (a.publisher_name + "-state")
    require(not store.exists() and not state.exists() and a.store_root.is_dir(),
            "publisher store/state must be fresh and under dedicated cohort")
    a.scratch.mkdir()
    store.mkdir()
    state.mkdir()
    a.key_receipt = create_key(a)
    a.cli = a.cli.resolve()
    python = root(shutil.which("python3"))
    rust = Path("/nix/store/1hz41jg6cc3py3szsn6cxw3m5p42r4fj-rust-default-1.96.0-nightly-2026-04-08")
    linker = Path("/nix/store/hb2bs5fg5wkm04x565737qd5nh2hy5nk-gcc-wrapper-15.2.0")
    git = root(shutil.which("git"))
    require((python / "bin/python3").exists() and (rust / "bin/rustc").exists()
            and (linker / "bin/cc").exists() and (git / "bin/git").exists(), "missing pinned toolchain")
    base_runtime = [p for p in runtime(python / "bin/python3", rust / "bin/rustc", linker / "bin/cc")
                    if p not in (str(python), str(rust), str(linker), str(git))]
    try:
        report = {"schema": "mantle-lock-vendor-replay-v1", "cli_sha256": sha(a.cli),
                  "scratch": str(a.scratch), "publisher_store": str(store),
                  "publisher_state": str(state), "key": a.key_receipt}
        report["input_cache"] = import_signed_runtime(a, state, store, python, rust, linker, git, base_runtime)
        if a.mode in ("two-crate", "all"):
            report["two_crate"] = two_crate(a, state, store, python, rust, linker, git, base_runtime)
        if a.mode in ("workspace", "all"):
            require(a.index.is_dir() and a.git_cache.is_dir() and a.git_trees.is_file()
                    and a.registry_cache.is_dir() and a.table.is_file(),
                    "full lock requires immutable registry/Git input cache and reviewed table")
            report["workspace"] = workspace(a, state, store, python, rust, linker, git, base_runtime)
        report["result"] = "passed"
    except Exception as error:
        report = locals().get("report", {"schema": "mantle-lock-vendor-replay-v1"})
        report["result"] = "failed"
        report["error"] = str(error)
        (a.scratch / "report.json").write_text(json.dumps(report, indent=2) + "\n")
        raise
    (a.scratch / "report.json").write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()
