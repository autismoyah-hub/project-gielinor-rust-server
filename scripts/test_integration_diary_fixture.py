import sqlite3
import tempfile
import unittest
from contextlib import closing
from pathlib import Path
from integration_diary_fixture import prepare_kotlin, prepare_rust
from integration_diary_state import kotlin, rust


class DiaryFixture(unittest.TestCase):
    def test_new_clone_upgrade_and_canonical_state_match(self):
        with tempfile.TemporaryDirectory(dir=Path(__file__).resolve().parent.parent/'work/tmp') as temp:
            root=Path(temp);database=root/'state.sqlite';directory=root/'integration239'
            with closing(sqlite3.connect(database)) as db, db:
                db.executescript("""CREATE TABLE accounts(id INTEGER,login_username TEXT);
                    CREATE TABLE characters(id INTEGER PRIMARY KEY,account_id INTEGER,realm_id INTEGER,world_id INTEGER);
                    CREATE TABLE _rust_migrations(version INTEGER PRIMARY KEY);
                    INSERT INTO accounts VALUES(1,'integration239');
                    INSERT INTO characters VALUES(1,1,1,NULL);""")
                db.executemany('INSERT INTO _rust_migrations VALUES(?)',[(n,) for n in range(1,9)])
                prepare_rust(db)
            prepare_kotlin(directory)
            self.assertEqual(rust(database),kotlin(directory))
            self.assertTrue(rust(database)['schema11'])
            with self.assertRaises(FileExistsError):prepare_kotlin(directory)
            with closing(sqlite3.connect(database)) as db, db:
                with self.assertRaises(sqlite3.IntegrityError):prepare_rust(db)
                db.execute('UPDATE characters SET world_id=1')
                with self.assertRaises(ValueError):prepare_rust(db)

    def test_unknown_migration_history_is_rejected_before_mutation(self):
        with closing(sqlite3.connect(':memory:')) as db:
            db.executescript("""CREATE TABLE accounts(id INTEGER,login_username TEXT);
                CREATE TABLE characters(id INTEGER,account_id INTEGER,realm_id INTEGER,world_id INTEGER);
                CREATE TABLE _rust_migrations(version INTEGER);
                INSERT INTO accounts VALUES(1,'integration239');
                INSERT INTO characters VALUES(1,1,1,NULL);
                INSERT INTO _rust_migrations VALUES(8);""")
            with self.assertRaises(ValueError):prepare_rust(db)
            self.assertIsNone(db.execute("SELECT name FROM sqlite_master WHERE name='tower_diary'").fetchone())


if __name__=='__main__':unittest.main()
