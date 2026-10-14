"""Five-hide Tower fixture, applied only to a freshly cloned offline account."""
def prepare(db):
    rows = db.execute("""SELECT c.id,c.world_id FROM characters c JOIN accounts a ON a.id=c.account_id
        WHERE a.login_username='integration239' AND c.realm_id=1""").fetchall()
    if len(rows) != 1 or rows[0][1] is not None:
        raise ValueError('Tower fixture requires one offline disposable character')
    character = rows[0][0]
    if not db.in_transaction:
        db.execute('BEGIN')
    db.execute('DELETE FROM inventory_objs WHERE inventories_id IN (SELECT id FROM inventories WHERE character_id=?)', (character,))
    db.execute('INSERT OR IGNORE INTO inventories(character_id,inv_type) VALUES(?,93)', (character,))
    inventory = db.execute('SELECT id FROM inventories WHERE character_id=? AND inv_type=93', (character,)).fetchone()[0]
    db.executemany('INSERT INTO inventory_objs(inventories_id,slot,obj,count,vars) VALUES(?,?,1739,1,0)',
                   [(inventory, slot) for slot in range(5)])
    db.execute('UPDATE characters SET x=10424,z=120,level=0 WHERE id=?', (character,))
