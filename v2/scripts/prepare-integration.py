"""Create a new review layout. Does not apply patches, install, or rebuild a host."""
from pathlib import Path
import argparse,tarfile,shutil,subprocess
root=Path(__file__).resolve().parents[1];p=argparse.ArgumentParser();p.add_argument('--output',required=True,type=Path);args=p.parse_args();out=args.output.resolve()
if out.exists():raise SystemExit('Output must be a new review directory')
subprocess.run(['cargo','package','--manifest-path',str(root/'Cargo.toml'),'--locked','--offline','--allow-dirty','--no-verify'],check=True)
archive=root/'target/package/tezbar-asciimath-to-latex-converter-0.1.0.crate';out.mkdir(parents=True)
dest=out/'src-tauri/native-extensions/asciimath-to-latex-converter';dest.mkdir(parents=True)
with tarfile.open(archive) as tar:
 for member in tar.getmembers():
  parts=Path(member.name).parts[1:]
  if not parts or not member.isfile():continue
  assert '..' not in parts
  f=dest/Path(*parts);f.parent.mkdir(parents=True,exist_ok=True);f.write_bytes(tar.extractfile(member).read())
icon=out/'src/renderer/public/extensions/icons/asciimath-to-latex-converter.png';icon.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(root/'assets/icon.png',icon)
licenses=out/'src/renderer/public/extensions/licenses/asciimath-to-latex-converter';licenses.mkdir(parents=True)
for f in root.glob('LICENSE*'):shutil.copyfile(f,licenses/f.name)
shutil.copyfile(root/'NOTICE',licenses/'NOTICE');shutil.copyfile(root/'fixtures/katex/package/LICENSE',licenses/'LICENSE-KATEX');shutil.copyfile(root/'fixtures/library/package/package.json',licenses/'ASCIIMATH-LIBRARY-PACKAGE.json')
for name in ['registration.patch','required-renderer.patch','current-host-icon-prerequisite.patch']:shutil.copyfile(root/'integration'/name,out/name)
print('Review layout prepared; patches are unapplied and no host was installed:',out)
