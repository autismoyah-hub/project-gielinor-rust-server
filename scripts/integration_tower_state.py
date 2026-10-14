"""Read the arrival-relevant state outside the canonical account snapshot.

Only the disposable integration239 character is read. No state is modified.
This is a narrow projection, not a full Tower diary equivalence assertion.
"""
import argparse,json,re,sqlite3
from contextlib import closing
from pathlib import Path

FLAGS=('needle','vambraces','scimitar','grove','shrimp','minds','potion','arrows','lite','cleared','bones')

def kotlin(directory):
    def positive(name):
        path=directory/name
        if not path.exists():return False
        raw=path.read_text(encoding="utf-8").strip()
        if not re.fullmatch(r"[+-]?\d+",raw):return False
        try:value=int(raw)
        except ValueError:return False
        return 0<value<=2147483647
    return dict(kit_given=(directory/'kit').exists(),staff_taken=(directory/'staff').exists(),
                diary_progress=any((directory/name).exists() for name in FLAGS) or positive('tans') or positive('offers'))

def rust(database):
    with closing(sqlite3.connect(database.resolve().as_uri()+'?mode=ro',uri=True)) as db:
        if not db.execute("SELECT c.id FROM characters c JOIN accounts a ON a.id=c.account_id WHERE a.login_username='integration239' AND c.realm_id=1").fetchone():raise ValueError('Disposable character missing')
        if not db.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='tower_arrival'").fetchone():
            version=db.execute('SELECT max(version) FROM _rust_migrations').fetchone()[0]
            if version is None or not 1<=version<=8:raise ValueError('Missing Tower table outside known pre-V9 schema')
            return dict(kit_given=False,staff_taken=False,diary_progress=False)
        row=db.execute("SELECT t.kit_given,t.staff_taken,t.diary_progress FROM tower_arrival t JOIN characters c ON c.id=t.character_id JOIN accounts a ON a.id=c.account_id WHERE a.login_username='integration239' AND c.realm_id=1").fetchone()
        return dict(zip(('kit_given','staff_taken','diary_progress'),map(bool,row or (0,0,0))))

def main():
    p=argparse.ArgumentParser();p.add_argument('environment',choices=('rust','kotlin'));p.add_argument('source',type=Path);p.add_argument('output',type=Path);a=p.parse_args()
    state=rust(a.source) if a.environment=='rust' else kotlin(a.source)
    with a.output.open('x',encoding='utf8') as f:json.dump(state,f,indent=2)

if __name__=='__main__':main()
