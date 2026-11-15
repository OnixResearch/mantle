#!/usr/bin/env python3
"""Run expansion-edge-audit with the exact frozen used_inputs field set."""
import runpy

AUDIT = '/tmp/luna-e9a58881-expansion-edges-20261115/expansion-edge-audit.py'
ns = runpy.run_path(AUDIT, run_name='luna_expansion_audit')
original = ns['used_refs']
def exact_used_refs(step):
    # The frozen RecipeDependencyIndex::used_inputs destructures Exec fields
    # without command_state; remove it before applying the inert port.
    if step['operation']['kind'] != 'exec' or 'command_state' not in step['operation']:
        return original(step)
    cloned = dict(step)
    operation = dict(step['operation'])
    operation.pop('command_state')
    cloned['operation'] = operation
    return original(cloned)
ns['used_refs'] = exact_used_refs
ns['main']()
