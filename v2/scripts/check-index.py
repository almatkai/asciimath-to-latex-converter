from pathlib import Path
import hashlib,json,subprocess,sys
root=Path(__file__).resolve().parents[1];entries=json.loads((root/'INDEX.json').read_text())['files']
for e in entries:
 p=root/e['path'];data=p.read_bytes();assert len(data)==e['bytes'],e['path'];assert hashlib.sha256(data).hexdigest()==e['sha256'],e['path']
if '--git' in sys.argv:
 visible=subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','--','v2'],cwd=root.parent,text=True).splitlines();visible={p.removeprefix('v2/') for p in visible}
 assert all(e['path'] in visible for e in entries),'Indexed file is invisible to Git'
print('INDEX verified:',len(entries),'files')
