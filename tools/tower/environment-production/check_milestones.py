"""Validate cumulative real server placement stages and preserve functional sites."""
from pathlib import Path
import os,subprocess,json,csv,time
from navigation import audit
T=Path(__file__).resolve().parent;S=T.parents[2];E=S.parent;R=E/'cinderworks-environment-private';W=E.parents[1]
def read(name):return list(csv.DictReader((R/name/'objects.tsv').open(),delimiter='\t'))
def key(r):return tuple(r[k]for k in['x','z','id','shape','angle','width','length','block_walk'])
def edges(name):
    return {tuple(int(r[k])for k in['x','z','dx','dz'])for r in csv.DictReader((R/name/'walk-edges.tsv').open(),delimiter='\t')if(50<=int(r['x'])<=62 and 67<=int(r['z'])<=77)or(68<=int(r['x'])<=75 and 68<=int(r['z'])<=75)}
if __name__=='__main__':
    baseline=read('baseline-world');protected={key(r)for r in baseline if int(r['id'])not in[7016,7017,6143,6145]};original_edges=edges('baseline-world');reports=[]
    for stage in ['05-furnace-ash','06-boss-reward','07-whole-floor-refinement']:
        env=os.environ.copy();env['TOWER_ASSET_PACKAGE']=str(R/'package-v1');env['TOWER_CINDERWORKS_ENVIRONMENT']=str(R/'placements'/(stage+'.tsv'))
        result=subprocess.run([str(E/'build/cinderworks-server/debug/examples/cinderworks_environment_audit.exe'),str(W/'resource-staging/server/work/integration/a-kotlin/.data'),str(R/stage)],env=env,capture_output=True,text=True)
        (R/(stage+'-audit.log')).write_text(result.stdout+result.stderr);result.check_returncode();print(result.stdout.strip(),flush=True)
        report=audit(stage);new={key(r)for r in read(stage)};assert protected<=new,(stage,'protected existing scenery/functional sites changed',protected-new)
        assert edges(stage)==original_edges,(stage,'combat movement edges changed',len(original_edges-edges(stage)))
        reports.append({'stage':stage,'reachable_tiles':report['reachable_tiles'],'review_route_steps':report['whole_floor_steps'],'all_rooms_and_anchor_approaches_reachable':True,'all_protected_original_placements_unchanged':True,'combat_cardinal_edges_exact':True})
    (R/'milestone-validation.json').write_text(json.dumps(reports,indent=2)+'\n')
