"""Observe a completed account save before terminating an owned test server.

An already-offline fixture is not proof: last_logout must advance from the
pre-client checkpoint. This observer never writes the account database.
"""
import argparse,json,sqlite3,time
from pathlib import Path

def checkpoint(database):
    with sqlite3.connect(Path(database).resolve().as_uri()+'?mode=ro',uri=True,timeout=1) as db:
        row=db.execute("SELECT c.world_id,c.last_logout FROM characters c JOIN accounts a ON a.id=c.account_id WHERE a.login_username='integration239' AND c.realm_id=1").fetchone()
        if row is None:raise ValueError('Disposable character missing')
        return {'world_id':row[0],'last_logout':row[1]}

def completed(before,after):
    return after['world_id'] is None and after['last_logout'] is not None and after['last_logout']!=before['last_logout']

def wait(database,before,seconds):
    start=time.monotonic();deadline=start+seconds;last=None;error=None
    while True:
        try:
            last=checkpoint(database);error=None
            if completed(before,last):return {'status':'PASS','save_completed':True,'elapsed_seconds':round(time.monotonic()-start,3),'before':before,'after':last}
        except sqlite3.OperationalError as exc:error=type(exc).__name__
        if time.monotonic()>=deadline:break
        time.sleep(min(.25,max(0,deadline-time.monotonic())))
    return {'status':'BLOCKED','save_completed':False,'reason':'No new offline logout save observed before deadline','elapsed_seconds':round(time.monotonic()-start,3),'before':before,'after':last,'database_error':error}

def main():
    p=argparse.ArgumentParser();p.add_argument('mode',choices=['capture','wait']);p.add_argument('database',type=Path);p.add_argument('output',type=Path);p.add_argument('--before',type=Path);p.add_argument('--seconds',type=float,default=40);a=p.parse_args()
    if not 0<=a.seconds<=40:raise ValueError('Logout observation bound must be 0..40 seconds')
    if a.output.exists():raise ValueError('Refusing to overwrite lifecycle evidence')
    if a.mode=='capture':result=checkpoint(a.database)
    else:
        if not a.before:raise ValueError('Pre-client checkpoint required')
        result=wait(a.database,json.loads(a.before.read_text()),a.seconds)
    with a.output.open('x',encoding='utf8') as f:json.dump(result,f,indent=2)
    print(json.dumps({'mode':a.mode,'status':result.get('status','captured')}))
if __name__=='__main__':main()
