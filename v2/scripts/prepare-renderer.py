from pathlib import Path
import shutil,subprocess,tempfile
root=Path(__file__).resolve().parents[1];(root/'harness-work').mkdir(exist_ok=True);dest=Path(tempfile.mkdtemp(prefix='renderer-',dir=root/'harness-work'))
shutil.copytree(root/'fixtures/beta',dest,dirs_exist_ok=True)
subprocess.run(['patch','--batch','--dry-run','-p1','-d',str(dest),'-i',str(root/'integration/required-renderer.patch')],check=True)
subprocess.run(['patch','--batch','-p1','-d',str(dest),'-i',str(root/'integration/required-renderer.patch')],check=True)

shutil.copytree(dest,root/'harness-work/renderer',dirs_exist_ok=True)
