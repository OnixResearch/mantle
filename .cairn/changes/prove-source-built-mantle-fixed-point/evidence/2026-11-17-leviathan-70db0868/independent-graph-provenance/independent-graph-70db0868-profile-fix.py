#!/usr/bin/env python3
from pathlib import Path
import sys
s=Path(sys.argv[1]).read_text()
s=s.replace("if len(record_prefix)>30 and not is_selected:raise ValueError('record identity not within initial bounded fields')", "if len(record_prefix)>30 and not is_selected:record_prefix=[]")
sys.stdout.write(s)
