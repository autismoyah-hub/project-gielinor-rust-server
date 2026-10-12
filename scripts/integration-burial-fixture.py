"""Append two bones to an offline disposable account; never replace its earned items/XP."""
import json
from pathlib import Path
import sqlite3
import sys
root=Path(__file__).resolve().parent.parent
target, name=sys.argv[1:]
paths={"floor":root/"work/state/integration-floor/accounts.sqlite",
       "reference":root/"work/integration/a-kotlin/.data/saves/game.db"}
database=paths[target]
record=(root/"work/integration"/name).resolve()
if record.parent!=(root/"work/integration").resolve() or record.exists() or record.with_suffix(".sqlite").exists():
    raise ValueError("use a new record basename")
db=sqlite3.connect(database.as_uri()+"?mode=rw",uri=True)
row=db.execute("""SELECT c.id,c.world_id,s.fine_xp FROM characters c JOIN accounts a ON a.id=c.account_id
    JOIN stats s ON s.character_id=c.id AND s.stat_id=5
    WHERE a.login_username='integration239' AND c.realm_id=1""").fetchone()
if row is None or row[1] is not None or row[2]!=0:
    raise ValueError("expected logged-out account with unchanged zero Prayer XP")
with sqlite3.connect(record.with_suffix(".sqlite")) as backup: db.backup(backup)
with db:
    db.execute("BEGIN IMMEDIATE")
    if db.execute("SELECT world_id FROM characters WHERE id=?",(row[0],)).fetchone()[0] is not None:
        raise ValueError("account logged in")
    inventory=db.execute("SELECT id FROM inventories WHERE character_id=? AND inv_type=93",(row[0],)).fetchone()[0]
    before=db.execute("SELECT slot,obj,count,vars FROM inventory_objs WHERE inventories_id=? ORDER BY slot",(inventory,)).fetchall()
    if any(item[1] in (526,530) for item in before): raise ValueError("bones already present; do not reseed")
    free=[slot for slot in range(28) if slot not in {item[0] for item in before}]
    if len(free)<2: raise ValueError("requires two free slots")
    added=[(free[0],526,1,0),(free[1],530,1,0)]
    db.executemany("INSERT INTO inventory_objs(inventories_id,slot,obj,count,vars) VALUES (?,?,?,?,?)",[(inventory,*item) for item in added])
db.close()
record.write_text(json.dumps({"fixture_setup_only":True,"database":str(database),"backup":str(record.with_suffix('.sqlite')),"inventory_before":before,"added":added,"prayer_fine_xp_unchanged":0},indent=2)+"\n")
print(f"Prepared {target} burial fixture; existing inventory and XP preserved")
