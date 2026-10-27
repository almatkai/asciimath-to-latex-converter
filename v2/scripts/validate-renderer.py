from pathlib import Path
import subprocess,json
from materialize import materialize
root=Path(__file__).resolve().parents[1];logs=root/'test-results';logs.mkdir(exist_ok=True)
runroot=materialize(root)
commands=[('prepare',['python3','scripts/prepare-renderer.py'],root),('views',['cargo','run','--manifest-path','integration/harness/Cargo.toml','--locked','--offline','--bin','views'],runroot),('renderer',['npm','test'],root/'integration/renderer'),('renderer-types',['npx','--no-install','tsc','-p','tsconfig.json'],root/'integration/renderer'),('schema',['node','scripts/schema.cjs'],root),('oracle',['node','scripts/oracle.cjs'],root)]
results=[]
for label,cmd,cwd in commands:
 p=subprocess.run(cmd,cwd=cwd,text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 (logs/(label+'.log')).write_text(p.stdout+p.stderr)
 if label=='views':(root/'harness-work/views.json').write_text(p.stdout)
 results.append({'label':label,'command':cmd,'exitCode':p.returncode});print(label,'PASS' if p.returncode==0 else 'FAIL',flush=True)
 if p.returncode:print(p.stdout+p.stderr);raise SystemExit(p.returncode)
(logs/'renderer-summary.json').write_text(json.dumps(results,indent=2)+'\n')
