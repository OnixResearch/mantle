#!/usr/bin/env python3
"""Retain reconstructed-fixture rust-plan receipts from the original source-owned cohort."""
import json
from pathlib import Path
import shutil
import subprocess
import sys

root = Path(sys.argv[1]).resolve()
binary = Path(sys.argv[2]).resolve()
fixtures = Path('/home/brittonr/scratch/mantle-da00-t11-fixtures')
out = root / 'receipts'
out.mkdir(mode=0o700)
cargo = shutil.which('cargo')
rustc = shutil.which('rustc')
assert cargo and rustc
records = []
for name in ['proc-macro2', 'serde-derive', 'libc', 'native-answer-sys']:
    for no_oracle in [False, True]:
        mode = 'no-cargo-oracle' if no_oracle else 'cargo-oracle'
        argv = ['timeout', '--signal=TERM', '--kill-after=5s', '120', str(binary), '--json',
                '--store', str(root / 'store'), '--state-dir', str(root / 'state'),
                'rust-plan', '--root', str(fixtures / name), '--cargo', cargo, '--rustc', rustc]
        if no_oracle:
            argv.append('--no-cargo-oracle')
        stdout = out / f'{name}-{mode}.json'
        stderr = out / f'{name}-{mode}.stderr'
        with stdout.open('wb') as output, stderr.open('wb') as errors:
            result = subprocess.run(argv, stdout=output, stderr=errors, check=False)
        item = {'fixture': name, 'mode': mode, 'argv': argv, 'exit_code': result.returncode,
                'stdout': str(stdout), 'stderr': str(stderr), 'valid_baseline_receipt': False}
        try:
            receipt = json.loads(stdout.read_text())
            item['schema_version'] = receipt.get('schema_version')
            item['receipt_hash'] = receipt.get('receipt_hash')
            item['cargo_mode'] = receipt.get('cargo_mode')
            item['unit_graph'] = receipt.get('unit_graph')
            item['host_blockers'] = receipt.get('native_host_unit_graph_planning', {}).get('blockers')
            item['valid_baseline_receipt'] = (result.returncode == 0 and receipt.get('schema_version') == 1
                and isinstance(receipt.get('receipt_hash'), str)
                and len(receipt['receipt_hash']) == 64
                and receipt.get('cargo_mode', {}).get('no_cargo_oracle') == no_oracle)
        except (ValueError, KeyError, TypeError) as error:
            item['parse_error'] = str(error)
        records.append(item)
        (root / 'receipts-result.json').write_text(json.dumps({'schema': 'mantle-da00-original-source-receipts-v1',
            'records': records, 'boundary': 'reconstructed fixture baseline, not contemporaneous execution or app success'}, indent=2) + '\n')
        print(f'{name} {mode}: actual_exit={result.returncode} valid_baseline_receipt={item["valid_baseline_receipt"]}', flush=True)
assert len(records) == 8 and all(item['valid_baseline_receipt'] for item in records)
print('original source-owned da00 baseline receipts retained', flush=True)
