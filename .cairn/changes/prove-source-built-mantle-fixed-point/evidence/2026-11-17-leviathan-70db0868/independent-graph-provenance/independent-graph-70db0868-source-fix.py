#!/usr/bin/env python3
from pathlib import Path
import sys
s=Path(sys.argv[1]).read_text()
s=s.replace("args=req['arguments'];c['static_templates']+=1", "args=req['invocation']['arguments'];c['static_templates']+=1")
sys.stdout.write(s)
