"""Create versioned fixtures and fresh run copies; never reset an existing database."""
import argparse,hashlib,json,sqlite3
from integration_creator_fixture import prepare_creator_resume
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
FIXTURE=ROOT/'work/preparation/fixtures/basic-v1'
KOTLIN_FIXTURE=ROOT/'work/preparation/fixtures/basic-kotlin-v1'

def kotlin_fixture():
    source=ROOT/'work/integration/a-kotlin/.data/saves/game.db'
    if export(source)['world_id'] is not None:raise ValueError('Kotlin fixture source is online')
    expected=export(FIXTURE/'state.sqlite')
    target=KOTLIN_FIXTURE/'state.sqlite'
    backup(source,target)
    with sqlite3.connect(target) as db:
        if not db.execute('SELECT count(*) FROM flyway_schema_history WHERE success=1').fetchone()[0]:raise ValueError('Missing real migration history')
        cid=db.execute("SELECT c.id FROM characters c JOIN accounts a ON a.id=c.account_id WHERE a.login_username='integration239' AND c.realm_id=1").fetchone()[0]
        db.execute('UPDATE characters SET x=?,z=?,level=?,varps=?,run_energy=?,xp_rate_in_hundreds=?,world_id=NULL WHERE id=?',(expected['x'],expected['z'],expected['level'],json.dumps(expected['varps']),expected['run_energy'],expected['xp_rate_in_hundreds'],cid))
        db.execute('DELETE FROM stats WHERE character_id=?',(cid,))
        for s in expected['stats']:db.execute('INSERT INTO stats(character_id,stat_id,vis_level,base_level,fine_xp) VALUES (?,?,?,?,?)',(cid,s['stat_id'],s['vis_level'],s['base_level'],s['fine_xp']))
        db.execute('DELETE FROM inventory_objs WHERE inventories_id IN (SELECT id FROM inventories WHERE character_id=?)',(cid,))
        for item in expected['inventory']:
            db.execute('INSERT OR IGNORE INTO inventories(character_id,inv_type) VALUES (?,?)',(cid,item['inv_type']))
            inv=db.execute('SELECT id FROM inventories WHERE character_id=? AND inv_type=?',(cid,item['inv_type'])).fetchone()[0]
            db.execute('INSERT INTO inventory_objs(inventories_id,slot,obj,count,vars) VALUES (?,?,?,?,?)',(inv,item['slot'],item['obj'],item['count'],item['vars']))
    if export(target)!=expected:raise ValueError('Canonical fixture mismatch')
    (KOTLIN_FIXTURE/'initial-state.json').write_text(json.dumps(expected,indent=2)+'\n')
    metadata=json.loads((FIXTURE/'fixture.json').read_text());metadata.update(name='basic-kotlin-v1',source='owned isolated Kotlin offline schema and migration history; canonical basic-v1 gameplay state')
    (KOTLIN_FIXTURE/'fixture.json').write_text(json.dumps(metadata,indent=2)+'\n')
    print(json.dumps({'status':'created','fixture_id':metadata['fixture_id']}))
def backup(source,target):
    if target.exists():raise ValueError('Refusing to replace a database')
    target.parent.mkdir(parents=True,exist_ok=True)
    with sqlite3.connect(source.resolve().as_uri()+'?mode=ro',uri=True) as src,sqlite3.connect(target) as dst:src.backup(dst)
def export(path):
    with sqlite3.connect(path.resolve().as_uri()+'?mode=ro',uri=True) as db:
        db.row_factory=sqlite3.Row
        row=db.execute("SELECT c.id,c.x,c.z,c.level,c.varps,c.run_energy,c.xp_rate_in_hundreds,c.world_id FROM characters c JOIN accounts a ON a.id=c.account_id WHERE a.login_username='integration239' AND c.realm_id=1").fetchone()
        if row is None:raise ValueError('Missing disposable fixture account')
        result=dict(row);cid=result.pop('id');result['varps']=json.loads(result['varps'])
        result['stats']=[dict(r) for r in db.execute('SELECT stat_id,vis_level,base_level,fine_xp FROM stats WHERE character_id=? ORDER BY stat_id',(cid,))]
        result['inventory']=[dict(r) for r in db.execute('SELECT i.inv_type,o.slot,o.obj,o.count,o.vars FROM inventories i JOIN inventory_objs o ON o.inventories_id=i.id WHERE i.character_id=? ORDER BY i.inv_type,o.slot',(cid,))]
        return result
def main():
    p=argparse.ArgumentParser();p.add_argument('mode',choices=['fixture','kotlin-fixture','clone','export']);p.add_argument('--run');p.add_argument('--environment',choices=['kotlin','rust']);p.add_argument('--output');p.add_argument('--fixture',choices=['basic','tower','creator-resume'],default='basic');a=p.parse_args()
    if a.mode=='kotlin-fixture':kotlin_fixture();return
    if a.mode=='fixture':
        source=ROOT/'work/state/integration-b/accounts.sqlite'
        if export(source)['world_id'] is not None:raise ValueError('Source fixture is online')
        backup(source,FIXTURE/'state.sqlite')
        with sqlite3.connect(FIXTURE/'state.sqlite') as db:
            cid=db.execute("SELECT c.id FROM characters c JOIN accounts a ON a.id=c.account_id WHERE a.login_username='integration239' AND c.realm_id=1").fetchone()[0]
            db.execute('DELETE FROM inventory_objs WHERE inventories_id IN (SELECT id FROM inventories WHERE character_id=?)',(cid,))
            db.execute('INSERT OR IGNORE INTO inventories(character_id,inv_type) VALUES (?,93)',(cid,))
            inv=db.execute('SELECT id FROM inventories WHERE character_id=? AND inv_type=93',(cid,)).fetchone()[0]
            for slot,item,count in [(0,1277,1),(1,995,1000),(2,379,1),(3,379,1)]:db.execute('INSERT INTO inventory_objs(inventories_id,slot,obj,count,vars) VALUES (?,?,?,?,0)',(inv,slot,item,count))
            db.execute('UPDATE characters SET x=3224,z=3218,level=0,run_energy=10000,world_id=NULL WHERE id=?',(cid,))
        s=export(FIXTURE/'state.sqlite');data=json.dumps(s,sort_keys=True,separators=(',',':')).encode();fid=hashlib.sha256(data).hexdigest()
        (FIXTURE/'initial-state.json').write_text(json.dumps(s,indent=2)+'\n')
        (FIXTURE/'fixture.json').write_text(json.dumps({'fixture_id':fid,'name':'basic-v1','setup_only':True,'source':'owned integration-b offline copy','tutorial_completed_fixture':True,'does_not_validate_character_creation':True},indent=2)+'\n')
        print(json.dumps({'fixture_id':fid,'status':'created'}));return
    if not a.run or not a.run.replace('-','').isalnum() or not a.environment:raise ValueError('Safe run id and environment required')
    database=(ROOT/f'work/state/integration-prep-{a.run}/accounts.sqlite') if a.environment=='rust' else (ROOT/f'work/integration/preparation/{a.run}/.data/saves/game.db')
    if a.mode=='clone':
        template=KOTLIN_FIXTURE if a.environment=='kotlin' else FIXTURE
        if export(template/'state.sqlite')!=json.loads((FIXTURE/'initial-state.json').read_text()):raise ValueError('Template state changed')
        backup(template/'state.sqlite',database)
        if a.fixture=='tower':
            with sqlite3.connect(database) as db:db.execute("UPDATE characters SET x=10424,z=120,level=0 WHERE realm_id=1 AND account_id=(SELECT id FROM accounts WHERE login_username='integration239')")
        if a.fixture=='creator-resume':
            symbols=dict(line.split('\t',1)[::-1] for line in (ROOT/'work/integration/a-kotlin/.data/symbols/varp.sym').read_text().splitlines() if '\t' in line and not line.startswith('#'))
            with sqlite3.connect(database) as db:
                prepare_creator_resume(db,int(symbols['tutorial_2']),int(symbols['tut2_completed']))
        fid=hashlib.sha256(json.dumps(export(database),sort_keys=True,separators=(',',':')).encode()).hexdigest()
        print(json.dumps({'database':str(database),'fixture_id':fid,'fixture_setup_only':True}))
    else:
        if not a.output:raise ValueError('Output required')
        with Path(a.output).open('x',encoding='utf8') as f:json.dump(export(database),f,indent=2)
if __name__=='__main__':main()
