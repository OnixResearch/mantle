#!/usr/bin/env python3
"""Build the lock-vendor producer solely from declared source and toolchain paths."""
import hashlib
import os
from pathlib import Path
import subprocess
import shutil
import sys
import tempfile


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def main():
    require(len(sys.argv) == 4, "usage: vendor-producer-build.py <source-bundle> <rustc> <linker>")
    bundle, rustc, linker = (Path(value) for value in sys.argv[1:])
    require(bundle.is_dir() and not bundle.is_symlink(), "missing declared source bundle")
    require(rustc.is_file() and linker.is_file(), "missing declared Rust compiler or linker")
    core = bundle / "producer-src/core/lib.rs"
    shared = bundle / "producer-src/core/shared_table.rs"
    producer = bundle / "producer-src/main.rs"
    require(all(path.is_file() and not path.is_symlink() for path in (core, shared, producer)),
            "incomplete declared producer source")
    output = Path(os.environ["out"])
    require(output.parent.is_dir() and not output.exists(), "invalid producer output path")
    with tempfile.TemporaryDirectory(prefix="lock-vendor-producer-", dir=Path.cwd()) as scratch:
        bindir = Path(scratch) / "bin"
        bindir.mkdir()
        library = Path(scratch) / "libmantle_lock_vendor_core.rlib"
        subprocess.run([str(rustc), "--edition=2024", "--crate-name", "mantle_lock_vendor_core",
                        "--crate-type=rlib", "-C", "debuginfo=0", "-C", "opt-level=2",
                        str(core), "-o", str(library)], check=True, cwd=scratch)
        subprocess.run([str(rustc), "--edition=2024", "--crate-name", "mantle_lock_vendor_producer",
                        "-C", "debuginfo=0", "-C", "opt-level=2", "-C", f"linker={linker}",
                        "--extern", f"mantle_lock_vendor_core={library}", str(producer),
                        "-o", str(bindir / "lock-vendor-producer")], check=True, cwd=scratch)
        executable = bindir / "lock-vendor-producer"
        require(executable.is_file(), "compiler did not produce executable")
        output.mkdir()
        published = output / "bin"
        published.mkdir()
        published_executable = published / executable.name
        shutil.copyfile(executable, published_executable)
        published_executable.chmod(0o755)
        with executable.open("rb") as built, published_executable.open("rb") as copied:
            require(hashlib.file_digest(built, "sha256").digest() == hashlib.file_digest(copied, "sha256").digest(),
                    "producer binary changed during publication")


if __name__ == "__main__":
    main()
