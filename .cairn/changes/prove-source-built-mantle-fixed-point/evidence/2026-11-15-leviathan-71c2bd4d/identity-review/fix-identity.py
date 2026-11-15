from pathlib import Path
p = Path('/home/brittonr/git/OnixResearch/mantle/.cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-71c2bd4d/identity-review/identity-audit.py')
s = p.read_text()
a = "for member in ('source-built/rust/program.json', 'cairn/rust/compiler-environment.rs')\n        authored ="
b = "for member in ('source-built/rust/program.json', 'cairn/rust/compiler-environment.rs'):\n        authored ="
if s.count(a) != 1: raise SystemExit(f'expected one Rust copy loop, found {s.count(a)}')
p.write_text(s.replace(a, b, 1))
print('repaired Rust copy loop syntax')
