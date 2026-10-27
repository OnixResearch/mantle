#!/usr/bin/env python3
"""Real local watch edit/add/invalid/coalescing/delete rail; never builds Mantle.

Requires Linux /proc and a freshly built Mantle with working Bubblewrap/pidfds.
Uses a new scratch directory and retains all sources, logs, and result.json.
"""

import argparse
import json
from pathlib import Path
import re
import signal
import subprocess
import time


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def atomic_write(path, text):
    temporary = path.with_suffix(path.suffix + ".next")
    temporary.write_text(text)
    temporary.replace(path)


def descendants(pid):
    pending, found = [pid], {}
    while pending:
        parent = pending.pop()
        try:
            tasks = list(Path(f"/proc/{parent}/task").iterdir())
            children = []
            for task in tasks:
                try:
                    children.extend((task / "children").read_text().split())
                except FileNotFoundError:
                    continue
        except FileNotFoundError:
            continue
        for child in map(int, children):
            if child in found:
                continue
            try:
                stat = Path(f"/proc/{child}/stat").read_text().rsplit(") ", 1)[1].split()
                command = Path(f"/proc/{child}/cmdline").read_bytes().split(b"\0")
            except FileNotFoundError:
                continue
            found[child] = {"starttime": stat[19], "command": [part.decode(errors="replace") for part in command]}
            pending.append(child)
    return found


def stopped(pid, identity):
    try:
        stat = Path(f"/proc/{pid}/stat").read_text().rsplit(") ", 1)[1].split()
    except FileNotFoundError:
        return True
    return stat[19] != identity["starttime"] or stat[0] == "Z"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mantle", required=True, type=Path)
    parser.add_argument("--scratch", required=True, type=Path, help="must not already exist")
    args = parser.parse_args()
    binary = args.mantle.resolve(strict=True)
    scratch = args.scratch.absolute()
    scratch.mkdir(parents=True, exist_ok=False)
    source = scratch / "roots.ncl"
    store = scratch / "store"
    store.mkdir()
    revisions = scratch / "source-revisions"
    revisions.mkdir()
    transitions = []

    def record_transition(name, offset, **observations):
        events = stdout.read_text()[offset:]
        path = scratch / f"{len(transitions):02d}-{name}.log"
        path.write_text(events)
        snapshot = revisions / f"{len(transitions):02d}-{name}"
        snapshot.mkdir()
        for edited in scratch.glob("*.ncl"):
            (snapshot / edited.name).write_bytes(edited.read_bytes())
        transitions.append({"scenario": name, "log": str(path), "stdout_offset_chars": offset,
                            "source_snapshot": str(snapshot), **observations})
    library = Path(__file__).resolve().parents[1] / "lib/lib.ncl"
    prefix = f"watch-{scratch.name}"
    require(re.fullmatch(r"[a-zA-Z0-9_-]+", prefix), "scratch basename must use letters, digits, hyphens or underscores")

    def derivation(name, text, slow=False):
        command = f"printf '%s\\n' '{text}' > \"$out\""
        if slow:
            command += "; /bin/busybox sleep 20; printf '%s\\n' 'late-output' >> \"$out\""
        return (f"let mantle = import {json.dumps(str(library))} in\n"
                + "{ name = " + json.dumps(f"{prefix}-{name}")
                + ', builder = "/bin/sh", args = ["-c", ' + json.dumps(command)
                + "] } | mantle.Derivation\n")

    keep = scratch / "keep.ncl"
    moving = scratch / "moving.ncl"
    added = scratch / "added.ncl"
    slow = scratch / "slow.ncl"
    atomic_write(keep, derivation("keep", "keep"))
    atomic_write(moving, derivation("moving-v1", "moving-v1"))

    def select(names):
        atomic_write(source, "{ " + ", ".join(f'{name} = import "{name}.ncl"' for name in names) + " }\n")

    select(["keep", "moving"])
    stdout = scratch / "watch.stdout.log"
    stderr = scratch / "watch.stderr.log"
    command = [str(binary), "--store", str(store), "--state-dir", str(scratch / "state"),
               "build", "--watch", str(source), "--no-substitute", "--jobs", "2"]
    (scratch / "command.json").write_text(json.dumps(command, indent=2) + "\n")
    result = {"status": "failed", "command": command, "scratch": str(scratch)}
    with stdout.open("wb") as output, stderr.open("wb") as errors:
        process = subprocess.Popen(command, stdout=output, stderr=errors)
        try:
            def wait_until(predicate, message, seconds=40):
                deadline = time.monotonic() + seconds
                while time.monotonic() < deadline:
                    require(process.poll() is None, f"watch exited {process.returncode}: see {stderr}")
                    value = predicate()
                    if value:
                        return value
                    time.sleep(0.05)
                raise RuntimeError(message)

            def log():
                return stdout.read_text()

            def settle_after(offset, count):
                wait_until(lambda: re.search(rf"generation \d+: settled {count} successful, 0 failed root\(s\)", log()[offset:]),
                           f"expected a settled {count}-root generation after byte {offset}")
                # Goal lines precede Settled; physical artifacts must also exist.

            def artifact(name, contents):
                matches = list(store.glob(f"*-{prefix}-{name}"))
                require(len(matches) == 1, f"expected one {name} artifact, found {matches}")
                path = matches[0]
                require(path.read_text() == contents + "\n", f"wrong contents in {path}")
                stat = path.stat()
                return {"path": str(path), "inode": stat.st_ino, "mtime_ns": stat.st_mtime_ns,
                        "blake3": subprocess.check_output(["b3sum", "--no-names", str(path)], text=True).strip()}

            settle_after(0, 2)
            retained = artifact("keep", "keep")
            artifact("moving-v1", "moving-v1")
            record_transition("initial", 0, retained=retained)
            offset = len(log())
            atomic_write(moving, derivation("moving-v2", "moving-v2"))
            settle_after(offset, 2)
            edit_events = log()[offset:]
            require("  retained " in edit_events and "  retracted " in edit_events and "  added " in edit_events,
                    "one-root edit did not classify retained/retracted/added identities")
            artifact("moving-v2", "moving-v2")
            require(artifact("keep", "keep") == retained, "unrelated completed artifact changed")
            record_transition("one-root-edit", offset, replacement=artifact("moving-v2", "moving-v2"))

            offset = len(log())
            atomic_write(added, derivation("added", "added"))
            select(["keep", "moving", "added"])
            settle_after(offset, 3)
            addition_events = log()[offset:]
            require(addition_events.count("  added ") == 1 and addition_events.count("  retained ") == 2,
                    "added-root edit did not retain exactly the previous roots")
            artifact("added", "added")
            record_transition("added-root", offset, added=artifact("added", "added"))

            offset = len(log())
            atomic_write(moving, "{ invalid =\n")
            wait_until(lambda: "edit rejected:" in log()[offset:], "invalid import did not report rejection")
            require("  retracted " not in log()[offset:], "invalid import retracted admitted goals")
            require(artifact("keep", "keep") == retained, "invalid import damaged retained output")
            record_transition("invalid-source", offset, retained=retained)
            offset = len(log())
            atomic_write(moving, derivation("moving-v2", "moving-v2"))
            settle_after(offset, 3)

            # Stop only the coordinator after all work settled. These edits are
            # deliberately in one observation window, not a timing-based guess.
            offset = len(log())
            process.send_signal(signal.SIGSTOP)
            try:
                wait_until(lambda: Path(f"/proc/{process.pid}/stat").read_text().rsplit(") ", 1)[1].split()[0] == "T",
                           "coordinator did not acknowledge SIGSTOP")
                for revision in range(1, 21):
                    atomic_write(moving, derivation(f"rapid-{revision}", f"rapid-{revision}"))
            finally:
                process.send_signal(signal.SIGCONT)
            settle_after(offset, 3)
            artifact("rapid-20", "rapid-20")
            for revision in range(1, 20):
                require(not list(store.glob(f"*-{prefix}-rapid-{revision}")), "unobserved intermediate edit produced output")
            require(log()[offset:].count("  added ") == 1, "coalesced edit admitted duplicate replacement goals")
            record_transition("coalescing", offset, edits=20, admitted_replacements=1,
                              final=artifact("rapid-20", "rapid-20"))

            offset = len(log())
            atomic_write(slow, derivation("slow", "slow-started", slow=True))
            select(["keep", "moving", "added", "slow"])
            owned = wait_until(
                lambda: (current if any(row["command"][:3] == ["/bin/busybox", "sleep", "20"]
                                        for row in current.values()) else None)
                if (current := descendants(process.pid)) else None,
                "slow payload never reached its actual sleep child")
            (scratch / "owned-slow-processes.json").write_text(json.dumps(owned, indent=2) + "\n")
            select(["keep", "moving", "added"])
            settle_after(offset, 3)
            wait_until(lambda: all(stopped(pid, identity) for pid, identity in owned.items()),
                       "owned sandbox descendants remain alive after retraction")
            time.sleep(21)
            require(not list(store.glob(f"*-{prefix}-slow")), "cancelled root produced a physical output")
            run_ids = set(re.findall(r"watch run ([^ ]+) (?:generation|edit)", log()))
            require(len(run_ids) == 1, f"watch reports lack one stable run identity: {run_ids}")
            result["run_identity"] = next(iter(run_ids))
            require(f"-{prefix}-slow\n" not in log(), "cancelled root was reported as a successful output")
            require(artifact("keep", "keep") == retained, "retained artifact changed during cancellation")
            record_transition("running-root-deletion", offset, owned_processes=owned,
                              descendants_stopped=True, late_write_observation_seconds=21,
                              cancelled_output_present=False)
            result.update(status="passed", retained=retained, owned_slow_processes=owned,
                          assertions=["one-root edit", "added root", "invalid imported source retention",
                                      "coalesced replacements", "running-root deletion", "descendant teardown", "no late output"])
        except Exception as error:
            result["error"] = str(error)
            raise
        finally:
            if process.poll() is None:
                process.send_signal(signal.SIGCONT)
                process.send_signal(signal.SIGTERM)
                try:
                    process.wait(timeout=40)
                except subprocess.TimeoutExpired:
                    result.update(status="failed", pending_teardown_pid=process.pid)
                    result["error"] = "watch still owns pending teardown after SIGTERM; not force-killed"
            result["exit_code"] = process.poll()
            result["transitions"] = transitions
            result["binary"] = {"path": str(binary), "blake3": subprocess.check_output(
                ["b3sum", "--no-names", str(binary)], text=True).strip()}
            result["driver_blake3"] = subprocess.check_output(
                ["b3sum", "--no-names", str(Path(__file__).resolve())], text=True).strip()
            if process.returncode not in (None, 0):
                result.update(status="failed", error=f"watch shutdown exited {process.returncode}")
            (scratch / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    require(result["status"] == "passed", result.get("error", "watch rail failed"))
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
