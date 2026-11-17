#!/usr/bin/env python3
import sys
from pathlib import Path
p=Path(sys.argv[1]);s=p.read_text()
a=s.index('RECIPES=[');b=s.index('\nINDEX=',a)
s=s[:a]+'''RECIPES=[
 ('stagex-transition','program-sources/source-built-stagex-programs/stagex-transition.json'),
 ('stagex-provider-publication','program-sources/source-built-stagex-programs/stagex-provider-publication.json'),
 ('full-source-native-provider','program-sources/fixed-point-native-source-program/source-built/native/program.json'),
 ('full-source-rust-provider','inputs/rust-source-archives/source-built/rust/program.json'),
 ('mantle-stage1','program-sources/source-built-mantle-programs/bootstrap/source-built/mantle-stage1.json'),
 ('mantle-stage2','program-sources/source-built-mantle-programs/bootstrap/source-built/mantle-stage2.json')]
'''+s[b:]
s=s.replace('inputs/native-source-authority/source-built/native/program.json.expansion.json','program-sources/fixed-point-native-source-program/source-built/native/program.expansion.json')
s=s.replace("count['dependency_edges']+=len(actual);dependencies[stage+'/'+sid]=actual", "if operation=='observe-phase':actual.add(stage+'/'+op['phase'])\n   count['dependency_edges']+=len(actual);dependencies[stage+'/'+sid]=actual")
s=s.replace("if family=='roots' and ns==stage:export_refs.append(aid)","if ns==stage:export_refs.append(aid)")
sys.stdout.write(s)
