"""Set completed-tutorial state on an offline disposable gameplay fixture.

This is not proof of character creation/tutorial parity. It isolates ordinary
inventory tests from the separately tracked first-login tutorial/creator flow.
"""
import argparse,json,sqlite3
from pathlib import Path

parser=argparse.ArgumentParser()
parser.add_argument('database',type=Path);parser.add_argument('record',type=Path)
args=parser.parse_args()
root=Path(__file__).resolve().parent.parent
allowed={(root/'work/integration/a-kotlin/.data/saves/game.db').resolve(),(root/'work/state/integration-b/accounts.sqlite').resolve()}
path=args.database.resolve(strict=True)
if path not in allowed or args.record.exists(): raise ValueError('requires an owned disposable database and new record path')
symbols=dict(line.split('\t',1)[::-1] for line in (root/'work/integration/b-rust/symbols/varp.sym').read_text().splitlines() if '\t' in line and not line.startswith('#'))
key=str(int(symbols['tut2_completed']))
with sqlite3.connect(path) as db:
    db.execute('BEGIN IMMEDIATE')
    row=db.execute("SELECT c.id,c.world_id,c.varps FROM characters c JOIN accounts a ON a.id=c.account_id WHERE a.login_username='integration239' AND c.realm_id=1").fetchone()
    if row is None or row[1] is not None: raise ValueError('integration account must exist and be offline')
    values=json.loads(row[2])
    if not isinstance(values,dict): raise ValueError('unsupported persisted varp format')
    before=values.get(key)
    values[key]=1
    db.execute('UPDATE characters SET varps=? WHERE id=?',(json.dumps(values,separators=(',',':')),row[0]))
args.record.write_text(json.dumps({'scenario':'completed-tutorial-gameplay-v1','database':str(path),'varp':int(key),'previous':before,'value':1,'fixture_setup_only':True,'does_not_validate_tutorial_or_creator':True},indent=2)+'\n')
print('Prepared offline completed-tutorial gameplay fixture; no tutorial/creator result claimed')
