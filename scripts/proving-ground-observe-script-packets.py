"""One-time readonly post-send capture: scripted bank menus emit asynchronously."""
import hashlib,json,shutil,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];WORK=ROOT/'work/independent-native-observer';OUT=ROOT/'docs/openrune/verification/independent/client-observer-build'
runtime=WORK/'source/src/live_runtime.rs';manifest=OUT/'source-manifest.json';digest=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
state=json.loads(manifest.read_text(encoding='utf-8'));assert digest(runtime)==state['instrumented_runtime_sha256']
text=runtime.read_text(encoding='utf-8');assert 'PG_SENT_PACKETS' not in text
needle='if matches!(opcode,27|47|95|115) { println!("PG_WIRE_TX tick={} opcode={} payload={:02x?}",client.server_tick,opcode,payload); }';assert text.count(needle)==1
stamp=str(time.time_ns());history=OUT/('script-packet-history-'+stamp);history.mkdir();shutil.copyfile(manifest,history/'source-manifest.json')
private=WORK/'source-history'/history.name;private.mkdir(parents=True);shutil.copyfile(runtime,private/'live_runtime.rs')
text=text.replace('pub struct Connection {','/// Readonly copy after actual successful gameplay transport submission. No login packets.\npub static PG_SENT_PACKETS: std::sync::Mutex<Vec<(i32,u16,Vec<u8>)>> = std::sync::Mutex::new(Vec::new());\npub struct Connection {',1)
text=text.replace(needle,needle[:-2]+' PG_SENT_PACKETS.lock().unwrap().push((client.server_tick,opcode,payload.clone())); }')
runtime.write_text(text,encoding='utf-8',newline='\n');state['instrumented_runtime_sha256']=digest(runtime)
state['owned_overlays']['src/live_runtime.rs']+='; readonly copy of successfully submitted gameplay frames lets the observer correlate asynchronous native-script bank operations. Original dispatch and transport unchanged.'
manifest.write_text(json.dumps(state,indent=2)+'\n',encoding='utf-8',newline='\n');print('Added readonly successful-send gameplay packet capture; bank harness immediate-packet assumption corrected')
