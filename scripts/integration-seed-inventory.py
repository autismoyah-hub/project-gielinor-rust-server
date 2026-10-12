"""Seed a versioned inventory fixture in an OFFLINE disposable integration character.

This is explicit fixture setup, never evidence of an in-game item transaction.
Refuses occupied accounts, existing different contents and paths outside A/B state.
"""
import argparse
import json
from pathlib import Path
import sqlite3


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("database", type=Path)
    parser.add_argument("record", type=Path)
    parser.add_argument("--position", help="Optional explicit offline X,Z,LEVEL scenario placement")
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    allowed = {
        (root / "work/integration/a-kotlin/.data/saves/game.db").resolve(),
        (root / "work/state/integration-b/accounts.sqlite").resolve(),
    }
    database = args.database.resolve(strict=True)
    if database not in allowed:
        raise ValueError("only the two disposable integration databases are permitted")
    if args.record.exists():
        raise ValueError("use a new fixture record path")
    position = None
    if args.position:
        position = tuple(map(int, args.position.split(',')))
        if len(position) != 3 or not (0 <= position[0] < 16384 and 0 <= position[1] < 16384 and 0 <= position[2] < 4):
            raise ValueError("invalid fixture position")
    expected = [(0, 1277, 1, 0), (1, 995, 1000, 0), (2, 379, 1, 0), (3, 379, 1, 0)]
    with sqlite3.connect(database) as db:
        db.execute("PRAGMA foreign_keys=ON")
        db.execute("BEGIN IMMEDIATE")
        row = db.execute("""SELECT c.id,c.world_id,c.x,c.z,c.level FROM characters c JOIN accounts a
            ON a.id=c.account_id WHERE a.login_username='integration239' AND c.realm_id=1""").fetchone()
        if row is None or row[1] is not None:
            raise ValueError("integration239 must exist and be logged out")
        character = row[0]
        existing = db.execute("""SELECT i.inv_type,o.slot,o.obj,o.count,o.vars
            FROM inventories i JOIN inventory_objs o ON o.inventories_id=i.id
            WHERE i.character_id=? ORDER BY i.inv_type,o.slot""", (character,)).fetchall()
        exact = [(93, *item) for item in expected]
        if existing and existing != exact:
            raise ValueError("character has different inventory contents; no changes made")
        if not existing:
            db.execute("INSERT OR IGNORE INTO inventories(character_id,inv_type) VALUES (?,93)", (character,))
            inventory = db.execute("SELECT id FROM inventories WHERE character_id=? AND inv_type=93", (character,)).fetchone()[0]
            db.executemany("INSERT INTO inventory_objs(inventories_id,slot,obj,count,vars) VALUES (?,?,?,?,?)", [(inventory, *item) for item in expected])
        if position is not None:
            db.execute("UPDATE characters SET x=?,z=?,level=? WHERE id=?", (*position, character))
    args.record.write_text(json.dumps({"scenario": "inventory-v1", "database": str(database), "username": "integration239", "realm": 1, "items": exact, "position_before": row[2:], "fixture_position": position, "fixture_setup_only": True}, indent=2) + "\n")
    print("Prepared offline disposable inventory-v1 fixture; no gameplay result claimed")


if __name__ == "__main__":
    main()
