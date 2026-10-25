from pathlib import Path
import hashlib,json,subprocess,sys
root=Path(__file__).resolve().parents[1];p=json.loads((root/'PROVENANCE.json').read_text());files=[]
# Git's actual visibility is the source of the review inventory. No staging occurs.
visible=subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','--','v2'],cwd=root.parent,text=True).splitlines()
for name in sorted(set(visible)):
 rel=name.removeprefix('v2/')
 if rel=='INDEX.json':continue
 data=(root/rel).read_bytes();origin='local implementation / review artifact (unstaged)'
 if rel.startswith('fixtures/own/'):origin=p['own']['repository']+'/blob/'+p['own']['sha']+'/'+rel.removeprefix('fixtures/own/')
 elif rel.startswith('fixtures/beta/'):origin='Tezbar git object '+p['beta']['sha']+':'+rel.removeprefix('fixtures/beta/').removesuffix('.fixture')
 elif rel=='fixtures/host-committed/revision.txt':origin='Read-only git rev-parse HEAD: '+p['hostCommitted']['sha']
 elif rel.startswith('fixtures/host-committed/'):origin='Tezbar committed object '+p['hostCommitted']['sha']+':'+rel.removeprefix('fixtures/host-committed/').removesuffix('.fixture')
 elif rel=='fixtures/library/registry.json':origin='https://registry.npmjs.org/asciimath-to-latex/0.5.1'
 elif rel.startswith('fixtures/library'):origin=p['library']['url']+('#'+rel.removeprefix('fixtures/library/') if rel.startswith('fixtures/library/') else '')
 elif rel=='fixtures/katex/registry.json':origin='https://registry.npmjs.org/katex/0.16.25'
 elif rel.startswith('fixtures/katex'):origin=p['math']['url']+('#'+rel.removeprefix('fixtures/katex/') if rel.endswith('.tgz') is False else '')
 elif rel.startswith('fixtures/original/'):origin=p['original']['repository']+'/blob/'+p['original']['sha']+'/'+(('LICENSE' if rel.endswith('REPOSITORY_LICENSE') else rel.removeprefix('fixtures/original/')) if 'LICENSE' in rel else p['original']['path']+'/'+rel.removeprefix('fixtures/original/'))
 elif rel=='assets/icon.png':origin=p['icon']['url']
 elif rel=='src/symbols.json':origin='Initialized AMsymbols exported from executable locked asciimath-to-latex 0.5.1; scripts/oracle.cjs verifies exact table'
 elif rel=='fixtures/oracle.json':origin='Executable own locked library outputs; deterministic scripts/oracle.cjs'
 files.append({'path':rel,'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'source':origin})
(root/'INDEX.json').write_text(json.dumps({'algorithm':'SHA256','files':files},indent=2)+'\n');print('INDEX wrote',len(files),'records')
