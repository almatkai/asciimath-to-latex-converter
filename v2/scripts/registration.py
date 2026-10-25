"""Generate review-only registration patch from an exported host tree; never edits it."""
from pathlib import Path
import json,difflib,argparse
root=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser();parser.add_argument('--source',type=Path,default=root/'fixtures/beta');parser.add_argument('--output',type=Path,default=root/'integration/registration.patch');parser.add_argument('--files-output',type=Path);args=parser.parse_args()
base=args.source.resolve(); manifest=json.loads((root/'manifest.json').read_text());slug=manifest['slug']
files={p:((base/p) if (base/p).exists() else (base/(p+'.fixture'))).read_text() for p in ['src-tauri/src/native_core/extensions/mod.rs','src-tauri/src/native_core/extensions/v2_runtime.rs','src-tauri/src/native_core/extensions/v2_catalog.json','src-tauri/src/native_core/extensions/v2_catalog.rs','src-tauri/Cargo.toml']}
changed=dict(files)
p='src-tauri/src/native_core/extensions/mod.rs';assert 'mod v2_asciimath;' not in files[p]
changed[p]+='\n#[path = "../../../native-extensions/asciimath-to-latex-converter/adapter/v2_asciimath.rs"]\nmod v2_asciimath;\n'
p='src-tauri/src/native_core/extensions/v2_runtime.rs';s=files[p]
s=s.replace('    match value {','    match value {\n        "asciimath-to-latex-converter" => Some(value),',1)
s=s.replace('pub(super) fn commands(slug: &str) ->', 'pub(super) fn commands(slug: &str) ->',1)
needle='    match slug {';first=s.index(needle);s=s[:first]+s[first:].replace(needle,needle+'\n        "asciimath-to-latex-converter" => super::v2_asciimath::commands(),',1)
last=s.rindex(needle);s=s[:last]+s[last:].replace(needle,needle+'\n        "asciimath-to-latex-converter" => {\n            super::v2_asciimath::handle(ctx, command, operation, payload, state).await\n        }',1);changed[p]=s
p='src-tauri/src/native_core/extensions/v2_catalog.json';old=json.loads(files[p]);assert not any(v['slug']==slug for v in old)
# Only append; preserve preceding port bytes and catalog order.
assert files[p].rstrip().endswith(']')
changed[p]=files[p].rstrip()[:-1].rstrip()+',\n'+''.join('  '+line+'\n' for line in json.dumps(manifest,indent=2).splitlines())+']\n'
assert json.loads(changed[p])[:-1]==old
p='src-tauri/src/native_core/extensions/v2_catalog.rs';needle=f'assert_eq!(items.len(), {len(old)});';assert needle in files[p];changed[p]=files[p].replace(needle,f'assert_eq!(items.len(), {len(old)+1});')
p='src-tauri/Cargo.toml';needle='[dependencies]';assert needle in files[p];changed[p]=files[p].replace(needle,needle+'\ntezbar-asciimath-to-latex-converter = { path = "native-extensions/asciimath-to-latex-converter" }',1)
patch=''.join(''.join(difflib.unified_diff(files[p].splitlines(True),changed[p].splitlines(True),fromfile='a/'+p,tofile='b/'+p)) for p in files)
args.output.write_text(patch)
print(f'Review-only patch: {len(old)} -> {len(old)+1} manifests; preceding entries unchanged')

if args.files_output:
 for p,body in changed.items():
  dest=args.files_output/(p+'.fixture' if p.endswith('Cargo.toml') else p);dest.parent.mkdir(parents=True,exist_ok=True);dest.write_text(body)
