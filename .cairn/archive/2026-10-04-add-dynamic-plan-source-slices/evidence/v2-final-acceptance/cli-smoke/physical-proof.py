import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
a, b = (json.loads((root / f'{name}.stdout').read_text()) for name in ('one', 'two'))
def outcome(report, label):
    return next(item for item in report['outcomes'] if item['label'] == label)
def output(item, name):
    return next(out for out in item['outputs'] if out['name'] == name)
x = outcome(a, 'v2-cli-producer-one')
y = outcome(b, 'v2-cli-producer-two')
source_one, source_two = (Path(output(item, 'sources')['path']) for item in (x, y))
assert (source_one / 'outside.txt').read_bytes() == b'outside-one'
assert (source_two / 'outside.txt').read_bytes() == b'outside-two'
assert (source_one / 'package/file.txt').read_bytes() == (source_two / 'package/file.txt').read_bytes() == b'slice content stays the same'
assert Path(output(x, 'out')['path']).read_bytes() == b'producer-one'
assert Path(output(y, 'out')['path']).read_bytes() == b'producer-two'
unit_one, unit_two = (outcome(report, 'v2-cli-unit.drv') for report in (a, b))
assert unit_one['cached'] is False and unit_two['cached'] is True
assert output(unit_one, 'out')['path'] == output(unit_two, 'out')['path']
assert Path(output(unit_two, 'out')['path']).read_text() == a['native_dynamic_plans'][0]['source_slices'][0]['admitted_store_path']
print('PASS: producer outside bytes differ, slice subtree bytes match, second derived unit reads admitted logical slice path')
