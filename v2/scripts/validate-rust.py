from pathlib import Path
import subprocess,json,os
from materialize import materialize
root=Path(__file__).resolve().parents[1];logs=root/'test-results';logs.mkdir(exist_ok=True)
runroot=materialize(root)
manifest_paths=['Cargo.toml','integration/direct/Cargo.toml','integration/harness/Cargo.toml']
results=[]
def run(label,cmd,env=None):
 p=subprocess.run(cmd,cwd=runroot,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,env=env)
 (logs/(label+'.log')).write_text(p.stdout);results.append({'label':label,'command':cmd,'exitCode':p.returncode})
 print(label, 'PASS' if p.returncode==0 else 'FAIL',flush=True)
 if p.returncode:print(p.stdout[-6000:],flush=True);raise SystemExit(p.returncode)
owned=[str(p.relative_to(root)) for base in ['src','adapter','tests','integration/harness/src','integration/harness/tests','integration/harness/runtime/src','integration/harness/runtime/tauri-stub/src'] for p in (root/base).rglob('*.rs')]
for tc in ['stable','1.85.0']:
 env=os.environ.copy();env.pop('SUPERCMD_EXTENSION_PATHS',None);env['CARGO_TARGET_DIR']=str(root/'target'/('validation-'+tc))
 run(tc+'-fmt',['rustup','run',tc,'rustfmt','--edition','2021','--config','skip_children=true','--check',*owned])
 for manifest in manifest_paths:
  name=manifest.replace('/','-').replace('.toml','')
  for operation in ['test','clippy']:
   cmd=['cargo','+'+tc,operation,'--manifest-path',manifest,'--locked','--offline']
   if operation=='clippy':cmd+=['--all-targets','--','-D','warnings']
   run(tc+'-'+name+'-'+operation,cmd,env)
  for operation in ['check','clippy']:
   cmd=['cargo','+'+tc,operation,'--manifest-path',manifest,'--locked','--offline','--all-targets','--target','x86_64-pc-windows-gnu']
   if operation=='clippy':cmd+=['--','-D','warnings']
   run(tc+'-'+name+'-windows-'+operation,cmd,env)
(logs/'rust-summary.json').write_text(json.dumps(results,indent=2)+'\n')
