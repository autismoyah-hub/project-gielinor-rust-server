const DATA={"prefix": "", "private": false, "views": {"quarter": "Three-quarter", "front": "Front", "side": "Side", "rear": "Rear", "above": "Elevated", "gameplay": "Gameplay scale", "face": "Face detail", "armor": "Armor detail"}};
let view='quarter',mode='native';const keys=['anatomy','face','surfaces','compatibility','overall'];
function update(){const env=view.startsWith('environment-');const detail=['face','armor'].includes(view);if(env||detail)mode='native';
 for(const label of ['A','B']){const path=env?'blind/'+label+'/'+view+'.png':DATA.prefix+'models/'+label+'/'+mode+'/'+view+'.png';const im=document.getElementById('image'+label);im.src=path;im.parentElement.href=path;}
 document.querySelectorAll('.angle').forEach(n=>n.textContent=DATA.views[view]);document.querySelectorAll('[data-view]').forEach(n=>n.classList.toggle('active',n.dataset.view===view));
 document.getElementById('native').classList.toggle('active',mode==='native');document.getElementById('studio').classList.toggle('active',mode==='studio');document.getElementById('studio').disabled=env||detail;
 document.getElementById('condition').textContent=env?'Same real revision239.1 courtyard terrain and scenery, actor origin and ground height; read-only offline submission.':mode==='studio'?'Supplementary Cycles32-sample/AgX lighting, with fixed canonical bounds and equivalent native camera; not native acceptance evidence.':'Unchanged native Rust renderer · ambient72 · contrast850 · light[-30,-50,-30] · palette brightness0.8 · no candidate alignment or individual rescaling.';
}
document.querySelectorAll('[data-view]').forEach(n=>n.onclick=()=>{view=n.dataset.view;update();});for(const m of ['native','studio'])document.getElementById(m).onclick=()=>{mode=m;update();};
function feedback(){return {schema:1,reviewed_at:new Date().toISOString(),ratings:Object.fromEntries(keys.map(k=>[k,document.getElementById(k).value])),notes:document.getElementById('notes').value,artistic_approval:false};}
function save(){try{localStorage.setItem('ashhorn-blind-review',JSON.stringify(feedback()));}catch(e){}}
document.getElementById('download').onclick=()=>{save();let u=URL.createObjectURL(new Blob([JSON.stringify(feedback(),null,2)],{type:'application/json'}));let a=document.createElement('a');a.href=u;a.download='ashhorn-blind-assessment.json';a.click();setTimeout(()=>URL.revokeObjectURL(u),1000);};
document.getElementById('reviewed').onchange=e=>document.getElementById('reveal').disabled=!e.target.checked;
document.getElementById('reveal').onclick=()=>{save();location.href='technical/index.html';};update();
