"""Bounded JSONL observations and strict, provenance-aware scenario comparison."""
import argparse, collections, hashlib, json, re, struct
from pathlib import Path

VERSION=1
MAX_RECORD=65536
FORBIDDEN={'password','secret','private_key','credentials','session_key','isaac_seed','rsa_key','token'}
def safe(value):
    if isinstance(value,dict):
        for key,item in value.items():
            if key.lower() in FORBIDDEN: raise ValueError('Sensitive telemetry field rejected')
            safe(item)
    elif isinstance(value,list):
        for item in value:safe(item)
    elif isinstance(value,str) and ('PRIVATE KEY-----' in value or len(value)>16384): raise ValueError('Unsafe telemetry string')
def validate(event):
    required={'schema_version','event_id','run_id','scenario_id','environment','source','timestamp','game_tick','actor_id','kind','observation','data','evidence'}
    if set(event)!=required or event['schema_version']!=VERSION:raise ValueError('Invalid event envelope')
    if event['observation'] not in ('observed','inferred','unavailable'):raise ValueError('Invalid observation provenance')
    if event['game_tick'] is not None and not isinstance(event['game_tick'],int):raise ValueError('Invalid game tick')
    safe(event)
    if len(json.dumps(event).encode())>MAX_RECORD:raise ValueError('Telemetry record too large')
class Writer:
    def __init__(self,path,run,scenario,environment,version):
        self.file=Path(path).open('x',encoding='utf8');self.seq=0
        self.base=dict(schema_version=VERSION,run_id=run,scenario_id=scenario,environment=environment,source={'implementation':'evidence-adapter','version':version},timestamp=None,game_tick=None,actor_id='player:fixture',observation='observed')
    def emit(self,kind,data,evidence,**extra):
        self.seq+=1;e=dict(self.base,event_id=f'{self.base["run_id"]}:{self.seq:06d}',kind=kind,data=data,evidence=evidence);e.update(extra);validate(e)
        self.file.write(json.dumps(e,separators=(',',':'))+'\n');self.file.flush()
    def close(self):self.file.close()
def load(path):
    events=[];ids=set()
    with Path(path).open(encoding='utf8') as f:
        for line in f:
            if len(line.encode())>MAX_RECORD:raise ValueError('Oversized JSONL record')
            e=json.loads(line);validate(e)
            if e['event_id'] in ids:raise ValueError('Duplicate event id')
            ids.add(e['event_id']);events.append(e)
            if len(events)>100000:raise ValueError('Comparison event bound exceeded')
    return events
def state(path):
    value=json.loads(Path(path).read_text(encoding='utf-8-sig'))
    # Session timestamps and account IDs are transport metadata, not gameplay state.
    return {key:value[key] for key in ('x','z','level','varps','run_energy','xp_rate_in_hundreds','stats','inventory') if key in value}
def import_run(run,out,environment,scenario,baseline,fixture,cache,detail='packets'):
    run=Path(run);read=lambda n:json.loads((run/n).read_text(encoding='utf-8-sig')) if (run/n).exists() else {}
    result=read('result.json');shutdown=read('shutdown.json');server=read('server.json')
    w=Writer(out,run.name,scenario,environment,hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
    evidence=lambda name,line=None:{'file':str(run/name),'line':line}
    try:
        w.emit('context',{'baseline_id':baseline,'client_sha256':result.get('clientHash'),'fixture_id':fixture,'cache_id':cache,'server_sha256':server.get('binaryHash'),'input_mode':result.get('mode'),'revision':'239.1'},evidence('result.json'))
        if (run/'before-state.json').exists():w.emit('state.initial',state(run/'before-state.json'),evidence('before-state.json'))
        if (run/'before-tower-state.json').exists():w.emit('tower.initial',read('before-tower-state.json'),evidence('before-tower-state.json'))
        if (run/'client.log').exists():
            with (run/'client.log').open(encoding='utf-8-sig',errors='replace') as lines:
                for number,line in enumerate(lines,1):
                    pos=re.search(r'position=\[([0-9]+), ([0-9]+), ([0-9]+)\]',line);tick=re.search(r'\btick=(\d+)',line)
                    if pos:w.emit('player.position',{'plane':int(pos[1]),'x':int(pos[2]),'z':int(pos[3])},evidence('client.log',number),game_tick=int(tick[1]) if tick else None)
                    menu=re.search(r'Native Continue click at .*handled=(true|false), outbound=\[([^\]]*)\]',line)
                    if menu:w.emit('client.continue',{'handled':menu[1]=='true','outgoing_opcodes':[int(s.strip()) for s in menu[2].split(',') if s.strip()],'server_acceptance':None},evidence('client.log',number))
                    prayer=re.search(r'Native prayer operation: component=(\d+), outbound=\[([\d, ]*)\]',line)
                    if prayer:w.emit('prayer.dispatch',{'component':int(prayer[1]),'operation':1,'outgoing_opcodes':[int(s.strip()) for s in prayer[2].split(',') if s.strip()],'server_acceptance':None},evidence('client.log',number))
                    prayer_continue=re.search(r'Native prayer Continue: outbound=\[([\d, ]*)\]',line)
                    if prayer_continue:w.emit('prayer.continue',{'outgoing_opcodes':[int(s.strip()) for s in prayer_continue[1].split(',') if s.strip()],'server_acceptance':None},evidence('client.log',number))
                    bit=re.search(r'Client observed Thick Skin (?:active|inactive): bit4104=([01])',line)
                    if bit:w.emit('prayer.client_state',{'varbit':4104,'value':int(bit[1]),'server_acceptance':None},evidence('client.log',number))
                    if line.startswith('Observed server prayer requirement notice:'):
                        w.emit('action.effect',{'action':'prayer.requirement_notice','parameters':{'text':line.split(':',1)[1].strip()},'client_dispatch_result':'effect_observed','server_acceptance':None},evidence('client.log',number))
                    if line.startswith('Prayer scenario observed activation, deactivation, requirement notice and native Continue closure'):
                        w.emit('action.effect',{'action':'prayer.notice_closed','parameters':{'interface':229},'client_dispatch_result':'effect_observed','server_acceptance':None},evidence('client.log',number))
                    target=re.search(r'Talk-to man definition (\d+), slot (\d+)',line)
                    if target:w.emit('action.target',{'action':'npc.talk','definition':int(target[1])},evidence('client.log',number))
                    if 'Server NPC response:' in line:w.emit('dialogue.response',{'text':line.split('Server NPC response:',1)[1].strip()},evidence('client.log',number))
                    if 'Server confirmed first Tower arrival: [10424, 120]' in line:
                        w.emit('action.effect',{'action':'tower.first_arrival','parameters':{'position':[0,10424,120]},'client_dispatch_result':'effect_observed','server_acceptance':None},evidence('client.log',number))
                    if 'Server confirmed first-arrival inventory and equipment cleared' in line:
                        w.emit('action.effect',{'action':'tower.first_kit_clear','parameters':{'containers':[93,94]},'client_dispatch_result':'effect_observed','server_acceptance':None},evidence('client.log',number))
                    if 'Native login restored worn sword and its appearance, 1000 coins and two lobsters;' in line:
                        w.emit('action.effect',{'action':'inventory.restore','parameters':{'equipment':{'container':94,'slot':3,'item':1277},'coins':1000,'lobsters':2,'appearance_equipment_3':3325},'client_dispatch_result':'effect_observed','server_acceptance':None},evidence('client.log',number))
                    widget=re.search(r'component=(\d+), child=(-?\d+)',line)
                    if widget:
                        operation=re.search(r'Native interface operation (\d+):',line)
                        sent=re.search(r'outbound=\[([\d, ]*)\]',line)
                        opcodes=[int(part.strip()) for part in sent[1].split(',') if part.strip()] if sent else None
                        w.emit('client.dispatch',{'component':int(widget[1]),'child':int(widget[2]),'operation':int(operation[1]) if operation else None,'outgoing_opcodes':opcodes,'dispatch_result':'packet_queued' if opcodes else 'attempt_logged','server_acceptance':None,'input_path':'native_interface_operation'},evidence('client.log',number))
                    for needle,action in [('Server confirmed player movement','walk'),('Server confirmed floor waypoint','walk'),('Server confirmed equip and appearance','equip'),('Server confirmed unequip','unequip'),('Server confirmed one coin deposited','bank.deposit'),('Server confirmed withdrawal','bank.withdraw'),('Server confirmed final equipped state','equip.persist_requested')]:
                        if needle in line:w.emit('action.effect',{'action':action,'parameters':{'position':list(map(int,re.findall(r'-?\d+',line)))} if action=='walk' else {},'client_dispatch_result':'effect_observed','server_acceptance':None},evidence('client.log',number))
                    if 'ownhidden=true' in line or 'hidden=true' in line:
                        component=re.search(r'(\d+):(\d+)',line)
                        if component:w.emit('interface.hidden',{'interface':int(component[1]),'component':int(component[2]),'hidden':True},evidence('client.log',number))
        if detail=='packets' and (run/'packets.bin').exists():
            with (run/'packets.bin').open('rb') as f:
                if f.read(8)!=b'R239CAP1':raise ValueError('Unknown packet capture format')
                ordinal=0
                while header:=f.read(6):
                    if len(header)!=6:raise ValueError('Truncated packet header')
                    opcode,size=struct.unpack('>HI',header)
                    if size>65535:raise ValueError('Packet bound exceeded')
                    payload=f.read(size)
                    if len(payload)!=size:raise ValueError('Truncated packet payload')
                    ordinal+=1
                    if ordinal>100000:raise ValueError('Capture event bound exceeded')
                    w.emit('packet',{'direction':'server_to_client','opcode':opcode,'size':size,'ordinal':ordinal,'dispatch_result':None},evidence('packets.bin'))
        logout=read('logout-observation.json')
        w.emit('persistence.completion',{'save_completed':logout.get('save_completed')},evidence('logout-observation.json'),observation='observed' if logout else 'unavailable')
        if (run/'state.json').exists():w.emit('state.persisted',state(run/'state.json'),evidence('state.json'))
        if (run/'tower-state.json').exists():w.emit('tower.persisted',read('tower-state.json'),evidence('tower-state.json'))
        w.emit('session.end',{'client_exit':result.get('clientExit'),'server_exit':shutdown.get('exitCode'),'forced':shutdown.get('forced'),'server_exited':shutdown.get('exited')},evidence('shutdown.json'),timestamp=shutdown.get('finishedUtc'))
    finally:w.close()
def compare(a,b):
    contexts=[]
    for events in (a,b):
        c=[e for e in events if e['kind']=='context']
        if len(c)!=1:raise ValueError('Exactly one context is required')
        contexts.append(c[0])
    report={'status':'NOT TESTED','classification':None,'first_divergence':None,'coverage':{},'normalization':['timestamp','run_id','environment','source.version','event_id','actor/session ephemeral identity'],'reproduction':{'kotlin':a[0]['run_id'],'rust':b[0]['run_id']}}
    for key in ('scenario_id',):
        if contexts[0][key]!=contexts[1][key]:return dict(report,status='BLOCKED',classification='scenario mismatch')
    for key in ('baseline_id','client_sha256','fixture_id','cache_id','input_mode','revision'):
        x,y=(c['data'].get(key) for c in contexts)
        if x is None or y is None or x!=y:return dict(report,status='BLOCKED',classification='configuration or provenance mismatch',first_divergence={'field':key,'expected':x,'actual':y,'evidence':[c['evidence'] for c in contexts]})
    completion=[[e for e in rows if e['kind']=='persistence.completion'] for rows in (a,b)]
    if any(len(rows)!=1 or rows[0]['observation']!='observed' or rows[0]['data'].get('save_completed') is not True for rows in completion):
        return dict(report,status='BLOCKED',classification='logout save completion is UNKNOWN or incomplete',persistence_evidence=completion)
    kinds=('state.initial','tower.initial','action.effect','action.target','client.continue','dialogue.response','prayer.dispatch','prayer.continue','prayer.client_state','state.persisted','tower.persisted','session.end','interface.hidden','npc.state','object.state','server.decision')
    for kind in kinds:
        left=[e for e in a if e['kind']==kind];right=[e for e in b if e['kind']==kind]
        report['coverage'][kind]={'kotlin':len(left),'rust':len(right)}
        if not left and not right:continue
    left=[e for e in a if e['kind'] in kinds];right=[e for e in b if e['kind'] in kinds]
    for index in range(max(len(left),len(right))):
        x=left[index] if index<len(left) else None;y=right[index] if index<len(right) else None
        if x is None or y is None or x['kind']!=y['kind'] or x['data']!=y['data']:
            report.update(status='FAIL',classification='observed difference; attribution required',first_divergence={'subsystem':(x or y)['kind'],'ordinal':index,'expected':x,'actual':y});return report
    actions=[[e for e in rows if e['kind']=='action.effect'] for rows in (a,b)]
    report['timing']='NOT TESTED'
    if all(rows and all(e['game_tick'] is not None for e in rows) for rows in actions):
        offsets=[[e['game_tick']-rows[0]['game_tick'] for e in rows] for rows in actions]
        if offsets[0]!=offsets[1]:return dict(report,status='FAIL',classification='relative action timing differs',first_divergence={'subsystem':'timing','expected':offsets[0],'actual':offsets[1]})
        report['timing']='PASS'
    endings=[e['data'] for e in a+b if e['kind']=='session.end']
    if len(endings)!=2 or any(e.get('client_exit')!=0 or e.get('server_exit')!=0 or e.get('forced') is not False for e in endings):
        return dict(report,status='BLOCKED',classification='failed or incomplete lifecycle; equal failures are not parity')
    if not all(report['coverage'][k]['kotlin'] and report['coverage'][k]['rust'] for k in ('state.initial','state.persisted','action.effect')):
        return dict(report,status='BLOCKED',classification='insufficient comparable action/state coverage')
    return dict(report,status='PASS',classification='covered checkpoints match; not complete gameplay parity')
def main():
    p=argparse.ArgumentParser();sub=p.add_subparsers(dest='mode',required=True)
    imp=sub.add_parser('import');imp.add_argument('run');imp.add_argument('output');imp.add_argument('--environment',required=True,choices=['kotlin','rust']);imp.add_argument('--scenario',required=True);imp.add_argument('--baseline',required=True);imp.add_argument('--fixture');imp.add_argument('--cache')
    imp.add_argument('--detail',choices=['checkpoints','packets'],default='packets')
    chk=sub.add_parser('validate');chk.add_argument('input')
    dif=sub.add_parser('compare');dif.add_argument('kotlin');dif.add_argument('rust');dif.add_argument('output')
    args=p.parse_args()
    if args.mode=='import':import_run(args.run,args.output,args.environment,args.scenario,args.baseline,args.fixture,args.cache,args.detail)
    elif args.mode=='validate':print(json.dumps({'status':'PASS','events':len(load(args.input))}))
    else:
        report=compare(load(args.kotlin),load(args.rust));out=Path(args.output)
        with out.open('x',encoding='utf8') as f:json.dump(report,f,indent=2)
        with out.with_suffix('.md').open('x',encoding='utf8') as f:f.write('# Differential observation report\n\n'+report['status']+': '+report['classification']+'\n\n```json\n'+json.dumps(report,indent=2)+'\n```\n')
        print(json.dumps({'status':report['status'],'classification':report['classification']}))
if __name__=='__main__':main()
