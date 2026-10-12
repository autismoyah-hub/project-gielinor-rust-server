"""Read only normalized state from the disposable integration SQLite databases."""
import argparse
import json
from pathlib import Path
import sqlite3


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("database", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent / "work"
    path = args.database.resolve(strict=True)
    if not (path.is_relative_to(root / "integration") or path.is_relative_to(root / "state/integration-b") or path.is_relative_to(root / "state/integration-floor")):
        raise ValueError("only disposable integration databases may be inspected")
    db = sqlite3.connect(path.as_uri() + "?mode=ro", uri=True)
    db.row_factory = sqlite3.Row
    row = db.execute("""SELECT c.id,c.x,c.z,c.level,c.varps,c.run_energy,
        c.xp_rate_in_hundreds,c.world_id,c.last_login,c.last_logout,a.display_name
        FROM characters c JOIN accounts a ON a.id=c.account_id
        WHERE a.login_username=? AND c.realm_id=1""", ("integration239",)).fetchone()
    if row is None:
        raise ValueError("disposable account has no character")
    result = dict(row)
    character_id = result.pop("id")
    result["varps"] = json.loads(result["varps"])
    result["stats"] = [dict(r) for r in db.execute("SELECT stat_id,vis_level,base_level,fine_xp FROM stats WHERE character_id=? ORDER BY stat_id", (character_id,))]
    result["inventory"] = [dict(r) for r in db.execute("""SELECT i.inv_type,o.slot,o.obj,o.count,o.vars FROM inventories i
        JOIN inventory_objs o ON o.inventories_id=i.id WHERE i.character_id=? ORDER BY i.inv_type,o.slot""", (character_id,))]
    result["session_metadata"] = {k: result.pop(k) for k in ("last_login", "last_logout", "world_id")}
    db.close()
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(f"Recorded disposable state at ({result['x']}, {result['z']}, {result['level']}); {len(result['stats'])} stats, {len(result['inventory'])} items")


if __name__ == "__main__":
    main()
