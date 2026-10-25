from pathlib import Path
import tempfile,shutil,subprocess,hashlib,json
root=Path(__file__).resolve().parents[1];work=root/'harness-work';work.mkdir(exist_ok=True);d=Path(tempfile.mkdtemp(prefix='v1-',dir=work));source=root/'fixtures/own';shutil.copytree(source,d,dirs_exist_ok=True)
logs=root/'test-results';logs.mkdir(exist_ok=True)
commands=[['npm','ci','--offline','--legacy-peer-deps','--ignore-scripts','--no-audit','--no-fund'],['npm','test'],['npm','run','typecheck']];results=[]
for i,cmd in enumerate(commands):
 p=subprocess.run(cmd,cwd=d,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT);(logs/f'v1-{i}.log').write_text(p.stdout);results.append({'command':cmd,'exitCode':p.returncode});print('V1',cmd[1],'PASS' if p.returncode==0 else 'FAIL',flush=True)
 if p.returncode:print(p.stdout);raise SystemExit(p.returncode)
for f in source.rglob('*'):
 if f.is_file():assert f.read_bytes()==(d/f.relative_to(source)).read_bytes(),f
(logs/'v1-summary.json').write_text(json.dumps({'results':results,'originalFilesUnchanged':True,'lockSha256':hashlib.sha256((d/'package-lock.json').read_bytes()).hexdigest()},indent=2)+'\n')
