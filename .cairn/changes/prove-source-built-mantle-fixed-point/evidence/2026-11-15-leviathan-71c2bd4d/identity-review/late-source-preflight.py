import json
import re
import runpy
import sys
from pathlib import Path

scope = runpy.run_path(sys.argv[1])
copies = scope['verify_source_copies']()
text = (scope['FROZEN'] / 'src/source_built_mantle_programs_native.rs').read_text()
pipe = re.search(r'const PIPE_SOURCE: &str = r###"(.*?)"###;', text, re.S)
if pipe is None:
    raise RuntimeError('frozen PIPE_SOURCE not found')
report = json.loads(scope['REPORT_PATH'].read_bytes())
stages = [scope['inspect_recipe_stage'](stage, pipe.group(1), report) for stage in (1, 2)]
selected = scope['verify_selected_sources'](stages)
result = {'source_copies': copies, 'stages': stages, 'selected_sources': selected,
          'scope': 'Late inert helper/source/script bindings only; not a complete identity verdict or protected execution.'}
Path(sys.argv[2]).write_text(json.dumps(result, indent=2, sort_keys=True) + '\n')
print(json.dumps({'frozen_copies': len(copies['frozen_helper_translator_copies']),
                  'rust_authority_copies': len(copies['rust_authority_copies']),
                  'aws_scripts': selected['aws_selected_count'], 'ring_scripts': selected['ring_selected_count']}))
