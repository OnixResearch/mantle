#!/usr/bin/env python3
"""Read-only static ControllerSpec/template audit for an explicitly supplied frozen cohort."""
import collections, json, re, runpy, sys
from pathlib import Path

HERE=Path(__file__).resolve().parent
G=runpy.run_path(str(HERE/'graph-audit.py'),run_name='graph_audit')
COHORT,SOURCE=G['COHORT'],G['SOURCE']

def rust_integer_constant(path,name):
    text=path.read_text()
    match=re.search(r'\bconst\s+'+re.escape(name)+r'\s*:\s*u(?:32|64|size)\s*=\s*(\d[\d_]*)\s*;',text)
    if not match: raise ValueError(f'cannot statically resolve Rust resource bound {name} in {path}')
    return int(match.group(1).replace('_',''))
MAX_CONTROLLER_COMMANDS=rust_integer_constant(SOURCE/'src/source_built_fixed_point.rs','SOURCE_BUILT_FIXED_POINT_EXEC_EVENT_COUNT_MAX')
MAX_RECIPE_STEPS=rust_integer_constant(SOURCE/'src/source_built_recipe.rs','STEP_COUNT_MAX')
def env_ok(s): return bool(s) and all(c.isascii() and (c.isalnum() or c=='_') for c in s)
def word(kind,**x): return {'parts':[{'kind':kind,**x}]}
def valid_exit_policy(p):
    if not isinstance(p,dict): return False
    kind=p.get('kind')
    if kind=='normal': return True
    if kind=='exact': return type(p.get('code')) is int and 0<=p['code']<=255
    if kind=='one-of':
        codes=p.get('codes')
        return isinstance(codes,list) and 2<=len(codes)<=8 and all(type(x) is int and 0<=x<=255 for x in codes) and all(a<b for a,b in zip(codes,codes[1:]))
    return False
def lit(s): return word('literal',text=s)
def inp(n,p=''): return word('input',name=n,path=p)
def outw(n,p=''): return word('output',name=n,path=p)
def ref(stage,step,out): return {'kind':'produced','action_id':f'{stage}/{step}','output_id':out}
def out(kind,size): return {'kind':kind,'bytes_max':size}
def streams(size): return {'stdout':out('object',size),'stderr':out('text',size),'exit-status':out('text',4)}
def exe_refs(e):
    k,v=e['kind'],e.get('value')
    if k=='fixed': return [v]
    if k=='adapted': return [v['executable'],v['original_image']]
    if k=='inventory': return [x[1] for x in v]
    if k=='request-output': return []
    return []
def catalog_refs(s):
    r={G['key'](s['controller'])}; r.update(G['key'](x) for x in s.get('inputs',{}).values())
    for slot in s.get('slots',[]):
        r.update(G['key'](x) for x in slot.get('inputs',{}).values())
        r.update(G['key'](x) for x in exe_refs(slot['executable']))
    return r

def runtime_bindings(s,issue):
    input_maps=[s.get('inputs',{})]+[x.get('inputs',{}) for x in s.get('slots',[])]
    mount_maps=[(s.get('inputs',{}),s.get('controller_runtime_mounts',{}))]
    mount_maps += [(x.get('inputs',{}),x.get('runtime_mounts',{})) for x in s.get('slots',[])]
    inputs={}; mounts={}
    for m in input_maps:
        for n,r in sorted(m.items()):
            if n in inputs and inputs[n]!=r: issue('controller input-name collision with different producer',spec=s['id'],name=n)
            else: inputs[n]=r
    for m,mm in mount_maps:
        for dest,n in sorted(mm.items()):
            r=m.get(n)
            if r is None: issue('runtime mount names undeclared local input',spec=s['id'],mount=dest,input=n); continue
            if dest in mounts:
                if inputs.get(mounts[dest])!=r: issue('conflicting runtime mount producer',spec=s['id'],mount=dest)
            else:
                alias=f'controller-runtime-{len(inputs)}'; inputs[alias]=r; mounts[dest]=alias
    return inputs,mounts

def audit_controller_templates(stage,recipe,owner,prior,stage_idx,word_result,ref_issues,allowed_roles):
    issues=[]; word_issues=[]; missing=[]; invalid_names=[]; payloads=[]; catalogs=[]
    temp_owner={}; ids=set(); per_spec_ids=collections.defaultdict(list); word_stats=collections.Counter(); template_guard_stats=collections.Counter()
    stepmap={s['id']:s for s in recipe['steps']}
    template_bindings=collections.ChainMap(temp_owner,owner)
    def issue(error,**kw): issues.append({'error':error,**kw})
    # Role authority and raw Exec environment names use exact current plan inputs/consumer predicate.
    role_issues=[]
    for st in recipe['steps']:
        for n,r in st.get('inputs',{}).items():
            if r.get('kind') in ('source','manifest-source') and r.get('role') not in allowed_roles:
                role_issues.append({'error':'source role not declared to stage','stage':stage,'step':st['id'],'input':n,'ref':r})
        op=st['operation']
        if op.get('kind')=='exec':
            for n in op.get('environment',{}):
                if not env_ok(n):
                    invalid_names.append({'origin':'RecipeOperation.environment','stage':stage,'step':st['id'],'environment_name':n})
                    word_issues.append({'where':f'{stage}/{st["id"]}.env.{n}','error':'invalid environment name'})
    # Static request values are available only from exact local WriteText producers.
    for consumer,request,input_name in word_result['invocations']:
        r=consumer.get('inputs',{}).get(input_name,{})
        aid=r.get('action_id',''); local=None
        if re.fullmatch(re.escape(stage)+r'/[A-Za-z0-9_.:-]+',aid): local=aid[len(stage)+1:]
        producer=stepmap.get(local) if local else None
        for name,value in request.get('environment',{}).items():
            if not env_ok(name):
                invalid_names.append({'origin':'static RecipeInvocation','stage':stage,'consumer_step':consumer['id'],
                    'request_input':input_name,'request_producer_action_id':aid,
                    'request_producer_step':producer['id'] if producer else None,
                    'environment_name':name,'value_word':value})
    def add_template(spec,tid,inputs,outputs,args=(),env=None,mounts=None,guard=None,dynamic=None,command_state=None,seeds=None):
        env,mounts=env or {},mounts or {}; guard_names={x['status_input'] for x in (guard or {}).get('conditions',[])}
        if tid in ids: issue('duplicate generated template step ID',stage=stage,spec=spec['id'],template=tid)
        ids.add(tid); per_spec_ids[spec['id']].append(tid)
        aid=f'controller-validation/{tid}'
        # Validate every exact symbolic input producer/output before lowering its local identity.
        for input_name,input_ref in inputs.items():
            if input_ref.get('kind')=='produced':
                producer_id=input_ref.get('action_id',''); match=re.fullmatch(r'([A-Za-z0-9_.:-]+)/([A-Za-z0-9_.:-]+)',producer_id)
                if not match:
                    issue('template input has malformed full producer namespace',stage=stage,spec=spec['id'],template=tid,input=input_name,ref=input_ref); continue
                namespace=match.group(1); producer=None
                if namespace=='controller-validation': producer=temp_owner.get(producer_id)
                elif namespace==stage: producer=owner.get(producer_id)
                else:
                    for prior_stage,prior_owner in prior.items():
                        if producer_id in prior_owner: producer=prior_owner[producer_id]; break
                if producer is None:
                    issue('template input has unknown exact producer',stage=stage,spec=spec['id'],template=tid,input=input_name,ref=input_ref)
                elif input_ref.get('output_id') not in producer.get('outputs',{}):
                    issue('template input names unknown output on exact producer',stage=stage,spec=spec['id'],template=tid,input=input_name,ref=input_ref)
                else:
                    binding_kind=producer['outputs'][input_ref['output_id']].get('kind')
                    if binding_kind is None: missing.append({'where':f'{stage}/{tid}.{input_name}','part':input_ref,'reason':'exact producer output kind unavailable'})
            elif input_ref.get('kind') not in ('source','manifest-source'):
                issue('template input has unknown artifact reference kind',stage=stage,spec=spec['id'],template=tid,input=input_name,ref=input_ref)
        temp_owner[aid]={'local':tid,'outputs':outputs}
        # Frozen lower_step requires each generated Exec to name an exact Produced Executable.
        er=inputs.get('executable'); ek=G['kind_for'](er or {},template_bindings,prior,stage_idx)
        if er is None or er.get('kind')!='produced': issue('generated Exec lacks exact Produced executable',stage=stage,spec=spec['id'],template=tid,ref=er)
        elif ek is None: missing.append({'where':f'{stage}/{tid}.executable','part':er,'reason':'executable binding kind unavailable'})
        elif ek!='executable': issue('generated Exec executable is not ArtifactKind::Executable',stage=stage,spec=spec['id'],template=tid,kind=ek,ref=er)
        if dynamic is not None:
            rr=inputs.get(dynamic); rk=G['kind_for'](rr or {},template_bindings,prior,stage_idx)
            if rr is None or rr.get('kind')!='produced' or rr.get('output_id')=='exit-status': issue('dynamic invocation is not ordinary Produced input',stage=stage,spec=spec['id'],template=tid,ref=rr)
            elif rk is None: missing.append({'where':f'{stage}/{tid}.invocation','part':rr,'reason':'dynamic invocation kind unavailable'})
            elif rk!='text': issue('dynamic invocation input is not ArtifactKind::Text',stage=stage,spec=spec['id'],template=tid,kind=rk,ref=rr)
        for dest,n in mounts.items():
            if n not in inputs: issue('Runtime mount names absent template-local input',stage=stage,spec=spec['id'],template=tid,mount=dest,input=n)
        for on,n in (seeds or {}).items():
            target=outputs.get(on); sr=inputs.get(n); sk=G['kind_for'](sr or {},template_bindings,prior,stage_idx)
            if target is None or sr is None: issue('output seed lacks exact output/input binding',stage=stage,spec=spec['id'],template=tid,output=on,input=n)
            elif target.get('kind')!='tree' or sk not in (None,'tree'): issue('output seed is not Tree-to-Tree',stage=stage,spec=spec['id'],template=tid,output=on,input=n,target_kind=target.get('kind'),seed_kind=sk)
            elif sk is None: missing.append({'where':f'{stage}/{tid}.seed.{n}','part':sr,'reason':'output seed binding kind unavailable'})
        for n in env:
            if not env_ok(n):
                invalid_names.append({'origin':'ControllerSpec.environment template use','stage':stage,'controller_spec':spec['id'],'template_step':tid,'environment_name':n})
                word_issues.append({'where':f'{stage}/{tid}.env.{n}','error':'invalid environment name'})
        if guard and guard.get('conditions'):
            template_guard_stats['guarded_steps']+=1
            template_guard_stats['conditions']+=len(guard['conditions'])
        synthetic={'id':tid,'inputs':inputs,'outputs':outputs,'operation':{'kind':'exec','arguments':list(args),'environment':env,'runtime_mounts':mounts,'guard':guard,'invocation_input':dynamic}}
        for v in list(args)+list(env.values()):
            word_stats['words']+=1; word_stats['parts']+=len(v.get('parts',[]))
            for p in v.get('parts',[]): word_stats['part:'+p.get('kind','?')]+=1
            ctx={'where':f'{stage}/{tid}','guard_names':guard_names}
            errs=G['static_word_errors'](v,ctx,synthetic,inputs,outputs,mounts,None,template_bindings,prior,stage_idx,missing)
            for e in errs: word_issues.append({'where':ctx['where'],'error':e,'word':v})
        return {n:{'kind':'produced','action_id':aid,'output_id':n} for n in outputs}

    # Exclude ControllerSpec words mis-scoped to generic outer controller-input-N aliases.
    controller_ids={s['id'] for s in recipe['steps'] if s['operation']['kind']=='controller'}
    base_issues=[x for x in word_result['issues'] if not any(x.get('where','').startswith(f'{stage}/{i}.controller.') for i in controller_ids)]
    base_unknown=[x for x in word_result['unknown_binding_kinds'] if not any(x.get('where','').startswith(f'{stage}/{i}.controller.') for i in controller_ids)]
    raw_spec_words=raw_spec_parts=0
    for outer in recipe['steps']:
        if outer['operation']['kind']!='controller': continue
        s=outer['operation']['specification']; sid=s['id']; slots=s.get('slots',[])
        raw_spec_words+=len(s.get('start_arguments',[]))+len(s.get('environment',{}))
        raw_spec_parts+=sum(len(x.get('parts',[])) for x in s.get('start_arguments',[]))+sum(len(x.get('parts',[])) for x in s.get('environment',{}).values())
        by_id={x['id']:x for x in slots}
        if len(by_id)!=len(slots): issue('duplicate ControllerSlot id',stage=stage,spec=sid)
        if not slots or type(s.get('commands_max')) is not int or s['commands_max']<=0 or type(s.get('file_bytes_max')) is not int or s['file_bytes_max']<4 or type(s.get('tree_bytes_max')) is not int or s['tree_bytes_max']<=0: issue('invalid resource bounds or empty controller catalog',stage=stage,spec=sid)
        if type(s.get('commands_max')) is int and s['commands_max']>MAX_CONTROLLER_COMMANDS: issue('controller commands_max exceeds source consumer bound',stage=stage,spec=sid,actual=s.get('commands_max'),maximum=MAX_CONTROLLER_COMMANDS)
        if not valid_exit_policy(s.get('exit_policy')): issue('invalid ControllerSpec exit policy',stage=stage,spec=sid,exit_policy=s.get('exit_policy'))
        constructed_count=3+sum(4 if x.get('executable',{}).get('kind') in ('inventory','request-output') else 3 for x in slots)
        if constructed_count>MAX_RECIPE_STEPS: issue('generated template count exceeds source recipe step bound',stage=stage,spec=sid,constructed_count=constructed_count,maximum=MAX_RECIPE_STEPS)
        starts=s.get('start_arguments',[]); indexes=s.get('restart_indices',[])
        if len(indexes)!=3 or any(type(i) is not int or i<0 or i>=len(starts) for i in indexes) or len(set(indexes))!=3:
            issue('invalid restart positions',stage=stage,spec=sid,indices=indexes,argument_count=len(starts))
        runtime_inputs,runtime_mounts=runtime_bindings(s,issue)
        expected=catalog_refs(s); gnames={x['status_input'] for x in (outer['operation'].get('guard') or {}).get('conditions',[])}
        actual={G['key'](r) for n,r in outer.get('inputs',{}).items() if n not in gnames}
        aliases={n for n in outer.get('inputs',{}) if n not in gnames}; expected_aliases={f'controller-input-{i}' for i in range(len(expected))}
        if actual!=expected: issue('outer controller refs differ from exact immutable catalog',stage=stage,controller_step=outer['id'],missing=sorted(expected-actual)[:8],extra=sorted(actual-expected)[:8])
        if aliases!=expected_aliases: issue('outer controller generic aliases not exact/contiguous',stage=stage,controller_step=outer['id'],expected=sorted(expected_aliases)[:8],actual=sorted(aliases)[:8])
        f,t=s['file_bytes_max'],s['tree_bytes_max']
        expected_out={'checkpoint':out('tree',t),'workspace':out('tree',t),'times':out('text',f),'generated':out('tree',t),'stdout':out('object',f),'stderr':out('text',f),'exit-status':out('text',4)}
        if outer.get('outputs')!=expected_out: issue('outer controller outputs differ from spec contract',stage=stage,controller_step=outer['id'])

        start_id=f'{sid}-start'; start_inputs=dict(s.get('inputs',{}))
        for n,r in runtime_inputs.items():
            if n in start_inputs and start_inputs[n]!=r: issue('start runtime binding collision',stage=stage,spec=sid,name=n)
            else: start_inputs[n]=r
        start_inputs['executable']=s['controller']
        start_args=[lit('--mantle-controller-start'),outw('checkpoint'),lit('--')]+starts
        start_out={'checkpoint':out('tree',t)}; start_out.update(streams(f))
        started=add_template(s,start_id,start_inputs,start_out,start_args,s.get('environment',{}),runtime_mounts)
        checkpoint=started['checkpoint']; observations=[]; slot_report=[]
        for slot in slots:
            name=slot.get('id','<missing>'); ex=slot.get('executable',{}); kind=ex.get('kind') if isinstance(ex,dict) else None
            if kind not in ('fixed','adapted','inventory','request-output'):
                issue('unknown ControllerExecutable variant is uncovered; no template shape inferred',stage=stage,spec=sid,slot=name,executable_variant=kind)
                slot_report.append({'slot':name,'executable_kind':kind,'coverage':'uncovered unknown variant','validation_template_ids':[]})
                continue
            base=f'{sid}-000000-{name}'; value=ex.get('value')
            if not valid_exit_policy(slot.get('exit_policy')): issue('invalid ControllerSlot exit policy',stage=stage,spec=sid,slot=name,exit_policy=slot.get('exit_policy'))
            if set(slot.get('inputs',{})) & set(s.get('inputs',{})): issue('source/slot input-name collision',stage=stage,spec=sid,slot=name)
            reserved={'executable','invocation','workspace-seed','descriptor-files-seed','stdin','times','data','prepare-status','copy-status'}
            if set(slot.get('inputs',{}))&reserved: issue('slot uses reserved input name',stage=stage,spec=sid,slot=name,names=sorted(set(slot.get('inputs',{}))&reserved))
            if kind=='request-output':
                producers=value.get('producer_slots',[]) if isinstance(value,dict) else []
                if not producers or any(p not in by_id or not by_id[p].get('generated_outputs') for p in producers): issue('RequestOutput names non-generated/missing producer slot',stage=stage,spec=sid,slot=name,producer_slots=producers)
            if kind=='inventory' and not value: issue('empty executable inventory',stage=stage,spec=sid,slot=name)
            prep_id=base+'-prepare'; pi=dict(s.get('inputs',{}))
            for n,r in slot.get('inputs',{}).items():
                if n in pi: issue('prepare input collision',stage=stage,spec=sid,slot=name,input=n)
                else: pi[n]=r
            pi['checkpoint']=checkpoint
            args=[lit('--mantle-controller-prepare'),inp('checkpoint'),lit(name),outw('invocation'),outw('workspace'),outw('stdin'),outw('times'),outw('data'),outw('descriptor-files')]
            for n in sorted(s.get('inputs',{})): args += [lit(n),inp(n)]
            args.append(lit('--command-inputs'))
            for n in sorted(slot.get('inputs',{})): args += [lit(n),inp(n)]
            rnames=sorted(set(slot.get('runtime_mounts',{}).values())); args += [lit('--runtime-inputs'),lit(str(len(rnames)))]+[lit(n) for n in rnames]+[lit('--')]
            if kind in ('fixed','adapted'):
                image=value if kind=='fixed' else value['original_image']; pi['selector-image']=image
                args += [lit('fixed-image'),lit(name),inp('selector-image')]
            elif kind=='inventory':
                args += [lit('inventory'),lit(str(len(value or [])))]
                for j,pair in enumerate(value or []): pi[f'image-{j}']=pair[1]; args += [lit(pair[0]),inp(f'image-{j}')]
            elif kind=='request-output':
                ps=value.get('producer_slots',[]); args += [lit('generated'),lit(str(len(ps)))]+[lit(x) for x in ps]
            for n,r in runtime_inputs.items():
                if n in pi and pi[n]!=r: issue('prepare runtime binding collision',stage=stage,spec=sid,slot=name,input=n)
                else: pi[n]=r
            pi['executable']=s['controller']
            po={'invocation':out('text',f),'workspace':out('tree',t),'stdin':out('object',f),'times':out('text',f),'data':out('text',f),'descriptor-files':out('tree',t)}; po.update(streams(f))
            prepared=add_template(s,prep_id,pi,po,args,s.get('environment',{}),runtime_mounts)
            ci=dict(slot.get('inputs',{})); guard=[{'status_input':'prepare-status','accepted_codes':[0]}]
            ci.update({'prepare-status':prepared['exit-status'],'invocation':prepared['invocation'],'workspace-seed':prepared['workspace'],'descriptor-files-seed':prepared['descriptor-files'],'stdin':prepared['stdin'],'times':prepared['times'],'data':prepared['data']})
            for n,r in runtime_inputs.items():
                if n in ci and ci[n]!=r: issue('command runtime binding collision',stage=stage,spec=sid,slot=name,input=n)
                else: ci[n]=r
            if kind in ('fixed','adapted'): executable=value if kind=='fixed' else value['executable']
            else:
                copy_id=base+'-executable'; qi={'checkpoint':checkpoint,'prepare-status':prepared['exit-status'],'executable':s['controller']}
                qa=[lit('--mantle-controller-executable'),inp('checkpoint'),lit(name),outw('program')]
                if kind=='inventory':
                    qa += [lit('inventory'),lit(str(len(value or [])))]
                    for j,pair in enumerate(value or []): qi[f'image-{j}']=pair[1]; qa += [lit(pair[0]),inp(f'image-{j}')]
                elif kind=='request-output':
                    ps=value.get('producer_slots',[]); qa += [lit('generated'),lit(str(len(ps)))]+[lit(x) for x in ps]
                for n,r in runtime_inputs.items():
                    if n in qi and qi[n]!=r: issue('executable-copy runtime collision',stage=stage,spec=sid,slot=name,input=n)
                    else: qi[n]=r
                qo={'program':out('executable',f)}; qo.update(streams(f))
                copied=add_template(s,copy_id,qi,qo,qa,{},runtime_mounts,{'conditions':[{'status_input':'prepare-status','accepted_codes':[0]}]})
                ci['copy-status']=copied['exit-status']; guard.append({'status_input':'copy-status','accepted_codes':[0]}); executable=copied['program']
            ci['executable']=executable
            co={'workspace':out('tree',t),'times':out('text',f),'descriptors':out('text',f),'descriptor-files':out('tree',t)}
            if slot.get('generated_outputs'): co['generated']=out('tree',t)
            co.update(streams(f)); cmd=base+'-command'
            state={'workspace_output':'workspace','prior_times_input':'times','times_output':'times','descriptors_output':'descriptors','descriptor_files_output':'descriptor-files'}
            seeds={'workspace':'workspace-seed','descriptor-files':'descriptor-files-seed'}
            add_template(s,cmd,ci,co,[],{},slot.get('runtime_mounts',{}),{'conditions':guard},'invocation',state,seeds)
            if ci.get('invocation')!=prepared['invocation'] or po['invocation']['kind']!='text': issue('dynamic request is not exact same-slot prepared Text output',stage=stage,spec=sid,slot=name)
            for n,r in [('prepare-status',prepared['exit-status']),('workspace-seed',prepared['workspace']),('descriptor-files-seed',prepared['descriptor-files']),('stdin',prepared['stdin']),('times',prepared['times']),('data',prepared['data'])]:
                if ci.get(n)!=r: issue('command slot input differs from exact prepare output',stage=stage,spec=sid,slot=name,input=n)
            symbolic_link_verified=ci.get('invocation')==prepared['invocation'] and po['invocation'].get('kind')=='text' and prepared['invocation'].get('action_id')==f'controller-validation/{prep_id}' and prepared['invocation'].get('output_id')=='invocation'
            payloads.append({'stage':stage,'controller_step':outer['id'],'controller_spec':sid,'slot':name,
                'validation_prepare_step':prep_id,'validation_command_step':cmd,'validation_producer_ref':prepared['invocation'],
                'consumer_input':'invocation','runtime_prepare_action_id_pattern':f'{stage}/{sid}/turn/{{turn}}/{name}/prepare',
                'runtime_command_action_id_pattern':f'{stage}/{sid}/turn/{{turn}}/{name}/command',
                'producer_output_id':'invocation','producer_output_kind':'text','reference_relation':'Current same-slot Prepare -> Command at same turn',
                'symbolic_bindings_verified':symbolic_link_verified,
                'guard_inputs':[ci[x['status_input']] for x in guard],'guard_conditions':guard,'exit_policy':slot.get('exit_policy'),'workspace_seed_ref':prepared['workspace'],
                'descriptor_files_seed_ref':prepared['descriptor-files'],'command_state':state,'output_seeds':seeds,
                'prepare_exit_policy':{'kind':'one-of','codes':[0,1]},'command_output_contract':co,
                'prepare_input_binding_count':len(pi),'prepare_input_name_examples':sorted(pi)[:32],
                'command_input_binding_count':len(ci),'command_input_name_examples':sorted(ci)[:32],
                'runtime_request_bytes_observed':False})
            obs=base+'-observation'; phase=ref('controller-validation',obs,'phase'); phaseout={'phase':out('tree',t)}
            payloads[-1]['observe_phase']={'template_step':obs,'phase_producer_template':cmd,'phase_ref':phase,'state':None,'alternatives':['executed command phase','skipped command phase from guard failure'],'dependency_relation':'ObservePhase observes exact same-slot command phase'}
            if obs in ids: issue('duplicate observation template ID',stage=stage,spec=sid,slot=name)
            ids.add(obs); per_spec_ids[sid].append(obs); temp_owner[f'controller-validation/{obs}']={'local':obs,'outputs':phaseout}
            observations.append((name,bool(slot.get('generated_outputs')),phase))
            slot_report.append({'slot':name,'executable_kind':kind,'generated_outputs':bool(slot.get('generated_outputs')),'exit_policy':slot.get('exit_policy'),'merge_stderr':slot.get('merge_stderr'),'runtime_mounts':slot.get('runtime_mounts',{}),'executable_copy_exit_policy':{'kind':'exact','code':0} if kind in ('inventory','request-output') else None,
                'validation_template_ids':[prep_id,base+'-executable' if kind in ('inventory','request-output') else None,cmd,obs]})
        resume_id=f'{sid}-000000-resume'; ri=dict(s.get('inputs',{})); ri['checkpoint']=checkpoint
        ra=[lit('--mantle-controller-resume'),inp('checkpoint'),outw('checkpoint')]+[lit(str(indexes[i]) if len(indexes)>i else '0') for i in range(3)]+[lit(str(len(observations)))]
        for i,(name,generated,phase) in enumerate(observations): ri[f'phase-{i}']=phase; ra += [lit(name),lit('1' if generated else '0'),inp(f'phase-{i}')]
        ra += [lit('--')]+starts
        for n,r in runtime_inputs.items():
            if n in ri and ri[n]!=r: issue('resume runtime binding collision',stage=stage,spec=sid,name=n)
            else: ri[n]=r
        ri['executable']=s['controller']; ro={'checkpoint':out('tree',t)}; ro.update(streams(f))
        add_template(s,resume_id,ri,ro,ra,s.get('environment',{}),runtime_mounts)
        for i,(_,_,phase) in enumerate(observations):
            if ri.get(f'phase-{i}')!=phase: issue('resume omits exact slot observation',stage=stage,spec=sid,index=i)
        finish_id=f'{sid}-finish'; fi={'checkpoint':checkpoint}
        for n,r in runtime_inputs.items():
            if n in fi and fi[n]!=r: issue('finish runtime binding collision',stage=stage,spec=sid,name=n)
            else: fi[n]=r
        fi['executable']=s['controller']; fa=[lit('--mantle-controller-finish'),inp('checkpoint'),outw('workspace'),outw('times'),outw('generated'),outw('checkpoint')]
        fo={'workspace':out('tree',t),'times':out('text',f),'generated':out('tree',t),'checkpoint':out('tree',t)}; fo.update(streams(f))
        add_template(s,finish_id,fi,fo,fa,{},runtime_mounts)
        expected=3+sum(4 if x['executable']['kind'] in ('inventory','request-output') else 3 for x in slots)
        actual=len(per_spec_ids[sid])
        if expected!=actual: issue('template count differs from frozen constructor formula',stage=stage,spec=sid,expected=expected,actual=actual)
        patterns={'start':f'{stage}/{sid}/start','resume':f'{stage}/{sid}/turn/{{turn}}/_/resume','finish':f'{stage}/{sid}'}
        for slot in slots:
            if slot.get('executable',{}).get('kind') not in ('fixed','adapted','inventory','request-output'): continue
            p=f'{stage}/{sid}/turn/{{turn}}/{slot["id"]}'
            for role in ('prepare','command','observation'): patterns[f'{slot["id"]}:{role}']=p+'/'+role
            if slot['executable']['kind'] in ('inventory','request-output'): patterns[f'{slot["id"]}:executable']=p+'/executable'
        runtime_template_ids=[f'@controller-{sid}-start',f'@controller-{sid}-000000-resume',f'@controller-{sid}-finish']
        for slot in slots:
            if slot.get('executable',{}).get('kind') not in ('fixed','adapted','inventory','request-output'): continue
            base_id=f'@controller-{sid}-000000-{slot["id"]}'
            runtime_template_ids += [base_id+'-prepare',base_id+'-command',base_id+'-observation']
            if slot['executable']['kind'] in ('inventory','request-output'): runtime_template_ids.append(base_id+'-executable')
        catalogs.append({'controller_step':outer['id'],'controller_spec':sid,'commands_max':s.get('commands_max'),'commands_max_consumer_bound':MAX_CONTROLLER_COMMANDS,
                         'file_bytes_max':f,'tree_bytes_max':t,'controller_exit_policy':s.get('exit_policy'),'outer_guard':outer['operation'].get('guard'),'start_exit_policy':{'kind':'one-of','codes':[0,1]},'resume_exit_policy':{'kind':'one-of','codes':[0,1]},
                         'slot_count':len(slots),'validation_template_step_ids':per_spec_ids[sid],
                         'runtime_step_template_ids':runtime_template_ids,'resolved_action_id_patterns':patterns,'slots':slot_report,
                         'restart_indices':indexes,'start_argument_count':len(starts)})

    # The old inspector put ControllerSpec words into controller-input-N scope. Exclude those rows;
    # all their generated uses above are checked in actual ControllerRuntime/template scopes.
    cids={s['id'] for s in recipe['steps'] if s['operation']['kind']=='controller'}
    base_word_issues=[x for x in word_result['issues'] if not any(x.get('where','').startswith(f'{stage}/{i}.controller.') for i in cids)]
    base_unknown=[x for x in word_result['unknown_binding_kinds'] if x.get('category')!='runtime-symbolic-invocation' and not any(x.get('where','').startswith(f'{stage}/{i}.controller.') for i in cids)]
    runtime_invocations=[]
    for row in word_result['unknown_binding_kinds']:
        if row.get('category')!='runtime-symbolic-invocation': continue
        where=row.get('where',''); local=where[len(stage)+1:] if where.startswith(stage+'/') else None
        consumer=stepmap.get(local,{}); op=consumer.get('operation',{}); request_ref=row.get('request_ref',{})
        request_name=op.get('invocation_input'); bound=consumer.get('inputs',{}).get(request_name)
        request_kind=G['kind_for'](request_ref,owner,prior,stage_idx)
        executable=consumer.get('inputs',{}).get(op.get('executable_input'),{})
        executable_kind=G['kind_for'](executable,owner,prior,stage_idx)
        valid=op.get('kind')=='exec' and request_ref.get('kind')=='produced' and request_ref.get('output_id')!='exit-status'
        valid=valid and bound==request_ref and request_kind=='text' and executable.get('kind')=='produced' and executable_kind=='executable'
        runtime_invocations.append({**row,'request_input':request_name,'request_binding_kind':request_kind,
            'consumer_executable_ref':executable,'consumer_executable_kind':executable_kind,
            'static_binding_verified':valid,'actual_runtime_bytes_observed':False,
            'remaining_obligation':'actual emitter bytes, strict RecipeInvocation decode/admission and protected command outcome'})
        if not valid: issue('runtime-generated invocation lacks an exact Produced Text/Executable binding',stage=stage,where=where,request_ref=request_ref,request_kind=request_kind,executable=executable,executable_kind=executable_kind)
    if len(runtime_invocations)!=word_result['dynamic_invocation_uninspectable_count']:
        issue('runtime invocation inventory does not cover every dynamic request',stage=stage,expected=word_result['dynamic_invocation_uninspectable_count'],covered=len(runtime_invocations))
    corrected=base_word_issues+word_issues; counts=collections.Counter(x.get('error','unknown') for x in corrected)
    raw_counts=collections.Counter(x.get('error','unknown') for x in word_result['issues'])
    unk=base_unknown+missing; unk_counts=collections.Counter(x.get('reason','unknown') for x in unk)
    ref_counts=collections.Counter(x.get('error','unknown') for x in ref_issues+role_issues)
    ex_by_cat=collections.defaultdict(list)
    for row in corrected:
        if len(ex_by_cat[row.get('error','unknown')])<20: ex_by_cat[row.get('error','unknown')].append(row)
    role_counts=collections.Counter()
    for templates in per_spec_ids.values():
        for template in templates:
            if template.endswith('-observation'): role_counts['ObservePhase']+=1
            elif template.endswith('-prepare'): role_counts['prepare']+=1
            elif template.endswith('-command'): role_counts['command']+=1
            elif template.endswith('-executable'): role_counts['executable']+=1
            elif template.endswith('-start'): role_counts['start']+=1
            elif template.endswith('-resume'): role_counts['resume']+=1
            elif template.endswith('-finish'): role_counts['finish']+=1
            else: role_counts['unknown-template-role']+=1
    variant_counts=collections.Counter(x.get('executable_kind','unknown') for c in catalogs for x in c['slots'])
    template_count_formula=sum(3+sum(4 if x.get('executable_kind') in ('inventory','request-output') else 3 for x in c['slots'] if x.get('executable_kind') in ('fixed','adapted','inventory','request-output')) for c in catalogs)
    if template_count_formula!=len(ids): issue('generated template role count differs from constructor coverage formula',stage=stage,expected=template_count_formula,actual=len(ids))
    
    return {'stage':stage,'recipe_step_count':len(recipe['steps']),'controller_spec_count':len(catalogs),
        'controller_slot_count':sum(x['slot_count'] for x in catalogs),'generated_template_step_count':len(ids),
        'generated_template_role_counts':dict(role_counts),'slot_executable_variant_counts':dict(variant_counts),
        'generated_template_guarded_step_count':template_guard_stats['guarded_steps'],'generated_template_guard_condition_count':template_guard_stats['conditions'],
        'generated_static_template_word_count':word_stats['words'],'generated_static_template_word_part_count':word_stats['parts'],
        'generated_static_template_word_part_kind_counts':{k[5:]:v for k,v in word_stats.items() if k.startswith('part:')},
        'corrected_total_static_word_count':word_result['word_count']-raw_spec_words+word_stats['words'],
        'corrected_total_static_word_part_count':word_result['word_part_count']-raw_spec_parts+word_stats['parts'],
        'raw_word_issue_count':len(word_result['issues']),'raw_word_issue_categories':dict(raw_counts),
        'corrected_word_issue_count':len(corrected),'corrected_word_issue_categories':dict(counts),
        'corrected_word_issue_examples_by_category':dict(ex_by_cat),'unknown_binding_kind_count':len(unk),
        'unknown_binding_kind_reason_counts':dict(unk_counts),'unknown_binding_kind_examples':unk[:100],
        'reference_issue_count':len(ref_issues)+len(role_issues),'reference_issue_categories':dict(ref_counts),
        'source_role_issue_count':len(role_issues),'source_role_issues':role_issues[:100],
        'controller_constructor_issue_count':len(issues),'controller_constructor_issues':issues[:100],
        'controller_constructor_issue_categories':dict(collections.Counter(x.get('error','unknown') for x in issues)),
        'static_invocation_json_count':word_result['invocation_count'],
        'uninspectable_dynamic_invocation_count':word_result['dynamic_invocation_uninspectable_count'],
        'runtime_invocation_bindings':runtime_invocations,'runtime_invocation_static_binding_verified_count':sum(x['static_binding_verified'] for x in runtime_invocations),
        'runtime_derived_payload_count':len(payloads),'runtime_derived_payload_symbolic_bindings_verified':sum(x['symbolic_bindings_verified'] for x in payloads),
        'runtime_derived_payloads':payloads,'controller_template_catalogs':catalogs,
        'invalid_environment_name_occurrence_count':len(invalid_names),'invalid_environment_name_occurrences':invalid_names[:100],
        'invalid_environment_name_counts':dict(collections.Counter(x['environment_name'] for x in invalid_names))}


def main():
    plan_path=COHORT/'source-program-preparation-input-plan.json'
    subject_path=Path(sys.argv[4])
    plan=G['load'](plan_path)
    graph_path=G['SCRATCH']/'graph-audit.json'
    base=G['load'](graph_path)
    selected=base['selected_input_blake3']
    keys={'stagex-transition':'stagex_transition_recipe','stagex-provider-publication':'publication_recipe','full-source-native-provider':'native_recipe','full-source-rust-provider':'rust_recipe','mantle-stage1':'mantle_stage1','mantle-stage2':'mantle_stage2'}
    if [x.get('stage_id') for x in plan.get('stages',[])]!=G['STAGES']:
        raise ValueError('input plan stage order differs from the graph inspector protocol')
    paths={stage:Path(selected[key]['path']) for stage,key in keys.items()}
    for stage,path in paths.items():
        record=selected[keys[stage]]
        if path.stat().st_size!=record['size_bytes'] or G['b3'](path)!=record['blake3']:
            raise ValueError(f'selected recipe changed after graph inspection: {stage}')
    allow={x['stage_id']:{v['role'] for v in x.get('inputs',[]) if v.get('authority')=='source'} for x in plan['stages']}
    prior={}; reports={}
    for i,ps in enumerate(plan['stages']):
        stage=ps['stage_id']; data=G['load'](paths[stage]); issues=[]
        owner,_=G['make_owner'](data,i,prior,i,issues)
        wr=G['recipe_words'](data,owner,prior,i)
        rep=audit_controller_templates(stage,data,owner,prior,i,wr,issues,allow[stage])
        rep.update({'recipe_path':str(paths[stage]),'recipe_blake3':selected[keys[stage]]['blake3'],'recipe_step_count':len(data['steps'])})
        reports[stage]=rep; prior[stage]=owner
    all_invalid=[x for rep in reports.values() for x in rep['invalid_environment_name_occurrences']]
    invalid_name_counts=collections.Counter()
    for rep in reports.values(): invalid_name_counts.update(rep['invalid_environment_name_counts'])
    word_counts=collections.Counter(); ref_counts=collections.Counter(); constructor_counts=collections.Counter(); unknown_counts=collections.Counter()
    for rep in reports.values():
        word_counts.update(rep['corrected_word_issue_categories'])
        ref_counts.update(rep['reference_issue_categories'])
        constructor_counts.update(rep['controller_constructor_issue_categories'])
        unknown_counts.update(rep['unknown_binding_kind_reason_counts'])
    codefiles=['src/source_built_recipe.rs','src/source_built_fixed_point.rs','src/source_built_recipe_exec.rs','src/source_built_recipe_controller.rs','src/source_built_fixed_point_action_controller.rs','src/source_built_fixed_point_action_trust.rs','src/source_built_stagex_programs_controller.rs','src/source_built_mantle_programs.rs','src/source_built_mantle_programs_graph.rs','src/source_built_mantle_programs_metadata_helper.rs','src/source_built_stagex_programs_controller_protocol.c','src/source_built_stagex_programs_controller_paths.c','src/source_built_stagex_programs_transform.c']
    codefiles=sorted(set(codefiles)|set(G['CONSUMERS'])|{'src/source_built_mantle_programs_native_helper.rs','src/source_built_mantle_programs_aws_helper.rs'})
    result={'scope':'Read-only inert-data audit of all six current selected recipes and every statically generated ControllerSpec template; no product/provider/controller/helper/compiler/NCL execution, build/test/lint/formatter, proof, or launch.',
        'subject_binding':base['subject_binding'],'observed_inputs':{'subject_json':{'path':str(subject_path),'size_bytes':subject_path.stat().st_size,'blake3':G['b3'](subject_path)},'plan':{'path':str(plan_path),'size_bytes':plan_path.stat().st_size,'blake3':G['b3'](plan_path)},'graph_report':{'path':str(graph_path),'size_bytes':graph_path.stat().st_size,'blake3':G['b3'](graph_path)},'recipes':{s:{'path':str(paths[s]),'size_bytes':paths[s].stat().st_size,'blake3':selected[keys[s]]['blake3']} for s in paths}},
        'selected_input_blake3':{k:v['blake3'] for k,v in selected.items()},
        'execution':{'interpreter':G['PYTHON'],'hash_tool':G['B3'],'commands':{'graph_inspector':[G['PYTHON'],str(HERE/'graph-audit.py'),str(COHORT),str(SOURCE),str(G['SCRATCH']),str(subject_path)],'template_inspector':[G['PYTHON'],str(HERE/'controller-template-audit.py'),str(COHORT),str(SOURCE),str(G['SCRATCH']),str(subject_path)]}},
        'review_script_blake3':{p:{'blake3':G['b3'](HERE/p),'size_bytes':(HERE/p).stat().st_size} for p in ['graph-audit.py','controller-template-audit.py']},
        'consumer_blake3':{p:{'blake3':G['b3'](SOURCE/p),'size_bytes':(SOURCE/p).stat().st_size} for p in codefiles},
        'runtime_payload_producer_consumer_contract':{'rust_producer':'source_built_stagex_programs_controller.rs::template_recipe/build_templates generates each Prepare invocation output as ArtifactKind::Text and same-slot Command consumes invocation_input=invocation.',
            'c_emitter':'source_built_stagex_programs_controller_protocol.c::mantle_controller_invocation serializes arguments, environment, optional cwd/argv0/executable_path, workspace_mounts, readonly_input_roots, optional process_attributes and descriptors; declared command inputs must be used exactly once and runtime mounts bind exact names.',
            'strict_decoder':'source_built_recipe.rs::RecipeInvocation has deny_unknown_fields; decode validates typed words, environment names, workspace/runtime mount roots, readonly input roots, process attributes and descriptor table.',
            'symbolic_payload_fields':['arguments','environment','cwd?','argv0?','executable_path?','workspace_mounts','readonly_input_roots','process_attributes?','descriptors?'],
            'execution_schedule':'source_built_recipe_exec.rs::run executes every recipe step in source order; static requests are audited irrespective of semantic-root reachability.',
            'runtime_bytes_observed':False},
        'current_consumer_ranges_read':{'template_builder':'src/source_built_stagex_programs_controller.rs','typed_template_validator':'src/source_built_recipe_controller.rs','typed_word_and_role_consumer':'src/source_built_recipe.rs','invocation_schema_and_decoder':'src/source_built_recipe.rs','invocation_emitter_and_observation_decoder':'src/source_built_stagex_programs_controller_protocol.c','invocation_decode_and_scheduler':'src/source_built_recipe_exec.rs','action_router':'src/source_built_fixed_point_action_controller.rs','Mantle exports':'src/source_built_mantle_programs.rs'},
        'full_category_tally':{'corrected_static_word_issue_categories':dict(word_counts),'reference_issue_categories':dict(ref_counts),'constructor_issue_categories':dict(constructor_counts),'unknown_binding_kind_reasons':dict(unknown_counts),'invalid_environment_name_counts':dict(invalid_name_counts),'invalid_environment_name_occurrence_count':sum(rep['invalid_environment_name_occurrence_count'] for rep in reports.values())},
        'invalid_environment_name_occurrences':all_invalid,'per_stage':reports,'mantle_export_and_help_root_audit':{},
        'limitations':['Controller-produced RecipeInvocation bytes are absent from static source inputs; symbolic producer/emitter/strict-decoder contracts are counted, but actual runtime decode and output remain protected-execution obligations.','Unknown artifact binding kinds are retained as uncovered obligations; no kind is inferred from a digest or pathname.','Physical profile/NAR identity, protected execution, fixed-point equality, reproducibility, native signed-row acceptance, and release eligibility are not established by this static audit.']}
    launch_criteria=[]
    for stage,rep in reports.items():
        for criterion,count in [('static_word_issues',rep['corrected_word_issue_count']),('reference_issues',rep['reference_issue_count']),('constructor_issues',rep['controller_constructor_issue_count']),('unknown_binding_kinds',rep['unknown_binding_kind_count'])]:
            if count: launch_criteria.append({'stage':stage,'criterion':criterion,'unresolved_count':count})
    result['coverage']={'per_stage':{stage:{'recipe_steps':{'total':rep['recipe_step_count'],'covered':rep['recipe_step_count']},'controller_specs':{'total':rep['controller_spec_count'],'covered':rep['controller_spec_count']},'slots':{'total':rep['controller_slot_count'],'covered':rep['controller_slot_count']-sum(1 for c in rep['controller_template_catalogs'] for x in c['slots'] if x.get('coverage')=='uncovered unknown variant')},'generated_templates':{'total':rep['generated_template_step_count'],'covered':rep['generated_template_step_count']},'static_words':{'total':rep['corrected_total_static_word_count'],'covered':rep['corrected_total_static_word_count']},'guard_conditions':{'total':rep['generated_template_guard_condition_count'],'covered':rep['generated_template_guard_condition_count']},'ObservePhase':{'total':rep['generated_template_role_counts'].get('ObservePhase',0),'covered':rep['generated_template_role_counts'].get('ObservePhase',0)},'runtime_symbolic_payloads':{'total':rep['runtime_derived_payload_count'],'covered':rep['runtime_derived_payload_symbolic_bindings_verified'],'actual_runtime_bytes_observed':0}} for stage,rep in reports.items()},'launch_blockers':launch_criteria,'uncovered_static_binding_kind_count':sum(r['unknown_binding_kind_count'] for r in reports.values())}
    for stage,rep in reports.items():
        result['coverage']['per_stage'][stage]['runtime_invocation_bindings']={'total':len(rep['runtime_invocation_bindings']),
            'covered':rep['runtime_invocation_static_binding_verified_count'],'actual_runtime_bytes_observed':0}
    graph_findings=base['coverage']['finding_counts']
    for criterion,count in graph_findings.items():
        if criterion=='recipe_reference_word_and_guard_errors': continue
        if count: launch_criteria.append({'stage':'graph','criterion':criterion,'unresolved_count':count})
    guard_failures=sum(len(rep['graph_guards']['issues']) for rep in base['six_recipe_reports'].values())
    if guard_failures: launch_criteria.append({'stage':'graph','criterion':'graph_guards','unresolved_count':guard_failures})
    if base['coverage']['unresolved_binding_kind_count']:
        launch_criteria.append({'stage':'graph','criterion':'static_source_binding_kinds','unresolved_count':base['coverage']['unresolved_binding_kind_count']})
    result['combined_static_contract_check']={'raw_graph_word_flags_superseded_by_complete_template_scopes':graph_findings['recipe_reference_word_and_guard_errors'],
        'guard_failures':guard_failures,'non_word_graph_findings':{k:v for k,v in graph_findings.items() if k!='recipe_reference_word_and_guard_errors'},
        'actual_runtime_invocation_bytes_observed':0,'runtime_invocation_static_bindings':sum(rep['runtime_invocation_static_binding_verified_count'] for rep in reports.values()),
        'static_source_binding_kind_failures':base['coverage']['unresolved_binding_kind_count'],
        'non_claim':'Complete measured static contracts only. Raw mis-scoped ControllerSpec diagnostics remain in the graph report. Runtime payloads are not unknown artifact kinds and remain explicit unobserved protected-execution obligations. No independent model acceptance or launch authority.'}
    for stage in ('mantle-stage1','mantle-stage2'):
        data=G['load'](paths[stage]); own=prior[stage]; sm={x['id']:x for x in data['steps']}; exports=data.get('exports',{}); roots={G['key'](x) for x in data.get('roots',[])}
        def resolve(r):
            if not isinstance(r,dict) or r.get('kind')!='produced': return {'valid':False,'reason':'not Produced','ref':r}
            aid=r.get('action_id',''); match=re.fullmatch(r'([A-Za-z0-9_.:-]+)/([A-Za-z0-9_.:-]+)',aid)
            if not match or match.group(1)!=stage: return {'valid':False,'reason':'malformed/foreign full producer namespace','ref':r}
            item=own.get(aid)
            if item is None: return {'valid':False,'reason':'unknown exact namespaced producer','ref':r}
            prod=sm[item['local']]; output=prod.get('outputs',{}).get(r.get('output_id'))
            if output is None: return {'valid':False,'reason':'unknown producer output ID','ref':r}
            return {'valid':True,'full_action_id':aid,'local_after_validation':item['local'],'output_id':r['output_id'],'output_kind':output.get('kind'),'producer_operation':prod['operation']['kind'],'producer_step_id':prod['id']}
        binary=exports.get(stage); smoke=exports.get('mantle-smoke'); helpstep=sm.get('mantle-help')
        if helpstep:
            op=helpstep['operation']; exe_ref=helpstep.get('inputs',{}).get(op.get('executable_input')); used=G['used_refs'](helpstep); stdout=helpstep.get('outputs',{}).get('stdout',{})
            cmd={'valid':op.get('kind')=='exec' and op.get('arguments')==[lit('--help')] and op.get('executable_input')=='mantle' and exe_ref==binary and G['key'](exe_ref) in used and stdout.get('kind')=='text' and smoke==ref(stage,'mantle-help','stdout'),'step_id':helpstep['id'],'operation_kind':op.get('kind'),'arguments':op.get('arguments'),'executable_ref':exe_ref,'executable_ref_actually_used':G['key'](exe_ref) in used,'stdout_output':stdout}
        else: cmd={'valid':False,'reason':'mantle-help step absent'}
        result['mantle_export_and_help_root_audit'][stage]={'export_names':sorted(exports),'canonical_binary_export':binary,'canonical_binary_owner':resolve(binary),'mantle_smoke_export':smoke,'mantle_smoke_owner':resolve(smoke),'binary_export_is_execution_root':G['key'](binary) in roots if binary else False,'smoke_export_is_execution_root':G['key'](smoke) in roots if smoke else False,'help_command':cmd,'root_count':len(data.get('roots',[]))}
    for stage,check in result['mantle_export_and_help_root_audit'].items():
        for criterion,passed in [('binary_owner',check['canonical_binary_owner']['valid']),('smoke_owner',check['mantle_smoke_owner']['valid']),('binary_root',check['binary_export_is_execution_root']),('smoke_root',check['smoke_export_is_execution_root']),('help_command',check['help_command']['valid'])]:
            if not passed: launch_criteria.append({'stage':stage,'criterion':'mantle_'+criterion,'unresolved_count':1})
    result['coverage']['launch_blockers']=launch_criteria
    result['bounded_verdict']='STATIC_TEMPLATE_REVIEW_CLEAR' if not launch_criteria else 'STATIC_TEMPLATE_REVIEW_BLOCKED'
    dest=G['SCRATCH']/'controller-template-audit.json'; dest.parent.mkdir(parents=True,exist_ok=True); dest.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
    print(json.dumps({'output':str(dest),'per_stage':{s:{'controllers':r['controller_spec_count'],'slots':r['controller_slot_count'],'templates':r['generated_template_step_count'],'template_words':r['generated_static_template_word_count'],'word_issues':r['corrected_word_issue_count'],'constructor_issues':r['controller_constructor_issue_count'],'payloads':r['runtime_derived_payload_count'],'source_role_issues':r['source_role_issue_count']} for s,r in reports.items()},'bounded_verdict':result['bounded_verdict']},sort_keys=True))

if __name__=='__main__':
    main()
