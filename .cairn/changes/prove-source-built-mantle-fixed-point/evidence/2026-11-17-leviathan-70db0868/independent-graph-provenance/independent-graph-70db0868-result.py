#!/usr/bin/env python3
"""Inert, bounded review collation from inspected current bytes and independent checks."""
import json, subprocess
from pathlib import Path
B3='/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum'
BASE=Path('.cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-17-leviathan-70db0868')
def load(rel):return json.loads((BASE/rel).read_bytes())
def hashfile(path):
 p=subprocess.run((B3,'--no-names',str(path)),capture_output=True,text=True,check=True)
 return p.stdout.strip()
def counts(total,covered=None):
 covered=total if covered is None else covered
 assert 0<=covered<=total,(total,covered)
 return {'total':total,'covered':covered,'uncovered':total-covered}
packet=load('review-packet-aws-db-fixed.json');subject=load('review-subject-aws-db-fixed.json')
collation=load('graph-current-inputs-aws-db-fixed.json');graph=load('graph-results-aws-db-fixed/graph-audit.json');selector=load('graph-results-aws-db-fixed/mantle-selector-audit.json')
template=load('graph-results-aws-db-fixed/controller-template-audit.json');helper=load('graph-results-aws-db-fixed/helper-authority-audit.json')
kind=load('graph-results-aws-db-fixed/source-binding-kind-inventory.json')
selector_negative=load('selector-mutations-aws-db-fixed.json');helper_negative=load('graph-results-aws-db-fixed/helper-authority-negative.json');seed_negative=load('graph-results-aws-db-fixed/output-seed-immutable-source-regressions.json');semantic_negative=load('graph-results-aws-db-fixed/semantic-root-regression.json')
assert packet['required_model']=='underclass/gpt-6-sol'
assert packet['source_code_revision']==collation['source_revision']==subject['frozen_product_revision']=='70db08685c0c5fa5c326bfe32db78576d7e2e182'
assert subject['frozen_complete_source_blake3']=='0e5731f87c8cfe87c9dfa9d75ee8d43f3684f86b76479aa28dfe5b27908721b1'
assert collation['subject_bytes_blake3']==packet['subject']['blake3']==hashfile(BASE/'review-subject-aws-db-fixed.json')
assert collation['source_profile_bytes_blake3']==subject['profile_file_raw_blake3']=='11c2b350fc9944d8bf71f2dea4ceb48b20eea869bf09922b20efa7168a92c534'
assert all(x['result']=='Success' for x in packet['current_actual_tasks'].values()) and set(x['task'] for x in packet['current_actual_tasks'].values())==set(range(1730,1736))
assert graph['bounded_verdict']=='HOLD' and template['bounded_verdict']=='STATIC_TEMPLATE_REVIEW_CLEAR'
assert template['combined_static_contract_check']['raw_graph_word_flags_superseded_by_complete_template_scopes']==graph['coverage']['typed_word_issues']==14037
assert template['combined_static_contract_check']['runtime_invocation_static_bindings']==6167 and template['combined_static_contract_check']['actual_runtime_invocation_bytes_observed']==0
assert not template['coverage']['launch_blockers'] and not template['full_category_tally']['corrected_static_word_issue_categories']
assert helper['totals']['covered']==helper['totals']['total']==1098 and helper['totals']['uncovered']==helper['totals']['invalid']==0
assert selector['bounded_verdict'] is True and kind['non_claim']
assert len(kind['resolved_references'])==819 and all(x['resolved_artifact_kind'] in ('tree','text','archive','executable','object') for x in kind['resolved_references'])
assert all(n['passed'] for n in helper_negative['cases'])
assert selector_negative['all_assertions_passed'] and seed_negative['all_assertions_passed'] and all(c['other_consumer_checks_passed'] and c['failure_count']==(0 if c['canonical_match'] else 1) for c in semantic_negative['cases']) and all(c['rejected'] for c in semantic_negative['selector_cases'])
report_hashes={}
for rel,expected in packet['current_reports'].items():
 # Product plan/source program files are frozen remotely; these are transferred local inspection outputs.
 p=BASE/rel
 if p.is_file():
  actual=hashfile(p)
  assert actual==expected,(rel,actual,expected)
  report_hashes[rel]={'bytes_blake3':actual,'size_bytes':p.stat().st_size}
assert len(report_hashes)>=16
stages={}
for record in graph['six_recipe_reports'].values():
 stage=record['stage_id'];g=record['coverage'];c=template['coverage']['per_stage'][stage];h=helper['stages'][stage]
 stages[stage]={
  'recipe_source_bytes':record['blake3'],'recipe_steps':counts(g['recipe_steps']['total'],g['recipe_steps']['covered']),
  'produced_and_other_artifact_refs':counts(g['artifact_refs']['total'],g['artifact_refs']['covered']),
  'declared_inputs':counts(g['declared_input_refs']['total'],g['declared_input_refs']['covered']),
  'dependency_edges':counts(g['dependency_edges']['total'],g['dependency_edges']['covered']),
  'raw_output_seeds':counts(g['output_seeds']['total'],g['output_seeds']['covered']),
  'guard_status_edges':counts(g['guard_status_edges']['total'],g['guard_status_edges']['covered']),
  'semantic_roots':counts(g['semantic_root_refs']['total'],g['semantic_root_refs']['covered']),
  'raw_static_words':counts(g['static_words']['total'],g['static_words']['covered']),
  'raw_word_issues':record['word_issue_count'],
  'corrected_static_words':counts(c['static_words']['total'],c['static_words']['covered']),
  'controller_specs':counts(c['controller_specs']['total'],c['controller_specs']['covered']),
  'controller_slots':counts(c['slots']['total'],c['slots']['covered']),
  'generated_templates':counts(c['generated_templates']['total'],c['generated_templates']['covered']),
  'generated_output_seeds':counts(c['generated_output_seeds']['total'],c['generated_output_seeds']['covered']),
  'helper_authority_uses':counts(h['total'],h['covered']),
  'runtime_invocation_static_bindings':counts(c['runtime_invocation_bindings']['total'],c['runtime_invocation_bindings']['covered']),
  'runtime_controller_symbolic_payloads':counts(c['runtime_symbolic_payloads']['total'],c['runtime_symbolic_payloads']['covered']),
  'actual_runtime_payload_bytes_observed':c['runtime_invocation_bindings']['actual_runtime_bytes_observed'],
  'unknown_source_binding_kinds':record['unknown_binding_kind_count'],
  'reference_issues':record['reference_issue_count'],
 }
 assert h['uncovered']==h['invalid']==0 and record['reference_issue_count']==record['unknown_binding_kind_count']==0
 assert all(v['uncovered']==0 for v in stages[stage].values() if isinstance(v,dict) and 'uncovered' in v)
assert len(stages)==6 and sum(s['recipe_steps']['total'] for s in stages.values())==125407
assert sum(s['raw_word_issues'] for s in stages.values())==14037
assert sum(s['generated_output_seeds']['total'] for s in stages.values())==9176
assert sum(s['controller_slots']['total'] for s in stages.values())==4588
assert sum(s['controller_specs']['total'] for s in stages.values())==2294
assert sum(s['generated_templates']['total'] for s in stages.values())==25053
assert sum(s['corrected_static_words']['total'] for s in stages.values())==2551864
assert selector['coverage']['primary_sources']['total']==2098
assert selector['coverage']['feature_environment']['total']==2386
assert selector['coverage']['immutable_root_source_digests']['total']==16
immutable=collation['immutable_compiler_source'];assert immutable['all_root_units_match'] and immutable['root_unit_count']==16
assert immutable['value']==selector['immutable_compiler_source_identity']['actual_immutable_compiler_source']['value']
assert collation['full_profile_transport_equal']
programs=packet['program_records'];assert collation['program_records']==programs
consumers=collation['frozen_consumers']
inspector_hashes={
 'graph-audit.py':packet['inspector_bindings']['graph'],
 'controller-template-audit.py':packet['inspector_bindings']['controller']['controller-template-audit.py']['blake3'],
 'helper-authority-audit.py':packet['inspector_bindings']['helper']['blake3'],
 'mantle-selector-audit.py':packet['inspector_bindings']['selector'],
 'seed-rejection-probe':packet['inspector_bindings']['seed_probe']}
assert all(len(x)==64 for x in inspector_hashes.values())
negative_counts={
 'semantic_root_cases':counts(len(semantic_negative['cases']),sum(x['other_consumer_checks_passed'] and x['failure_count']==(0 if x['canonical_match'] else 1) for x in semantic_negative['cases'])),
 'helper_authority_cases':counts(len(helper_negative['cases']),sum(x['passed'] for x in helper_negative['cases'])),
 'selector_cases':counts(len(selector_negative['negative_cases']),sum(x['passed'] for x in selector_negative['negative_cases'])),
 'seed_negative_cases':counts(len(seed_negative['seed_negative_cases']),sum(x['passed'] for x in seed_negative['seed_negative_cases'])),
 'immutable_source_cases':counts(len(seed_negative['immutable_compiler_source']['negative_cases']),sum(x['rejected'] for x in seed_negative['immutable_compiler_source']['negative_cases'])),
 'semantic_selector_cases':counts(len(semantic_negative['selector_cases']),sum(x['rejected'] for x in semantic_negative['selector_cases']))}
# The fixture families differ in schema; preserving the underlying report hashes and raw diagnostics is authoritative.
result={
 'schema':'independent-graph-review-aws-db-fixed-v1',
 'reviewer_model_actual':'underclass/gpt-6-sol',
 'scope':'Current 70db0868 frozen-source/prepared-cohort graph, ControllerSpec template, helper authority, selector/primary-source, and static source-kind contracts; complete six authenticated recipes; read-only inspection. No execution of products or candidate programs.',
 'revision':packet['source_code_revision'],
 'current_actual_inspection_tasks':packet['current_actual_tasks'],
 'inert_checker_attempts_preserved':[{'scope':'initial independent census stdin','outcome':'failed before census','error':'FileNotFoundError for remote __file__ <stdin>','effective_script_blake3':'01fd548f0d70a5bc21d4385e12e4ea1c2a23a5e8fc04e8daf6ceac4141e92a0a','corrected_execution_blake3':'fb77d3cabd1d8bc8c7bd5c0a4bc48daaf6ae783a54bcfe282b8ca49f289b672e'},{'scope':'independent primary source JSON template projection','outcome':'failed before coverage','error':'KeyError arguments; static request has invocation.arguments envelope','initial_source_script_blake3':'0b0d38c168f831537bef2454aef905fb1093695ea79a2d68ee4bbcb6f987d75e','corrected_source_script_blake3':'731937cb2408227c8c96fb1190bd0670af1390a5d56838c98cdf1ad20700b9be'}],
 'subject':{'path':packet['subject']['path'],'bytes_blake3':packet['subject']['blake3'],'corrected_git_database_pointer_only':True,'retained_predecessor_subject_is_rejected':True},
 'complete_source':{'blake3':subject['frozen_complete_source_blake3'],'git_code_archive_blake3':packet['frozen_source']['git_code_archive_blake3'],'frozen_tree':packet['frozen_source']['remote_tree']},
 'profile':{'manifest_blake3':collation['source_profile_manifest_blake3'],'raw_bytes_blake3':collation['source_profile_bytes_blake3'],'size_bytes':subject['profile_file_bytes'],'transport_equal':collation['full_profile_transport_equal'],'complete_record_streams_examined':2,'complete_embedded_pre_lowering_unit_plans_compared':2},
 'program_identities':programs,
 'bounded_acceptance':True,
 'assigned_criteria':counts(6),
 'uncovered_criteria':[],
 'criterion_1_all_six_recipes_and_expansions':{'recipes':counts(6),'independent_recipe_census':{'root_reachable_steps':counts(125407),'cycle_count':0,'all_direct_input_refs':counts(1183783),'Produced_direct_input_refs':counts(1174542),'partition_nodes_native_rust_transition':counts(162),'partition_step_assignments_native_rust_transition':counts(77135),'partition_unassigned_duplicate_or_unknown':0,'publication_index_member':'stagex-provider-publication.json'},'steps':counts(125407),'native_derivation_edges':counts(341),'stagex_lineage_source_edges':counts(132),'stagex_recipe_steps':counts(18+8131),'rust_bootstrap_outputs':counts(16),'per_stage':{k:{'steps':v['recipe_steps'],'artifact_refs':v['produced_and_other_artifact_refs'],'declared_inputs':v['declared_inputs'],'dependency_edges':v['dependency_edges']} for k,v in stages.items()},'independent_check':'Inert full-JSON census across six actual recipe bytes, exact step IDs, owner and partition identities; no bounded issues; stage ordering, producer references, and cycles inspected.'},
 'criterion_2_references_order_and_graph_guards':{'stage_artifact_refs':counts(sum(x['produced_and_other_artifact_refs']['total'] for x in stages.values())),'declared_input_refs':counts(sum(x['declared_inputs']['total'] for x in stages.values())),'dependency_edges':counts(sum(x['dependency_edges']['total'] for x in stages.values())),'guard_status_edges':counts(80),'ObservePhase':counts(80),'executed_skipped_state_alternatives':counts(32),'raw_output_seeds':counts(6832),'generated_template_seeds':counts(9176),'unknown_artifact_binding_kinds':0,'independent_check':'Produced producer/output exactness, same-recipe prior order, root/export binding and executable alternative/guard exit status checked against entire actual recipe data; seeds separately checked against producer kind, bounds and output IDs.'},
 'criterion_3_semantic_products_and_handoff':{'mantle_semantic_product_roots':counts(2486),'mantle_semantic_unit_keys':counts(2290),'mantle_build_script_roots':counts(196),'binary_roots_exact_semantic_product':counts(2),'smoke_help_consumers':counts(2),'smoke_export_exact_producer':counts(2),'native_linux_headers_explicit_bin_make_consumption':counts(1),'native_derivation_edges':counts(341),'rust_source_edges':counts(6),'stagex_lineage_source_edges':counts(132),'rust_bootstrap_outputs':counts(16),'independent_check':'Per-stage 1145 semantic keys and 1243 products, unique mantle binary semantic key, exact smoke/help producer, source-level handoff and native/rust/stagex edges inspected; not a claim that the compiled binary runs.'},
 'criterion_4_typed_words_nested_invocations':{'raw_static_words':counts(1488715),'corrected_scoped_static_words':counts(2551864),'generated_templates':counts(25053),'static_runtime_invocation_bindings':counts(6167),'raw_graph_word_issues_retained':14037,'raw_graph_verdict':'HOLD','corrected_scope_failures':0,'reconciliation':'All 14037 raw flags are unknown Input names in raw graph audit where ControllerSpec slot-local Input names are not in parent recipe scope (294 native + 11499 Rust + 1122 each Mantle). The full generated-template validator rechecks every generated template word using the actual slot-local and parent scopes; corrected failures zero. This does not erase or mark the raw diagnostics as successful, and actual runtime request bytes observed remain zero.','actual_runtime_invocation_bytes_observed':0},
 'criterion_5_controller_construction_and_helper_authority':{'controller_specs':counts(2294),'runtime_generated_symbolic_payloads':counts(4588),'controller_slots':counts(4588),'generated_templates':counts(25053),'generated_template_output_seeds':counts(9176),'helper_authority_uses':counts(1098),'source_kind_resolutions':counts(819),'source_binding_kind_usages':counts(sum(x['source_binding_kinds']['total'] for x in graph['coverage']['stages'].values())),'independent_check':'Generated catalog from actual ControllerSpec declarations independently reproduced and compared for each of four controller-bearing recipes; 2294 specs, 4588 slots, 25053 validation templates, same-slot generated output seeds, exactly one guarded selected invocation per turn; frozen C request producer, Rust dispatch/template consumer and stage controller path read; helper source/member/hash ownership and transport verified.'},
 'criterion_6_primary_source_and_compiler_identity':{'complete_profile_record_streams':counts(2),'independently_verified_single_typed_primary_inputs':counts(2098),'authenticated_pre_lowering_unit_plans':counts(2),'mapped_compile_units':counts(2098),'source_plan_derivations':counts(2094),'primary_source_occurrences':counts(2098),'feature_cfg_occurrences':counts(2098),'feature_environment_occurrences':counts(2386),'package_exclusions':counts(2),'immutable_compiler_root_digests':counts(16),'actual_root_source_digest':immutable['value'],'actual_root_source_file_count':immutable['file_count'],'per_stage_plan_derivation':collation['source_plan_derivations'],'independent_check':'Streamed complete 17.3GB output-profile and 2.37GB native-source manifest, exact selected record and embedded plan bytes vs two original plan members; matched compiler root and all 16 individual unit digests; inspected rust_plan.rs source and 2098 native/primary unit identity bindings. No normalization or predecessor count expectations.'},
 'per_stage_complete_static_counters':stages,
 'negative_fixture_checks':negative_counts,
 'bounded_findings':[],
 'raw_diagnostics_preserved':{'graph_report': 'graph-results-aws-db-fixed/graph-audit.json','graph_report_blake3':report_hashes['graph-results-aws-db-fixed/graph-audit.json']['bytes_blake3'],'raw_word_flags':14037,'controller_template_report':'graph-results-aws-db-fixed/controller-template-audit.json','controller_template_report_blake3':report_hashes['graph-results-aws-db-fixed/controller-template-audit.json']['bytes_blake3'],'corrected_scoped_word_failures':0},
 'runtime_unobserved_obligations':[{'requirement':'Protected runtime generated invocation/request/payload bytes','symbolic_bindings_checked':6167,'actual_runtime_bytes_observed':0,'claim':'NOT ESTABLISHED; outside inert static acceptance; must be checked in authorized protected execution'}],
 'non_claims':['Not proof of candidate/helper/compiler/provider/controller execution, stage success or executable semantics.','Not a cold replay, fixed-point equality, reproducibility, v2 proof, native signed row, release eligibility, or authority to launch.','Inspection of symbolic controller/runtime operands does not observe produced runtime payload bytes.','No frozen source/cohort/pins mutated; no product builds/tests/lints/formatters; prior rejected subject not accepted.'],
 'input_hashes':{'selected_inputs':collation['selected_inputs'],'source_profile_bytes_blake3':collation['source_profile_bytes_blake3'],'source_profile_manifest_blake3':collation['source_profile_manifest_blake3'],'immutable_compiler_source_digest':immutable['value'],'current_report_hashes':report_hashes,'graph_collation_blake3':hashfile(BASE/'graph-current-inputs-aws-db-fixed.json'),'review_contract_blake3':hashfile(BASE/'review-contract.md')},
 'consumer_hashes':consumers,
 'inspector_hashes':inspector_hashes,
 'independent_checker_hashes':{'census_source':'45969892e4a6dafadd9a1c480bf176cec8bab88a923011d7b5e397151b45ce7a','census_reconciled_execution':'fb77d3cabd1d8bc8c7bd5c0a4bc48daaf6ae783a54bcfe282b8ca49f289b672e','controller':'7f63bfbcb7b9a98ea85c712801abebb8cc308cfeba282b8285d916d09bc5dc60','complete_profile_stream':'4e21fcc062523c483a50fda358037b902c5412ba6fe46880b0c0deac53ee18bd','source_primary':'731937cb2408227c8c96fb1190bd0670af1390a5d56838c98cdf1ad20700b9be'},
 'inspection_commands':['ssh leviathan.cymric-daggertooth.ts.net /nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14 - < trusted independent-graph-70db0868-controller.py','python3 independent-graph-70db0868-reconcile.py independent-graph-70db0868-census.py | ssh leviathan.cymric-daggertooth.ts.net /nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14 -','python3 independent-graph-70db0868-profile-fix.py independent-graph-70db0868-profile.py | ssh leviathan.cymric-daggertooth.ts.net /nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14 -','ssh leviathan.cymric-daggertooth.ts.net /nix/store/k130ks1njmmgspbh0k2cr56asb79c9xb-python3-3.14.8-env/bin/python3.14 - < trusted independent-graph-70db0868-source.py','ssh leviathan.cymric-daggertooth.ts.net /nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum <frozen-consumer-paths> <graph-inspector-paths>','ssh leviathan.cymric-daggertooth.ts.net jq -c <actual frozen recipe section selectors> <current frozen JSON recipe>','python3 independent-graph-70db0868-result.py (trusted inert local result collation, writes only this owned JSON)'],
 'inspection_authority_boundary':'Advisory bounded static source/graph/template acceptance only; parent owns collation and launch gates.'
}
# Exact granular criterion counters must remain complete and honest.
assert result['assigned_criteria']==counts(6)
assert all(x['uncovered']==0 for section in result.values() if isinstance(section,dict) for x in section.values() if isinstance(x,dict) and {'total','covered','uncovered'}<=x.keys())
out=BASE/'independent-graph-review-aws-db-fixed.json'
out.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
print(json.dumps({'output':str(out),'bytes':out.stat().st_size,'bounded_acceptance':result['bounded_acceptance'],'criteria':result['assigned_criteria'],'negative_fixture_checks':negative_counts},separators=(',',':')))
