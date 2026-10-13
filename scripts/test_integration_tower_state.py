import sqlite3,tempfile,unittest
from contextlib import closing
from pathlib import Path
from integration_tower_state import kotlin,rust

class TowerStateTests(unittest.TestCase):
    def test_kit_and_staff_do_not_imply_diary_progress(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)
            for name in ('kit','staff','schema','bounty'):(root/name).write_text('given\n')
            self.assertEqual(kotlin(root),dict(kit_given=True,staff_taken=True,diary_progress=False))
            (root/'bones').write_text('offered\n')
            self.assertTrue(kotlin(root)['diary_progress'])
    def test_only_positive_valid_kotlin_counts_imply_progress(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)
            for count in ('-1','0','2147483648','invalid'):
                (root/'tans').write_text(count)
                self.assertFalse(kotlin(root)['diary_progress'])
            (root/'offers').write_text('1\n')
            self.assertTrue(kotlin(root)['diary_progress'])
    def test_owned_pre_migration_database_has_default_projection(self):
        with tempfile.TemporaryDirectory() as t:
            path=Path(t)/'state.sqlite'
            with closing(sqlite3.connect(path)) as db:
                db.executescript("CREATE TABLE accounts(id INTEGER,login_username TEXT);CREATE TABLE characters(id INTEGER,account_id INTEGER,realm_id INTEGER);INSERT INTO accounts VALUES(1,'integration239');INSERT INTO characters VALUES(1,1,1);CREATE TABLE _rust_migrations(version INTEGER);INSERT INTO _rust_migrations VALUES(8);")
            self.assertEqual(rust(path),dict(kit_given=False,staff_taken=False,diary_progress=False))
            with closing(sqlite3.connect(path)) as db:db.executescript('UPDATE _rust_migrations SET version=9;')
            with self.assertRaises(ValueError):rust(path)

if __name__=='__main__':unittest.main()
