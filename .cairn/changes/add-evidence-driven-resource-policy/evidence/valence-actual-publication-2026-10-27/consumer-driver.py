#!/usr/bin/env python3
"""Consume the public content identity retained by the actual successful gateway."""
import json
from pathlib import Path
import subprocess
import sys

run = Path(sys.argv[1]).resolve()
repo = run / 'worktree'
checker = Path(sys.argv[2]).resolve()
b3sum = Path(sys.argv[3]).resolve()
runtime = Path(sys.argv[4]).resolve()
out = Path(sys.argv[5]).resolve()
final = json.loads((runtime / 'final-coordinator.json').read_text())
attempts = list(final['gateway_attempts'].values())
assert len(attempts) == 1, 'expected the one actual successful gateway request'
leaf = attempts[0]['published_evidence']
assert isinstance(leaf, str) and len(leaf) == 69 and leaf.endswith('.json')
expected = leaf[:-5]
assert all(character in '0123456789abcdef' for character in expected)
publication = runtime / 'evidence' / leaf
argv = [sys.executable, str(repo / 'scripts/check-resource-evidence.py'),
        '--checker', str(checker), '--publication', str(publication),
        '--expected-blake3', expected, '--b3sum', str(b3sum), '--output', str(out)]
print(json.dumps({'source': str(runtime / 'final-coordinator.json'),
                  'publicly_retained_leaf': leaf, 'argv': argv,
                  'scope': 'actual signed process-only observation; no invented OOM, completeness or provenance'}), flush=True)
subprocess.run(argv, cwd=repo, check=True)
receipt = json.loads((out / 'receipt.json').read_text())
assert receipt['publication_blake3'] == expected
assert [item['exit'] for item in receipt['commands']] == [0, 2, 1, 1]
print('actual gateway Valence positive/digest/nonclaim/role controls PASS', flush=True)
