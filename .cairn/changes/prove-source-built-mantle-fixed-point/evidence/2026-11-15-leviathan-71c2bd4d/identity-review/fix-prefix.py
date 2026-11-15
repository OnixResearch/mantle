from pathlib import Path
p = Path('/home/brittonr/git/OnixResearch/mantle/.cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-71c2bd4d/identity-review/identity-audit.py')
s = p.read_text()
a = """            if not capture:
                require(len(pending_prefix) <= 65536, 'source record identity/metadata prefix exceeded 64KiB')
                pending_prefix.extend(line)
                identity = None
"""
b = """            if not capture:
                # Keep only the bounded record prefix; non-target records may contain
                # enormous one-line embedded content payloads after their identity.
                remaining = 65536 - len(pending_prefix)
                if remaining > 0:
                    pending_prefix.extend(line[:remaining])
                identity = None
"""
if s.count(a) != 1: raise SystemExit(f'expected one profile prefix block, found {s.count(a)}')
p.write_text(s.replace(a, b, 1))
print('bounded profile-record prefix retained')
