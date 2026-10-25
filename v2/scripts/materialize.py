"""Harness Cargo manifests are templates so Cargo packaging retains nested fixtures."""
from pathlib import Path
import shutil,tempfile,os
SKIP={'target','node_modules','harness-work','test-results','generated','.git'}
def instantiate(root):
    for f in root.rglob('Cargo.toml.template'):
        f.with_name('Cargo.toml').write_bytes(f.read_bytes())
    return root
def materialize(root):
    work=root/'harness-work';work.mkdir(exist_ok=True);dest=Path(tempfile.mkdtemp(prefix='rust-',dir=work))
    for directory,dirs,files in os.walk(root):
        dirs[:]=[d for d in dirs if d not in SKIP]
        for name in files:
            if name.endswith('.log'):continue
            p=Path(directory)/name;out=dest/p.relative_to(root);out.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,out)
    return instantiate(dest)
if __name__=='__main__':print(materialize(Path(__file__).resolve().parents[1]))
