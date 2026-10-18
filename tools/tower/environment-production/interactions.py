"""Ordinary existing service, denial and departure actions; no progression grants."""
from pathlib import Path
import json,time
from native_session import R,command
from traverse import wait_position
def run():
    session=Path(json.loads((R/'latest-native-session.json').read_text())['session']);out=session/'interactions';out.mkdir(exist_ok=True);report={'session':str(session),'actions':[],'scope':'Existing hooks only. No combat victory, reward eligibility, service completion or gameplay-loop completion is fabricated.'}
    def observe_until(predicate,timeout=20):
        before=time.monotonic()
        while time.monotonic()-before<timeout:
            v=command(session,'observe')
            if predicate(v):return v
            time.sleep(.3)
        raise TimeoutError(('existing interaction outcome',v.get('position'),v.get('messages')))
    def capture(name):
        v=command(session,'capture');frame=session/f'frame-{v["sequence"]}.bmp'
        for _ in range(80):
            if frame.exists():break
            time.sleep(.05)
        from PIL import Image
        Image.open(frame).save(out/(name+'.png'));(out/(name+'.json')).write_text(json.dumps(v,indent=2));return v
    try:
        receipt=command(session,'object 909 10422 115 1');assert receipt['status']=='accepted'
        value=observe_until(lambda v:'run 1'in v.get('journal','').lower());assert len(value['items'])>=6
        capture('plaque-start');report['actions'].append({'hook':'Existing plaque begins run1 and supplies normal tools','position':value['position'],'ordinary_native_action':True,'outcome':'journal run1; at least six tool slots visible'})
        receipt=command(session,'object 18491 10421 124 2');assert receipt['status']=='accepted'
        value=observe_until(lambda v:12 in v.get('modals',[]));capture('bank-open');report['actions'].append({'hook':'Existing bank booth operation2','outcome':'Native bank interface12 opened','position':value['position']});command(session,'close');observe_until(lambda v:12 not in v.get('modals',[]))
        before=command(session,'observe');last=max([m['uid']for m in before.get('message_events',[])]or[-1]);command(session,'object 409 10428 116 1')
        value=observe_until(lambda v:any(m['uid']>last and'prayer'in m['text'].lower()for m in v.get('message_events',[])));capture('altar-pray');report['actions'].append({'hook':'Existing Pray operation','outcome':'Fresh Prayer response observed','position':value['position']})
        before=command(session,'observe');items=before['items'];last=max([m['uid']for m in before.get('message_events',[])]or[-1]);command(session,'object 57694 10423 115 1')
        value=observe_until(lambda v:any(m['uid']>last for m in v.get('message_events',[])));assert value['items']==items;capture('reward-gated');report['actions'].append({'hook':'Existing reward chest before victory','outcome':'Fresh response; inventory unchanged, no reward granted','position':value['position'],'messages':value['messages'][-3:]})
        before=command(session,'observe');saved_items=before['items'];saved_journal=before['journal'];command(session,'object 57692 10432 124 1')
        value=observe_until(lambda v:v['position']==[0,3224,3218],30);capture('portal-outside');assert value['items']==saved_items
        command(session,'object 57692 3225 3218 1');value=observe_until(lambda v:v['position']==[0,10424,120],30);capture('portal-return');assert value['items']==saved_items and value['journal']==saved_journal
        report['actions'].append({'hook':'Existing portal exit and return','outside':[0,3224,3218],'return':value['position'],'inventory_and_journal_preserved':True});command(session,'camera 2000 0');report['all_passed']=True
    finally:(out/'result.json').write_text(json.dumps(report,indent=2)+'\n')
    print('EXISTING_INTERACTIONS_VERIFIED',len(report['actions']),'ordinary hooks; no rewards or victories fabricated',flush=True)
if __name__=='__main__':run()
