from pathlib import Path
import subprocess,json,hashlib,base64,argparse
root=Path(__file__).resolve().parents[1];repo=root.parent
p=argparse.ArgumentParser();p.add_argument('--host',type=Path);args=p.parse_args()
assert subprocess.check_output(['git','rev-parse','--show-toplevel'],cwd=repo,text=True).strip()==str(repo)
assert subprocess.check_output(['git','branch','--show-current'],cwd=repo,text=True).strip()=='feat/native-v2-rust'
assert not subprocess.check_output(['git','diff','--cached','--name-only'],cwd=repo,text=True).strip()
assert subprocess.check_output(['git','diff','--name-only'],cwd=repo,text=True).splitlines()==['README.md']
for f in (root/'fixtures/own').rglob('*'):
 if f.is_file():
  rel=str(f.relative_to(root/'fixtures/own'));assert f.read_bytes()==subprocess.check_output(['git','show','HEAD:'+rel],cwd=repo)
  if rel!='README.md':assert f.read_bytes()==(repo/rel).read_bytes(),rel
original=(root/'fixtures/own/README.md').read_bytes();assert (repo/'README.md').read_bytes()==original+b'\nNative Rust V2 implementation and integration notes: [v2/README.md](v2/README.md).\n'
for name,tar in [('library','library.tgz'),('katex','katex/katex-0.16.25.tgz')]:
 info=json.loads((root/'fixtures'/name/'registry.json').read_text());data=(root/'fixtures'/tar).read_bytes();assert info['dist']['integrity']=='sha512-'+base64.b64encode(hashlib.sha512(data).digest()).decode()
icon=(root/'assets/icon.png').read_bytes();assert len(icon)==16920;assert hashlib.sha256(icon).hexdigest()=='85d2d2a64b001b214cd2b88d01260f81b9c393863013abf3f7422d08612a1b88';assert icon==(root/'fixtures/original/assets/icon.png').read_bytes()
if args.host:
 pin='fd300e276356587726566026f072563eb9c34cb1'
 for f in (root/'fixtures/beta').rglob('*'):
  if f.is_file():
   rel=str(f.relative_to(root/'fixtures/beta')).removesuffix('.fixture');assert f.read_bytes()==subprocess.check_output(['git','-C',str(args.host),'show',pin+':'+rel]),rel
print('Scope, unchanged V1/lock, immutable own/beta fixtures, exact icon and npm SRI PASS')
