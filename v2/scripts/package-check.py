from pathlib import Path
import subprocess,tarfile,tempfile,json,os,hashlib
from materialize import instantiate
root=Path(__file__).resolve().parents[1];logs=root/'test-results';logs.mkdir(exist_ok=True);work=root/'harness-work';work.mkdir(exist_ok=True)
def run(label,cmd,cwd=root,env=None):
 p=subprocess.run(cmd,cwd=cwd,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,env=env);(logs/(label+'.log')).write_text(p.stdout)
 print(label,'PASS' if p.returncode==0 else 'FAIL',flush=True)
 if p.returncode:print(p.stdout[-6000:]);raise SystemExit(p.returncode)
 return p.stdout
run('package-create',['cargo','package','--manifest-path','Cargo.toml','--locked','--offline','--allow-dirty','--no-verify'])
archive=root/'target/package/tezbar-asciimath-to-latex-converter-0.1.0.crate';dest=Path(tempfile.mkdtemp(prefix='package-',dir=work))
with tarfile.open(archive) as tar:
 names=tar.getnames();prefix='tezbar-asciimath-to-latex-converter-0.1.0/'
 assert not any('/node_modules/' in n or '/target/' in n or '/harness-work/' in n for n in names)
 for member in tar.getmembers():
  parts=Path(member.name).parts;assert '..' not in parts
  if member.isfile():
   p=dest/Path(*parts);p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(tar.extractfile(member).read())
extracted=dest/prefix.rstrip('/')
entries=json.loads((root/'INDEX.json').read_text())['files']
for e in entries:
 p=extracted/e['path'];assert p.is_file(),e['path']
 data=p.read_bytes()
 # Cargo normalizes only its top-level manifest; original bytes are retained.
 if e['path']=='Cargo.toml':data=(extracted/'Cargo.toml.orig').read_bytes()
 assert len(data)==e['bytes'] and hashlib.sha256(data).hexdigest()==e['sha256'],e['path']
assert (extracted/'fixtures/katex/package/dist/fonts/KaTeX_Main-Regular.woff2').is_file()
assert (extracted/'fixtures/library/package/build/index.js').is_file()
instantiate(extracted)
env=os.environ.copy();env.pop('SUPERCMD_EXTENSION_PATHS',None);env.pop('CARGO_TARGET_DIR',None)
for tc in ['stable','1.85.0']:
 for manifest in ['Cargo.toml','integration/direct/Cargo.toml','integration/harness/Cargo.toml']:
  run('extracted-'+tc+'-'+manifest.replace('/','-'),['cargo','+'+tc,'test','--manifest-path',manifest,'--locked','--offline'],extracted,env)
summary={'archiveSha256':hashlib.sha256(archive.read_bytes()).hexdigest(),'archiveBytes':archive.stat().st_size,'files':len(names),'indexRecordsVerified':len(entries),'extractedOfflineRust':'stable and 1.85.0: core/direct adapter/NativeSessions+registry PASS','absoluteHostOrSourceDependency':False}
(logs/'package-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary),flush=True)
