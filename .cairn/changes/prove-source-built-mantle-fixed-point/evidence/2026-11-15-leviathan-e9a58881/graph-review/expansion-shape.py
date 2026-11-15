#!/usr/bin/env python3
"""Read-only shape/identity inventory for the frozen e9a58881 expansion inputs."""
import hashlib
import json
from pathlib import Path

ROOT = Path('/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-e9a58881-typed-index')
PATHS = {
    'cold-route': ROOT / 'inputs/mantle-source/bootstrap/source-built-cold-route.json',
    'stagex-lineage': ROOT / 'inputs/stagex-lineage/stagex-transition-lineage.json',
    'stagex-transition': ROOT / 'program-sources/source-built-stagex-programs/stagex-transition.json',
    'stagex-transition-index': ROOT / 'program-sources/source-built-stagex-programs/stagex-transition.json.expansion.json',
    'stagex-publication': ROOT / 'program-sources/source-built-stagex-programs/stagex-provider-publication.json',
    'stagex-publication-index': ROOT / 'program-sources/source-built-stagex-programs/stagex-provider-publication.json.expansion.json',
    'native-recipe': ROOT / 'program-sources/fixed-point-native-source-program/source-built/native/program.json',
    'native-index': ROOT / 'program-sources/fixed-point-native-source-program/source-built/native/program.expansion.json',
    'rust-recipe': ROOT / 'inputs/rust-source-archives/source-built/rust/program.json',
    'rust-index': ROOT / 'inputs/rust-source-archives/source-built/rust/program.json.expansion.json',
}

def summary(name, path):
    raw = path.read_bytes()
    result = {'name': name, 'path': str(path), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
    try:
        data = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        result['json'] = f'not parsed: {type(error).__name__}'
        print(json.dumps(result, sort_keys=True))
        return
    result['top_type'] = type(data).__name__
    if isinstance(data, dict):
        result['keys'] = sorted(data)
        for key, value in data.items():
            if isinstance(value, (dict, list)):
                result[f'{key}_count'] = len(value)
        for key in ('schema', 'recipe_member', 'recipe_path', 'route_path', 'route_member', 'roots'):
            if key in data:
                value = data[key]
                result[key] = value if not isinstance(value, list) else value[:10]
        for key in ('stages', 'tools', 'sources', 'patches', 'archives', 'derivations', 'generated_artifacts', 'transition_tools', 'stage_graph', 'provider_outputs'):
            value = data.get(key)
            if isinstance(value, dict) and value:
                result[f'{key}_sample_key'] = next(iter(value))
                result[f'{key}_sample'] = value[next(iter(value))]
            elif isinstance(value, list) and value:
                result[f'{key}_sample'] = value[0]
    print(json.dumps(result, sort_keys=True, default=str))

for name, path in PATHS.items():
    summary(name, path)

# The current Rust expansion index names its authenticated route source; report
# only the path/value here and never evaluate the route expression.
rust_index = json.loads(PATHS['rust-index'].read_bytes())
route_path = rust_index['route_path']
route = ROOT / 'inputs/rust-source-archives' / route_path
summary('rust-route-source', route)
