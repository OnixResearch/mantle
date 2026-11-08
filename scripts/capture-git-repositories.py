#!/usr/bin/env python3
"""Capture git dependency repositories at their locked revisions.

Source-built planning resolves a git-sourced package inside a captured
repository tree so intra-repository path dependencies keep their real layout.
This script materialises `<vendor>/git-repositories/<host>/<path>` from the
repository at the exact revision recorded in `Cargo.lock`, preferring local
repository databases and fetching only when the revision is absent.

It writes `<vendor>/git-repositories-capture.json` (schema
`mantle-captured-git-repositories-v1`) and never executes repository code.
"""
import argparse
import hashlib
import json
import os
import re
import subprocess
import sys

SCHEMA = "mantle-captured-git-repositories-v1"
DIR = "git-repositories"
GIT_PREFIX = "git+"


def run(args, **kwargs):
    return subprocess.run(args, check=True, capture_output=True, **kwargs)


def repository_relative(source):
    """Mirror the planner's normalization: scheme, query and fragment removed."""
    assert source.startswith(GIT_PREFIX), source
    body = source[len(GIT_PREFIX):]
    body = body.split("#", 1)[0]
    body = body.split("?", 1)[0]
    for prefix in ("https://", "http://", "ssh://git@", "ssh://", "git://"):
        if body.startswith(prefix):
            body = body[len(prefix):]
            break
    else:
        raise SystemExit(f"unsupported git source scheme: {source}")
    if body.startswith("git@"):
        body = body[len("git@"):]
    if body.endswith(".git"):
        body = body[:-4]
    components = [component for component in body.split("/") if component]
    if not components or any(component in (".", "..") or "\\" in component for component in components):
        raise SystemExit(f"invalid git source path: {source}")
    return components


def locked_revision(source):
    if "#" not in source:
        raise SystemExit(f"git source has no resolved revision fragment: {source}")
    revision = source.rsplit("#", 1)[1]
    if not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise SystemExit(f"git source revision is not a commit id: {source}")
    return revision


def local_repository(candidates, revision):
    for candidate in candidates:
        if not os.path.isdir(candidate):
            continue
        probe = subprocess.run(
            ["git", "-C", candidate, "cat-file", "-e", f"{revision}^{{commit}}"], capture_output=True
        )
        if probe.returncode == 0:
            return candidate
    return None


def fetch_repository(url, revision, cache):
    os.makedirs(cache, exist_ok=True)
    name = hashlib.blake3(url.encode()).hexdigest()[:16] if hasattr(hashlib, "blake3") else hashlib.sha256(url.encode()).hexdigest()[:16]
    repository = os.path.join(cache, f"{name}.git")
    if not os.path.isdir(repository):
        run(["git", "init", "--quiet", "--bare", repository])
    direct = subprocess.run(
        ["git", "-C", repository, "fetch", "--quiet", url, f"+{revision}:refs/mantle-capture"], capture_output=True
    )
    if direct.returncode != 0:
        run(["git", "-C", repository, "fetch", "--quiet", "--tags", url, "+refs/heads/*:refs/heads/*"])
    probe = subprocess.run(["git", "-C", repository, "cat-file", "-e", f"{revision}^{{commit}}"], capture_output=True)
    if probe.returncode != 0:
        raise SystemExit(f"cannot fetch {url} at {revision}")
    return repository


def extract(repository, revision, destination):
    if os.path.isdir(destination):
        subprocess.run(["rm", "-rf", destination], check=True)
    os.makedirs(destination)
    archive = run(["git", "-C", repository, "archive", "--format=tar", revision]).stdout
    subprocess.run(["tar", "-x", "-C", destination], input=archive, check=True)


def materialize_internal_symlinks(root):
    """The bundle carries symlinks only when their target has no `..` component.

    A repository may link to a sibling or directory through `..`; such a link is
    replaced by a copy of the resolved in-tree target. A link that escapes the
    repository is rejected instead of being captured.
    """
    root_real = os.path.realpath(root)
    replaced = 0
    pending = [root]
    while pending:
        directory = pending.pop()
        with os.scandir(directory) as entries:
            for entry in sorted(entries, key=lambda item: item.name):
                path = entry.path
                if not entry.is_symlink():
                    if entry.is_dir(follow_symlinks=False):
                        pending.append(path)
                    continue
                resolved = os.path.realpath(path)
                if not resolved.startswith(root_real + os.sep):
                    raise SystemExit(
                        f"captured repository symlink escapes its tree: {os.path.relpath(path, root)}"
                    )
                if os.path.isdir(resolved):
                    os.remove(path)
                    subprocess.run(["cp", "-a", resolved, path], check=True)
                else:
                    with open(resolved, "rb") as handle:
                        content = handle.read()
                    executable = os.access(resolved, os.X_OK)
                    os.remove(path)
                    with open(path, "wb") as handle:
                        handle.write(content)
                    if executable:
                        os.chmod(path, 0o755)
                replaced += 1
    return replaced


def digest_tree(root):
    digest = hashlib.sha256()
    files = 0
    for directory, directories, names in os.walk(root):
        directories.sort()
        for name in sorted(names):
            path = os.path.join(directory, name)
            relative = os.path.relpath(path, root)
            if os.path.islink(path):
                target = os.readlink(path)
                digest.update(f"link\0{relative}\0{target}\0".encode())
                files += 1
                continue
            if not os.path.isfile(path):
                continue
            with open(path, "rb") as handle:
                content = handle.read()
            mode = "755" if os.access(path, os.X_OK) else "644"
            digest.update(f"{mode}\0{relative}\0{len(content)}\0".encode())
            digest.update(content)
            files += 1
    return digest.hexdigest(), files


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--lock", required=True)
    parser.add_argument("--vendor-inputs", required=True)
    parser.add_argument("--repository-dir", action="append", default=[])
    parser.add_argument("--cache", required=True)
    parser.add_argument("--offline", action="store_true")
    arguments = parser.parse_args()

    text = open(arguments.lock, encoding="utf-8").read()
    sources = set(re.findall(r'^source = "(git\+[^"]+)"', text, re.MULTILINE))
    repositories = {}
    for source in sorted(sources):
        relative = repository_relative(source)
        revision = locked_revision(source)
        url = source[len(GIT_PREFIX):].split("#", 1)[0].split("?", 1)[0]
        key = "/".join(relative)
        entry = repositories.setdefault(key, {"source": source, "url": url, "revision": revision})
        if entry["revision"] != revision:
            raise SystemExit(f"one repository directory claims two revisions: {key}")

    candidates = list(arguments.repository_dir) + sorted(
        os.path.join(os.path.expanduser("~/.cargo/git/db"), name)
        for name in os.listdir(os.path.expanduser("~/.cargo/git/db"))
    )
    report = {"schema": SCHEMA, "lock": arguments.lock, "repositories": []}
    for key, entry in sorted(repositories.items()):
        destination = os.path.join(arguments.vendor_inputs, DIR, *key.split("/"))
        origin = local_repository(candidates, entry["revision"])
        if origin is None:
            if arguments.offline:
                raise SystemExit(f"revision {entry['revision']} for {entry['url']} is not available offline")
            origin = fetch_repository(entry["url"], entry["revision"], arguments.cache)
        extract(origin, entry["revision"], destination)
        materialized_symlinks = materialize_internal_symlinks(destination)
        content_digest, files = digest_tree(destination)
        report["repositories"].append({
            "repository": key,
            "url": entry["url"],
            "revision": entry["revision"],
            "origin": "local" if origin in candidates else "fetched",
            "files": files,
            "materialized_symlinks": materialized_symlinks,
            "content_sha256": content_digest,
        })
        print(f"captured {key} at {entry['revision']} ({files} files)")

    with open(os.path.join(arguments.vendor_inputs, f"{DIR}-capture.json"), "w", encoding="utf-8") as handle:
        json.dump(report, handle, indent=2, sort_keys=True)
        handle.write("\n")
    print(f"repositories captured: {len(report['repositories'])}")


if __name__ == "__main__":
    sys.exit(main())
