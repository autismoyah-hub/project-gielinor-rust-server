"""Install the curated original-only package into a fresh private runtime folder."""
from pathlib import Path
import shutil,json
T=Path(__file__).resolve().parent;S=T.parents[2];P=S/'art/tower/cinderworks-environment';R=S.parent/'cinderworks-environment-private'
def main():
    package=R/'package-v1';placements=R/'placements';package.mkdir(parents=True,exist_ok=True);placements.mkdir(exist_ok=True)
    for source in (P/'overlay').iterdir():
        target=package/source.name
        if target.exists()and target.read_bytes()!=source.read_bytes():raise RuntimeError('Refusing to replace different private runtime file: '+str(target))
        if not target.exists():shutil.copy2(source,target)
    target=placements/'07-whole-floor-refinement.tsv';source=P/'environment.tsv'
    if target.exists()and target.read_bytes()!=source.read_bytes():raise RuntimeError('Refusing changed private environment placement')
    if not target.exists():shutil.copy2(source,target)
    print('Original overlay installed:',package,'placements:',target)
if __name__=='__main__':main()
