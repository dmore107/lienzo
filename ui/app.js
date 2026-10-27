/* ================= PUENTE CON TAURI ================= */
const TAURI=window.__TAURI__;
const invoke=(c,a)=>TAURI?TAURI.core.invoke(c,a):Promise.reject('sin Tauri');
const listen=(e,f)=>TAURI&&TAURI.event.listen(e,f);
const fileSrc=p=>TAURI?TAURI.core.convertFileSrc(p):p;

/* ================= ICONOS ================= */
const I={
  text:'<svg viewBox="0 0 24 24"><path d="M5 6h14M5 12h14M5 18h9"/></svg>',
  image:'<svg viewBox="0 0 24 24"><rect x="4" y="5" width="16" height="14" rx="2.5"/><path d="M4 16l5-5 4 4 3-3 4 4"/></svg>',
  pin:'<svg viewBox="0 0 24 24"><path d="M9 4h6l-1 6 3 3H7l3-3z"/><path d="M12 13v7"/></svg>',
  copy:'<svg viewBox="0 0 24 24"><rect x="8" y="8" width="11" height="11" rx="2"/><path d="M5 15V6a1 1 0 0 1 1-1h9"/></svg>',
  pasteIn:'<svg viewBox="0 0 24 24"><path d="M12 4v10M8 10l4 4 4-4"/><path d="M5 16v3h14v-3"/></svg>',
  trash:'<svg viewBox="0 0 24 24"><path d="M5 7h14M10 7V5h4v2M7 7l1 12h8l1-12"/></svg>',
  search:'<svg viewBox="0 0 24 24"><circle cx="11" cy="11" r="6"/><path d="M20 20l-4.5-4.5"/></svg>',
  left:'<svg viewBox="0 0 24 24"><path d="M14 6l-6 6 6 6"/></svg>',
  right:'<svg viewBox="0 0 24 24"><path d="M10 6l6 6-6 6"/></svg>',
  x:'<svg viewBox="0 0 24 24"><path d="M7 7l10 10M17 7L7 17"/></svg>',
  paste:'<svg viewBox="0 0 24 24"><rect x="6" y="5" width="12" height="15" rx="2"/><path d="M9 5V4h6v1M9 11h6M9 15h4"/></svg>',
  pencil:'<svg viewBox="0 0 24 24"><path d="M5 19l1-4 9.5-9.5a2.1 2.1 0 0 1 3 3L9 18z"/><path d="M14 7l3 3"/></svg>',
  plus:'<svg viewBox="0 0 24 24"><path d="M12 5v14M5 12h14"/></svg>',
  restore:'<svg viewBox="0 0 24 24"><path d="M5 9h10a4 4 0 0 1 0 8H9"/><path d="M8 6L5 9l3 3"/></svg>',
  spell:'<svg viewBox="0 0 24 24"><path d="M3 17l4-10 4 10M4.5 13.5h5"/><path d="M14 15l2.5 2.5L21 12"/></svg>',
  pause:'<svg viewBox="0 0 24 24"><path d="M9 6v12M15 6v12"/></svg>',
  play:'<svg viewBox="0 0 24 24"><path d="M8 5l11 7-11 7z"/></svg>',
  drop:'<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="7.5"/><path d="M12 4.5v15" /><path d="M12 7.5h4.8M12 10.5h6.6M12 13.5h6.6M12 16.5h4.8" stroke-width="1.2"/></svg>',
  shrink:'<svg viewBox="0 0 24 24"><path d="M4 14h6v6M20 10h-6V4M10 14l-6 6M14 10l6-6"/></svg>',
  expand:'<svg viewBox="0 0 24 24"><path d="M14 4h6v6M10 20H4v-6M20 4l-7 7M4 20l7-7"/></svg>',
  sort:'<svg viewBox="0 0 24 24"><path d="M7 5v14M4 16l3 3 3-3M14 7h6M14 12h4M14 17h2"/></svg>',
  check:'<svg viewBox="0 0 24 24"><path d="M5 12.5l4.5 4.5L19 7"/></svg>',
  sliders:'<svg viewBox="0 0 24 24"><path d="M5 7h9M18 7h1M5 17h3M12 17h7"/><circle cx="16" cy="7" r="2"/><circle cx="10" cy="17" r="2"/></svg>',
  mic:'<svg viewBox="0 0 24 24"><rect x="9" y="3.5" width="6" height="11" rx="3"/><path d="M6 11a6 6 0 0 0 12 0M12 17v3.5"/></svg>',
  micOff:'<svg viewBox="0 0 24 24"><path d="M15 10V6.5a3 3 0 0 0-5.6-1.5M9 9v2.5a3 3 0 0 0 4.6 2.5M6 11a6 6 0 0 0 9.6 4.8M18 11a6 6 0 0 1-.5 2.4M12 17v3.5M4 4l16 16"/></svg>',
  session:'<svg viewBox="0 0 24 24"><rect x="9" y="3.5" width="6" height="11" rx="3"/><path d="M6 11a6 6 0 0 0 12 0M12 17v3.5"/></svg>',
  download:'<svg viewBox="0 0 24 24"><path d="M12 4v11M7.5 10.5L12 15l4.5-4.5"/><path d="M5 19h14"/></svg>',
  clock:'<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="7.5"/><path d="M12 8v4.2l2.8 1.8"/></svg>',
  stop:'<svg viewBox="0 0 24 24"><rect x="7" y="7" width="10" height="10" rx="2"/></svg>',
  folder:'<svg viewBox="0 0 24 24"><path d="M4 7.5A1.5 1.5 0 0 1 5.5 6H10l2 2h6.5A1.5 1.5 0 0 1 20 9.5v8a1.5 1.5 0 0 1-1.5 1.5h-13A1.5 1.5 0 0 1 4 17.5z"/></svg>',
  hand:'<svg viewBox="0 0 24 24"><path d="M8 13V6.5a1.5 1.5 0 0 1 3 0V12M11 11V5a1.5 1.5 0 0 1 3 0v6M14 11V6.5a1.5 1.5 0 0 1 3 0V14a6 6 0 0 1-6 6h-1a5 5 0 0 1-4.3-2.5L4 14.5a1.5 1.5 0 0 1 2.5-1.6L8 15"/></svg>'
};
document.querySelectorAll('[data-i]').forEach(e=>e.outerHTML=I[e.dataset.i]);

/* ================= UTILIDADES ================= */
const H=36e5, D=864e5, TKEY='lienzo-tema';
const $=s=>document.querySelector(s);
const ZOOMS=['year','month','day','hour'];
const esc=s=>String(s).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const clamp=(v,a,b)=>Math.max(a,Math.min(b,v));
const cap=s=>s.charAt(0).toUpperCase()+s.slice(1);
const byId=id=>items.find(i=>i.id===id);
const live=()=>items.filter(i=>!i.deletedAt);
const startOfDay=t=>{const d=new Date(t);return new Date(d.getFullYear(),d.getMonth(),d.getDate()).getTime()};
const fmtShort=t=>{const d=new Date(t);return d.toLocaleDateString('es-CO',{day:'numeric',month:'short'}).replace('.','')+' · '+d.toLocaleTimeString('es-CO',{hour:'2-digit',minute:'2-digit'})};
const inside=(el,e)=>{const r=el.getBoundingClientRect();return e.clientX>=r.left&&e.clientX<=r.right&&e.clientY>=r.top&&e.clientY<=r.bottom};
function moveInd(sel,btn){const ind=document.querySelector(sel+' .ind');if(!btn||!ind)return;ind.style.width=btn.offsetWidth+'px';ind.style.transform=`translateX(${btn.offsetLeft-3}px)`}

/* Convierte un elemento del backend al formato de la interfaz */
function fromDb(r){return{id:r.id,type:r.kind,content:r.content,src:r.kind==='image'?fileSrc(r.content):null,thumb:r.thumb?fileSrc(r.thumb):null,note:r.note||'',ts:r.ts,pinned:r.pinned,deletedAt:r.deletedAt??null,x:r.x??null,y:r.y??null,tone:r.tone||'',pinOrder:r.pinOrder??null,source:r.source||''}}
/* Grabaciones: se muestran en el tablero como una tarjeta más; el título vive en content */
const KIND={reunion:'Reunión',capacitacion:'Capacitación'};
const SSTATUS={grabando:'Grabando',grabada:'Audio guardado',en_cola:'En cola',transcribiendo:'Transcribiendo',transcrita:'Transcrita',error:'Error al grabar'};
function fromSession(r){return{id:r.id,type:'session',skind:r.kind,content:r.title||'',ts:r.ts,durationMs:r.durationMs||0,status:r.status,audioPath:r.audioPath,transcript:r.transcript||'',acta:r.acta||'',error:r.error||'',progress:0,deletedAt:r.deletedAt??null,x:r.x??null,y:r.y??null,tone:r.tone||'',pinned:false,note:'',source:''}}
const sessTitle=i=>i.content||(KIND[i.skind]||'Grabación')+' sin título';
const two=n=>String(n).padStart(2,'0');
const fmtDur=ms=>{const t=Math.round(ms/1000),h=Math.floor(t/3600),m=Math.floor(t/60)%60,x=t%60;return h?h+':'+two(m)+':'+two(x):two(m)+':'+two(x)};
const isSess=i=>!!i&&i.type==='session';
/* Nombre legible de la app de origen */
const APPS={chrome:'Chrome',msedge:'Edge',firefox:'Firefox',brave:'Brave',opera:'Opera',winword:'Word',excel:'Excel',powerpnt:'PowerPoint',outlook:'Outlook',olk:'Outlook',onenote:'OneNote',teams:'Teams','ms-teams':'Teams',code:'VS Code',explorer:'Explorador',notepad:'Bloc de notas',windowsterminal:'Terminal',powershell:'PowerShell',pwsh:'PowerShell',cmd:'Símbolo del sistema',whatsapp:'WhatsApp',slack:'Slack',discord:'Discord',notion:'Notion',obsidian:'Obsidian',acrord32:'Acrobat',acrobat:'Acrobat',snippingtool:'Recortes',screenclippinghost:'Recortes',zoom:'Zoom'};
const appName=s=>s?(APPS[s.toLowerCase()]||s):'';
/* Guarda cambios en la base de datos local */
function patch(it,fields){Object.assign(it,fields);if('content' in fields||'note' in fields)delete it._f;invoke(isSess(it)?'update_session':'update_item',{id:it.id,patch:fields}).catch(e=>toast('No se pudo guardar: '+e))}
/* Varios cambios en una sola transacción: [[item,{campos}],…] */
function patchMany(list){
  if(!list.length)return;
  list.filter(([it])=>isSess(it)).forEach(([it,f])=>patch(it,f));
  const rest=list.filter(([it])=>!isSess(it)); if(!rest.length)return;
  rest.forEach(([it,f])=>Object.assign(it,f));
  invoke('update_items',{patches:rest.map(([it,f])=>({id:it.id,patch:f}))}).catch(e=>toast('No se pudo guardar: '+e));
}

/* ================= ESTADO ================= */
let items=[];
const state={zoom:'day',cursor:Date.now(),sel:startOfDay(Date.now()),tab:'pins'};
let lastKey='', fresh=null, zTop=10, paused=false, keepOpen=false;

/* ================= TEMAS ================= */
function setTheme(t){
  document.documentElement.dataset.theme=t;
  document.querySelectorAll('.sw').forEach(s=>s.classList.toggle('on',s.dataset.th===t));
  try{localStorage.setItem(TKEY,t)}catch(e){}
}
$('#themes').addEventListener('click',e=>{const s=e.target.closest('.sw');if(s)setTheme(s.dataset.th)});
let th='light'; try{th=localStorage.getItem(TKEY)||'light'}catch(e){}
if(th==='blue')th='system';
setTheme(th);

/* Colores del sistema (acento de Windows) */
async function loadSystemColors(){
  try{
    const c=await invoke('system_colors'), n=parseInt(c.accent.slice(1),16), r=n>>16&255, g=n>>8&255, b=n&255, k=c.dark?.38:.6;
    const st=document.documentElement.style;
    st.setProperty('--acc',`${r},${g},${b}`);
    st.setProperty('--accD',`${Math.round(r*k)},${Math.round(g*k)},${Math.round(b*k)}`);
    document.documentElement.dataset.tone=(0.2126*r+0.7152*g+0.0722*b)/255>0.62?'light':'dark';
  }catch(e){}
}
window.addEventListener('focus',loadSystemColors);

/* Opacidad */
const AKEY='lienzo-opacidad';
function setAlpha(v){v=clamp(+v||86,30,100);document.documentElement.style.setProperty('--a',(v/100).toFixed(2));$('#opRange').value=v;$('#opVal').textContent=v+'%';try{localStorage.setItem(AKEY,v)}catch(e){}}
let a0=86; try{a0=+localStorage.getItem(AKEY)||86}catch(e){}
setAlpha(a0);
$('#opRange').addEventListener('input',e=>setAlpha(e.target.value));
document.addEventListener('click',e=>{
  const pop=$('#opPop'), b=e.target.closest('[data-op]');
  if(b){const r=b.getBoundingClientRect();pop.style.top=(r.bottom+8)+'px';pop.style.left=Math.max(8,Math.min(innerWidth-pop.offsetWidth-8,r.right-pop.offsetWidth))+'px';pop.classList.toggle('open');return}
  if(!e.target.closest('#opPop'))pop.classList.remove('open');
});

/* ================= TIMELINE ================= */
function getTicks(){
  const c=new Date(state.cursor), y=c.getFullYear(), m=c.getMonth(), out=[];
  if(state.zoom==='hour'){const b=new Date(y,m,c.getDate()).getTime();for(let h=0;h<24;h++)out.push({s:b+h*H,e:b+(h+1)*H,l:String(h).padStart(2,'0')})}
  else if(state.zoom==='day'){const n=new Date(y,m+1,0).getDate();for(let d=1;d<=n;d++)out.push({s:new Date(y,m,d).getTime(),e:new Date(y,m,d+1).getTime(),l:String(d)})}
  else if(state.zoom==='month'){for(let k=0;k<12;k++)out.push({s:new Date(y,k,1).getTime(),e:new Date(y,k+1,1).getTime(),l:new Date(y,k,1).toLocaleDateString('es-CO',{month:'short'}).replace('.','')})}
  else{for(let k=y-5;k<=y+1;k++)out.push({s:new Date(k,0,1).getTime(),e:new Date(k+1,0,1).getTime(),l:String(k)})}
  return out;
}
function range(){const t=getTicks();if(state.sel!=null){const k=t.find(x=>x.s===state.sel);if(k)return[k.s,k.e]}return[t[0].s,t[t.length-1].e]}
function parentLabel(){
  const c=new Date(state.cursor), y=c.getFullYear();
  if(state.zoom==='hour')return c.toLocaleDateString('es-CO',{weekday:'long',day:'numeric',month:'long',year:'numeric'});
  if(state.zoom==='day')return c.toLocaleDateString('es-CO',{month:'long',year:'numeric'});
  if(state.zoom==='month')return String(y);
  return (y-5)+' – '+(y+1);
}
function rangeLabel(){
  if(state.sel==null)return cap(parentLabel());
  const d=new Date(state.sel);
  if(state.zoom==='hour'){const h=d.getHours();return cap(d.toLocaleDateString('es-CO',{weekday:'long',day:'numeric',month:'long'}))+' · '+String(h).padStart(2,'0')+':00 – '+String(h+1).padStart(2,'0')+':00'}
  if(state.zoom==='day'){const t=startOfDay(Date.now());const pre=state.sel===t?'Hoy · ':state.sel===t-D?'Ayer · ':'';return pre+cap(d.toLocaleDateString('es-CO',{weekday:'long',day:'numeric',month:'long'}))}
  if(state.zoom==='month')return cap(d.toLocaleDateString('es-CO',{month:'long',year:'numeric'}));
  return String(d.getFullYear());
}
function renderTimeline(){
  const t=getTicks(), now=Date.now(), counts=t.map(()=>0), a=t[0].s, b=t[t.length-1].e;
  /* Una sola pasada: cada elemento cae en su marca por búsqueda binaria */
  for(const i of items){
    if(i.deletedAt||i.ts<a||i.ts>=b)continue;
    let lo=0,hi=t.length-1;
    while(lo<hi){const m=(lo+hi+1)>>1;if(t[m].s<=i.ts)lo=m;else hi=m-1}
    counts[lo]++;
  }
  const max=Math.max(1,...counts);
  $('#track').innerHTML=t.map((k,i)=>`<button class="tick${state.sel===k.s?' sel':''}${now>=k.s&&now<k.e?' now':''}" data-s="${k.s}" title="${counts[i]} elemento(s)"><span class="bar" style="height:${counts[i]?5+15*counts[i]/max:0}px"></span><span class="lb">${k.l}</span></button>`).join('');
  $("#parent").textContent=cap(parentLabel());
  document.querySelectorAll('#seg button').forEach(b=>b.classList.toggle('on',b.dataset.z===state.zoom));
  moveInd('#seg',document.querySelector(`#seg button[data-z="${state.zoom}"]`));
}
function zoomIn(s){const i=ZOOMS.indexOf(state.zoom);if(i>=3)return;state.cursor=s??state.sel??state.cursor;state.zoom=ZOOMS[i+1];state.sel=null;render()}
function zoomOut(){const i=ZOOMS.indexOf(state.zoom);if(i<=0)return;state.zoom=ZOOMS[i-1];const k=getTicks().find(x=>state.cursor>=x.s&&state.cursor<x.e);state.sel=k?k.s:null;render()}
function shift(dir){
  const c=new Date(state.cursor);
  if(state.zoom==='hour')c.setDate(c.getDate()+dir);
  else if(state.zoom==='day'){c.setDate(1);c.setMonth(c.getMonth()+dir)}
  else if(state.zoom==='month')c.setFullYear(c.getFullYear()+dir);
  else c.setFullYear(c.getFullYear()+7*dir);
  state.cursor=c.getTime();state.sel=null;render();
}
function goToday(){const n=Date.now();state.zoom='day';state.cursor=n;state.sel=startOfDay(n)}

/* ================= VISTA COMPACTA ================= */
let compact=false;
const pad=n=>String(n).padStart(2,'0');
const hourStart=t=>{const d=new Date(t);d.setMinutes(0,0,0);return d.getTime()};
function goNowHour(){const n=Date.now();state.zoom='hour';state.cursor=n;state.sel=hourStart(n)}
function goHome(){compact?goNowHour():goToday()}
function compactLabel(){
  const d=new Date(state.sel??state.cursor), t=startOfDay(Date.now()), ds=startOfDay(d.getTime());
  const day=ds===t?'Hoy':ds===t-D?'Ayer':cap(d.toLocaleDateString('es-CO',{weekday:'short',day:'numeric',month:'short'}).replace(/\./g,''));
  if(state.zoom!=='hour'||state.sel==null)return cap(rangeLabel());
  const h=d.getHours();return `${day} · ${pad(h)}:00–${pad((h+1)%24)}:00`;
}
function updateCompact(){
  const c=innerWidth<700; if(c===compact)return;
  compact=c; document.body.classList.toggle('compact',c); document.body.classList.remove('show-pins');
  $('#cPins').classList.remove('on'); goHome(); lastKey=''; render();
}
function shiftHour(dir){const s=(state.zoom==='hour'&&state.sel!=null?state.sel:hourStart(state.cursor))+dir*H;state.zoom='hour';state.sel=s;state.cursor=s;render()}
function setCompactPref(v){try{localStorage.setItem('lienzo-compacto',v?'1':'0')}catch(e){}invoke('set_compact',{value:v})}
$('#cPrev').onclick=()=>shiftHour(-1);
$('#cNext').onclick=()=>shiftHour(1);
$('#cNew').onclick=()=>openEditor('new');
$('#cSearch').onclick=()=>openSearch();
$('#cHide').onclick=()=>invoke('hide_window');
$('#cPins').onclick=()=>{const on=document.body.classList.toggle('show-pins');$('#cPins').classList.toggle('on',on)};
$('#cExpand').onclick=()=>setCompactPref(false);
$('#bCompact').onclick=()=>setCompactPref(true);
const track=$('#track');
track.addEventListener('click',e=>{const t=e.target.closest('.tick');if(!t)return;const s=+t.dataset.s;state.sel=state.sel===s?null:s;render()});
track.addEventListener('dblclick',e=>{const t=e.target.closest('.tick');if(t)zoomIn(+t.dataset.s)});
let wheelLock=0;
track.addEventListener('wheel',e=>{e.preventDefault();const n=Date.now();if(n-wheelLock<320)return;wheelLock=n;if(e.deltaY<0){const t=e.target.closest('.tick');zoomIn(t?+t.dataset.s:undefined)}else zoomOut()},{passive:false});
track.addEventListener('mousemove',e=>{track.querySelectorAll('.tick').forEach(el=>{const r=el.getBoundingClientRect(),d=e.clientX-(r.left+r.width/2);el.style.setProperty('--m',Math.exp(-(d*d)/(2*55*55)).toFixed(3))})});
track.addEventListener('mouseleave',()=>track.querySelectorAll('.tick').forEach(el=>el.style.setProperty('--m',0)));
$('#prev').onclick=()=>shift(-1);
$('#next').onclick=()=>shift(1);
$('#seg').addEventListener('click',e=>{const b=e.target.closest('button');if(!b)return;state.zoom=b.dataset.z;state.sel=null;render()});

/* ================= LIENZO ================= */
function sessionInfo(i){
  const st=i.status;
  if(st==='grabando')return '<span class="sbadge"><span class="rdot"></span>Grabando</span>';
  if(st==='transcribiendo')return `<span class="tbar"><i style="transform:scaleX(${(i.progress||0)/100})"></i></span><span class="tpct">${i.progress||0}%</span><button class="sact" data-a="tcancel" title="Cancelar">${I.x}</button>`;
  if(st==='en_cola')return `<span class="sbadge">En cola</span><button class="sact" data-a="tcancel" title="Quitar de la cola">${I.x}</button>`;
  if(st==='transcrita')return `<span class="sbadge">${i.acta?'Con acta':'Transcrita'}</span><span>${fmtDur(i.durationMs)}</span>`;
  if(st==='error')return '<span class="sbadge st-error">Error al grabar</span>';
  return (i.error?`<span class="sbadge st-error" title="${esc(i.error)}">No se transcribió</span>`:`<span>${fmtDur(i.durationMs)}</span>`)
    +`<button class="sact txt" data-a="transcribe">${i.error?'Reintentar':'Transcribir'}</button>`;
}
function sessionCardHTML(i,anim,k){
  const rec=i.status==='grabando';
  const prev=i.status==='transcrita'&&i.transcript?`<p class="tprev">${esc(i.transcript.replace(/\[[0-9:]+\]\s*/g,'').slice(0,240))}</p>`:'';
  return `<div class="card session${anim?' pop':''}${rec?' live':''}" data-id="${i.id}"${i.tone?` data-tone="${esc(i.tone)}"`:''} style="left:${i.x}px;top:${i.y}px;animation-delay:${anim?Math.min(k,20)*40:0}ms">
    <div class="meta">${I.session}<span class="tdot"></span><span title="${fmtShort(i.ts)}">${esc(KIND[i.skind]||'Grabación')} · ${new Date(i.ts).toLocaleTimeString('es-CO',{hour:'2-digit',minute:'2-digit'})}</span></div>
    <div class="body"><p class="stitle">${esc(sessTitle(i))}</p>${prev}<div class="sinfo">${sessionInfo(i)}</div></div>
    <div class="acts"><button data-a="view" title="Abrir">${I.text}</button><button data-a="edit" title="Renombrar">${I.pencil}</button><button data-a="del" title="Mover a la papelera">${I.trash}</button></div>
  </div>`;
}
function cardHTML(i,anim,k){
  if(isSess(i))return sessionCardHTML(i,anim,k);
  const body=i.type==='image'
    ?`<img src="${i.thumb||i.src}" alt="" draggable="false" loading="lazy">${i.note?`<p class="note">${esc(i.note)}</p>`:''}`
    :`<p>${esc(i.content)}</p>`;
  const app=appName(i.source);
  return `<div class="card${anim?' pop':''}${i.pinned?' pinned':''}" data-id="${i.id}"${i.tone?` data-tone="${esc(i.tone)}"`:''} style="left:${i.x}px;top:${i.y}px;animation-delay:${anim?Math.min(k,20)*40:0}ms">
    <div class="meta">${I[i.type]}<span class="tdot"></span><span>${fmtShort(i.ts)}</span>${app?`<span class="src" title="Copiado desde ${esc(app)}">${esc(app)}</span>`:''}<span class="pm">${I.pin}</span></div>
    <div class="body">${body}</div>
    <div class="acts"><button data-a="paste" title="Pegar en la app anterior">${I.pasteIn}</button><button data-a="copy" title="Copiar">${I.copy}</button><button data-a="edit" title="Editar">${I.pencil}</button><button data-a="pin" title="Fijar / quitar">${I.pin}</button><button data-a="del" title="Mover a la papelera">${I.trash}</button></div>
  </div>`;
}
const GAP=16, CW=220;
/* Modos de orden del tablero */
const SORTS={
  'time-asc':{l:'Hora · más antiguo primero',s:'Hora ↑'},
  'time-desc':{l:'Hora · más reciente primero',s:'Hora ↓'},
  'az':{l:'Alfabético · A → Z',s:'A → Z'},
  'za':{l:'Alfabético · Z → A',s:'Z → A'},
  'free':{l:'Modo libre',s:'Libre'}
};
let sortMode='time-asc'; try{sortMode=localStorage.getItem('lienzo-orden')||'time-asc'}catch(e){}
if(!SORTS[sortMode])sortMode='time-asc';
const sortKey=i=>(i.type==='image'?(i.note||''):i.content).trim();
function cmp(a,b){
  const m=compact&&sortMode==='free'?'time-desc':sortMode;
  if(m==='time-desc')return b.ts-a.ts;
  if(m==='az'||m==='za'){
    const A=sortKey(a),B=sortKey(b);
    if(!A!==!B)return !A?1:-1;              // lo que no tiene texto va al final
    const r=A.localeCompare(B,'es',{sensitivity:'base',numeric:true});
    return m==='az'?r:-r;
  }
  return a.ts-b.ts;
}
function setSort(m,quiet){
  sortMode=m; try{localStorage.setItem('lienzo-orden',m)}catch(e){}
  paintTidy(); renderCanvasKeep(true);
  if(!quiet)toast(m==='free'?'Modo libre: acomoda las notas donde quieras':'Ordenado: '+SORTS[m].l.toLowerCase());
}
function paintTidy(){$('#tidy').innerHTML=(sortMode==='free'?I.hand:I.sort)+SORTS[sortMode].s}
function renderCanvas(){
  const [a,b]=range(), board=$('#board'), W=board.clientWidth;
  const vis=live().filter(i=>i.ts>=a&&i.ts<b).sort(cmp);
  const key=a+'-'+b, anim=key!==lastKey; lastKey=key;
  const free=sortMode==='free';
  $('#layer').innerHTML=vis.map((i,k)=>{
    const own=free&&i.x!=null;
    return cardHTML({...i,x:own?clamp(i.x,0,Math.max(0,W-CW-10)):0,y:own?Math.max(0,i.y):0},anim||i.id===fresh,anim?k:0);
  }).join('');
  vis.forEach(i=>{if(!(free&&i.x!=null)){const c=layer.querySelector(`.card[data-id="${i.id}"]`);if(c)c.dataset.auto='1'}});
  layoutAuto(false);
  fresh=null;
  $('#empty').classList.toggle('show',!vis.length);
  paintTidy();
  $('#rlabel').textContent=rangeLabel();
  $('#count').textContent=vis.length+(vis.length===1?' elemento':' elementos');
  $('#clabel').textContent=compactLabel(); $('#clabel').title=$('#count').textContent;
}
/* Re-render sin animación de entrada; con 'animate' las tarjetas se deslizan a su nuevo lugar */
function renderCanvasKeep(animate){
  const prev={}; layer.querySelectorAll('.card').forEach(c=>prev[c.dataset.id]=[c.style.left,c.style.top]);
  lastKey=range().join('-'); renderCanvas();
  if(!animate||compact)return;
  const cards=[...layer.querySelectorAll('.card')], next=cards.map(c=>[c.style.left,c.style.top]);
  cards.forEach(c=>{const p=prev[c.dataset.id];if(p){c.classList.add('place');c.style.left=p[0];c.style.top=p[1]}});
  requestAnimationFrame(()=>requestAnimationFrame(()=>cards.forEach((c,k)=>{c.classList.remove('place');c.style.left=next[k][0];c.style.top=next[k][1]})));
}
function estH(c){const it=byId(c.dataset.id);if(!it)return 80;if(isSess(it))return 104+(it.status==='transcrita'&&it.transcript?78:0);if(it.type==='image')return 190+(it.note?36:0);return 58+Math.min(6,Math.ceil(it.content.length/30))*19}
/* Tablero ordenado en filas */
function layoutAuto(animate){
  const L=$('#layer');
  if(compact){L.style.height='';return}
  const W=$('#board').clientWidth||($('.stage').clientWidth-28)||900;
  const colW=CW+GAP, cols=Math.max(1,Math.floor((W-GAP)/colW));
  const x0=Math.max(GAP,(W-cols*colW+GAP)/2);
  const cards=[...L.querySelectorAll('.card')];
  /* Filas de izquierda a derecha: cada fila mide lo que su tarjeta más alta */
  let col=0,top=GAP,rowH=0;
  cards.forEach(c=>{
    if(c.dataset.auto!=='1')return;
    if(col===cols){col=0;top+=rowH+GAP;rowH=0}
    if(!animate)c.classList.add('place');
    c.style.left=(x0+col*colW)+'px'; c.style.top=top+'px';
    rowH=Math.max(rowH,c.offsetHeight||estH(c)); col++;
  });
  let bottom=top+rowH+GAP;
  cards.forEach(c=>{if(c.dataset.auto!=='1')bottom=Math.max(bottom,parseFloat(c.style.top)+(c.offsetHeight||estH(c))+GAP)});
  L.style.height=bottom+'px';
  if(!animate)requestAnimationFrame(()=>cards.forEach(c=>c.classList.remove('place')));
}
let relayoutT;
function relayout(){clearTimeout(relayoutT);relayoutT=setTimeout(()=>layoutAuto(true),60)}
/* Vuelve a medir cuando la ventana aparece, cambia de tamaño o terminan de cargar imágenes */
try{new ResizeObserver(()=>relayout()).observe($('#board'))}catch(e){}
document.addEventListener('visibilitychange',()=>{if(!document.hidden)setTimeout(()=>layoutAuto(false),80)});
$('#layer').addEventListener('load',()=>relayout(),true);
$('#tidy').onclick=e=>{const r=e.currentTarget.getBoundingClientRect();openCtx(r.left,r.bottom+6,sortMenu())};
function resetFree(){
  const [a,b]=range();
  patchMany(live().filter(i=>i.ts>=a&&i.ts<b&&i.x!=null).map(i=>[i,{x:null,y:null}]));
  renderCanvasKeep(true); toast('Tablero acomodado');
}
function refreshStill(){lastKey=range().join('-');renderTimeline();renderCanvas();renderSide()}

/* Arrastrar tarjetas */
const layer=$('#layer'), side=$('#side'); let drag=null;
layer.addEventListener('pointerdown',e=>{
  if(compact){if(!e.target.closest('#side'))document.body.classList.remove('show-pins'),$('#cPins').classList.remove('on');return}
  const c=e.target.closest('.card'); if(!c||e.target.closest('button')||e.button!==0)return;
  const it=byId(c.dataset.id), bd=$('#board'), br=bd.getBoundingClientRect(), x=parseFloat(c.style.left), y=parseFloat(c.style.top);
  drag={c,it,dx:e.clientX-br.left-x,dy:e.clientY-br.top+bd.scrollTop-y,ox:x,oy:y,x,y,moved:false};
  c.setPointerCapture(e.pointerId); c.style.zIndex=++zTop;
});
layer.addEventListener('pointermove',e=>{
  if(!drag)return; const bd=$('#board'), br=bd.getBoundingClientRect();
  const nx=e.clientX-br.left-drag.dx, ny=e.clientY-br.top+bd.scrollTop-drag.dy;
  if(!drag.moved&&Math.hypot(nx-drag.ox,ny-drag.oy)<4)return;
  if(!drag.moved){drag.moved=true;drag.c.classList.remove('pop');drag.c.classList.add('drag');document.body.classList.add('dragging')}
  drag.x=nx;drag.y=ny;drag.c.style.left=nx+'px';drag.c.style.top=ny+'px';
  side.classList.toggle('over',inside(side,e));
});
function endDrag(e){
  if(!drag)return; const {c,it}=drag;
  c.classList.remove('drag'); document.body.classList.remove('dragging');
  const over=drag.moved&&inside(side,e); side.classList.remove('over');
  if(over){
    c.style.left=drag.ox+'px'; c.style.top=drag.oy+'px';
    if(state.tab==='trash'){drag=null;trashItem(it,c);return}
    pinIt(it,true);if(!isSess(it)){c.classList.add('pinned');toast('Fijado')}
  }else if(drag.moved){
    const br=$('#board').getBoundingClientRect();
    const x=clamp(drag.x,0,br.width-CW-10), y=Math.max(0,drag.y);
    if(sortMode!=='free'){
      /* Al mover una nota a mano se pasa a modo libre: todas conservan el lugar que tienen ahora */
      const keep=[];
      layer.querySelectorAll('.card').forEach(o=>{const t=byId(o.dataset.id);delete o.dataset.auto;if(t&&t!==it)keep.push([t,{x:parseFloat(o.style.left),y:parseFloat(o.style.top)}])});
      patchMany(keep);
      sortMode='free';try{localStorage.setItem('lienzo-orden','free')}catch(e){}
      paintTidy();toast('Modo libre activado');
    }
    patch(it,{x,y});c.style.left=x+'px';c.style.top=y+'px';delete c.dataset.auto;
    layoutAuto(true);
  }
  drag=null;
}
layer.addEventListener('pointerup',endDrag);
layer.addEventListener('pointercancel',endDrag);
layer.addEventListener('dblclick',e=>{const c=e.target.closest('.card');if(!c||e.target.closest('button'))return;if(isSess(byId(c.dataset.id)))openViewer(c.dataset.id);else openEditor(c.dataset.id)});
layer.addEventListener('click',e=>{
  const b=e.target.closest('button[data-a]'); if(!b)return;
  const c=b.closest('.card'), it=byId(c.dataset.id), a=b.dataset.a;
  if(a==='paste')pasteItem(it);
  if(a==='copy')copyItem(it);
  if(a==='edit')openEditor(it.id);
  if(a==='folder')openFolder(it);
  if(a==='view')openViewer(it.id);
  if(a==='transcribe')startTranscribe(it);
  if(a==='tcancel')invoke('cancel_transcription',{id:it.id});
  if(a==='pin'){pinIt(it,!it.pinned);c.classList.toggle('pinned',it.pinned);toast(it.pinned?'Fijado':'Quitado de fijados')}
  if(a==='del')trashItem(it,c);
});

/* ================= PAPELERA ================= */
async function killItem(it){
  try{await invoke(isSess(it)?'delete_session':'delete_forever',{id:it.id});items=items.filter(x=>x!==it);renderSide();toast('Eliminado definitivamente')}
  catch(err){toast('Error: '+err)}
}
async function openFolder(it){try{await invoke('open_session_folder',{id:it.id})}catch(e){toast(''+e)}}
function trashItem(it,c){
  if(isSess(it)&&it.status==='grabando'){toast('Detén la grabación antes de eliminarla');return}
  const go=()=>{patch(it,{deletedAt:Date.now()});refreshStill();toast('Movido a la papelera','Deshacer',()=>{patch(it,{deletedAt:null});fresh=it.id;refreshStill()})};
  if(c){c.classList.add('out');setTimeout(go,240)}else go();
}
$('#emptyTrash').onclick=async()=>{
  try{const n=await invoke('empty_trash');items=items.filter(i=>!i.deletedAt);renderSide();toast(n+' elemento(s) eliminados definitivamente')}
  catch(e){toast('Error: '+e)}
};

/* ================= PANEL LATERAL ================= */
function renderSide(){
  const pins=pinnedList();
  const trash=items.filter(i=>i.deletedAt).sort((a,b)=>b.deletedAt-a.deletedAt);
  $('#pinCount').textContent=pins.length; $('#trashCount').textContent=trash.length;
  document.querySelectorAll('#tabs button').forEach(b=>b.classList.toggle('on',b.dataset.t===state.tab));
  moveInd('#tabs',document.querySelector(`#tabs button[data-t="${state.tab}"]`));
  const isT=state.tab==='trash', list=isT?trash:pins;
  $('#sideList').innerHTML=list.map(i=>{
    const prev=i.type==='image'?`<img src="${i.thumb||i.src}" alt="" draggable="false" loading="lazy">`:isSess(i)?`<p>${esc(sessTitle(i))}</p>`:`<p>${esc(i.content)}</p>`;
    const acts=isT
      ?`<button data-a="restore" title="Restaurar">${I.restore}</button><button data-a="kill" title="Eliminar definitivamente">${I.x}</button>`
      :`<button data-a="copy" title="Copiar">${I.copy}</button><button data-a="unpin" title="Quitar de fijados">${I.x}</button>`;
    return `<div class="pin${isT?' trashed':''}" data-id="${i.id}"${i.tone?` data-tone="${i.tone}"`:''} ${isT?'':'title="Clic para pegar · arrastra para reordenar"'}>${isT?'':'<span class="grip"></span>'}${prev}<span class="pdate">${isT?'Eliminado '+fmtShort(i.deletedAt):fmtShort(i.ts)}</span><div class="pacts">${acts}</div></div>`;
  }).join('');
  $('#sideEmpty').textContent=isT?'La papelera está vacía. Lo que elimines queda aquí por 30 días por si lo necesitas.':'Arrastra aquí lo que quieras tener siempre a mano, sin importar el día.';
  $('#sideEmpty').style.display=list.length?'none':'flex';
  $('#sfoot').classList.toggle('show',isT&&trash.length>0);
}
$('#tabs').addEventListener('click',e=>{const b=e.target.closest('button');if(!b)return;state.tab=b.dataset.t;renderSide()});
$('#sideList').addEventListener('click',async e=>{
  const row=e.target.closest('.pin'); if(!row)return; const it=byId(row.dataset.id);
  const a=e.target.closest('button[data-a]')?.dataset.a;
  if(a==='unpin'){pinIt(it,false);const c=document.querySelector(`.card[data-id="${it.id}"]`);if(c)c.classList.remove('pinned');renderSide();return}
  if(a==='copy'){copyItem(it);return}
  if(a==='restore'){patch(it,{deletedAt:null});fresh=it.id;refreshStill();toast('Restaurado');return}
  if(a==='kill'){killItem(it);return}
  if(state.tab==='pins'&&!pinDragged)pasteItem(it);
});

/* ---- Fijados: orden propio y arrastrar para reordenar ---- */
function pinnedList(){return live().filter(i=>i.pinned).sort((a,b)=>(a.pinOrder??-1e15)-(b.pinOrder??-1e15)||b.ts-a.ts)}
function pinIt(it,v){
  if(v&&isSess(it)){toast('Las grabaciones no se fijan: búscalas en la línea de tiempo o con Ctrl+K');return}
  if(v){const m=Math.min(0,...live().filter(i=>i.pinned&&i.pinOrder!=null).map(i=>i.pinOrder));patch(it,{pinned:true,pinOrder:m-1})}
  else patch(it,{pinned:false});
  renderSide();
}
let pd=null, pinDragged=false;
const plist=$('#sideList');
plist.addEventListener('pointerdown',e=>{
  if(state.tab!=='pins'||e.button!==0||e.target.closest('button'))return;
  const row=e.target.closest('.pin'); if(!row)return;
  pd={row,y0:e.clientY,on:false,id:e.pointerId}; pinDragged=false;
});
plist.addEventListener('pointermove',e=>{
  if(!pd)return; const dy=e.clientY-pd.y0;
  if(!pd.on){if(Math.abs(dy)<5)return;pd.on=true;pinDragged=true;pd.row.setPointerCapture(pd.id);pd.row.classList.add('lift');plist.classList.add('sorting')}
  pd.row.style.transform=`translateY(${dy}px) scale(1.03)`;
  const rows=[...plist.querySelectorAll('.pin')], me=pd.row, r=me.getBoundingClientRect(), mid=r.top+r.height/2;
  const idx=rows.indexOf(me);
  const prev=rows[idx-1], next=rows[idx+1];
  if(next){const nr=next.getBoundingClientRect();if(mid>nr.top+nr.height/2){const h=nr.height+8;plist.insertBefore(next,me);pd.y0+=h;me.style.transform=`translateY(${e.clientY-pd.y0}px) scale(1.03)`;flip(next,-h)}}
  if(prev){const pr=prev.getBoundingClientRect();if(mid<pr.top+pr.height/2){const h=pr.height+8;plist.insertBefore(me,prev);pd.y0-=h;me.style.transform=`translateY(${e.clientY-pd.y0}px) scale(1.03)`;flip(prev,h)}}
});
function flip(el,dy){el.style.transition='none';el.style.transform=`translateY(${-dy}px)`;requestAnimationFrame(()=>{el.style.transition='';el.style.transform=''})}
function endPin(){
  if(!pd)return; const {row,on}=pd; pd=null;
  if(!on)return;
  row.classList.remove('lift'); row.style.transform=''; plist.classList.remove('sorting');
  patchMany([...plist.querySelectorAll('.pin')].map((r,k)=>[byId(r.dataset.id),{pinOrder:k}]).filter(([it,f])=>it&&it.pinOrder!==f.pinOrder));
  setTimeout(()=>pinDragged=false,50);
}
plist.addEventListener('pointerup',endPin);
plist.addEventListener('pointercancel',endPin);
$('#sideList').addEventListener('dblclick',e=>{const row=e.target.closest('.pin');if(row&&state.tab==='pins')openEditor(row.dataset.id)});

/* ================= COPIAR Y PEGAR ================= */
async function copyItem(it){try{await invoke('copy_item',{id:it.id,paste:false});toast('Copiado al portapapeles')}catch(e){toast('No se pudo copiar: '+e)}}
async function pasteItem(it){try{await invoke('copy_item',{id:it.id,paste:true})}catch(e){toast('No se pudo pegar: '+e)}}

/* ================= NUEVOS ELEMENTOS ================= */
function addLocal(r,pos){
  const old=byId(r.id);
  if(old){
    /* Ya existía: el backend lo trajo al presente en lugar de duplicarlo */
    if(old.ts===r.ts&&!old.deletedAt)return;
    Object.assign(old,{ts:r.ts,deletedAt:null,source:r.source||old.source});
    const [a,b]=range(); if(!(old.ts>=a&&old.ts<b))goHome();
    fresh=old.id; render();
    if(!compact){const c=layer.querySelector(`.card[data-id="${old.id}"]`);if(c)c.scrollIntoView({block:'nearest',behavior:'smooth'})}
    return;
  }
  const it=fromDb(r), [a,b]=range();
  if(!(it.ts>=a&&it.ts<b))goHome();
  items.push(it);
  if(pos&&!compact)patch(it,{x:Math.max(0,pos.x),y:Math.max(0,pos.y)});
  fresh=it.id; renderTimeline(); renderCanvas(); renderSide();
  if(!compact){const c=layer.querySelector(`.card[data-id="${it.id}"]`);if(c)c.scrollIntoView({block:'nearest',behavior:'smooth'})}
}
listen('item-added',ev=>addLocal(ev.payload));

/* Soltar imágenes desde el Explorador */
const board=$('#board');
board.addEventListener('dragover',e=>{e.preventDefault();board.classList.add('dropping')});
board.addEventListener('dragleave',()=>board.classList.remove('dropping'));
board.addEventListener('drop',e=>{
  e.preventDefault();board.classList.remove('dropping');const r0=board.getBoundingClientRect(),br={left:r0.left,top:r0.top-board.scrollTop};
  [...e.dataTransfer.files].filter(f=>f.type.startsWith('image/')).forEach((f,k)=>{
    const r=new FileReader();
    r.onload=async()=>{try{const it=await invoke('create_image',{dataUrl:r.result});addLocal(it,{x:e.clientX-br.left-110+k*20,y:e.clientY-br.top-60+k*20});toast('Imagen guardada')}catch(err){toast('No se pudo guardar la imagen')}};
    r.readAsDataURL(f);
  });
  const t=e.dataTransfer.getData('text/plain');
  if(t&&t.trim()&&!e.dataTransfer.files.length)invoke('create_text',{content:t.trim()}).then(it=>{addLocal(it,{x:e.clientX-br.left-110,y:e.clientY-br.top-40});toast('Texto guardado')});
});

/* ================= EDITOR + REDACCIÓN ================= */
let editing=null;
function openEditor(id){
  editing=id; const ta=$('#etext');
  if(id==='new'){
    $('#etitle').textContent='Nueva nota';$('#edate').textContent='';$('#eimg').innerHTML='';ta.value='';ta.placeholder='Escribe aquí…';
  }else{
    const it=byId(id);
    $('#edate').textContent='· '+fmtShort(it.ts);
    if(isSess(it)){const k=(KIND[it.skind]||'Grabación').toLowerCase();$('#etitle').textContent='Renombrar '+k;$('#eimg').innerHTML='';ta.value=it.content;ta.placeholder='Título de la '+k+'…'}
    else if(it.type==='image'){$('#etitle').textContent='Imagen';$('#eimg').innerHTML=`<img src="${it.src}" alt="">`;ta.value=it.note||'';ta.placeholder='Agrega una descripción a la imagen…'}
    else{$('#etitle').textContent='Editar nota';$('#eimg').innerHTML='';ta.value=it.content;ta.placeholder='Escribe aquí…'}
  }
  $('#sugs').classList.remove('show');$('#sugs').innerHTML='';
  $('#editor').classList.add('open');setTimeout(()=>{ta.focus();ta.setSelectionRange(ta.value.length,ta.value.length)},60);
}
function closeEditor(){$('#editor').classList.remove('open');editing=null}
async function saveEditor(){
  const v=$('#etext').value;
  if(editing==='new'){
    closeEditor();
    if(v.trim()){try{const it=await invoke('create_text',{content:v.trim()});addLocal(it);toast('Nota creada')}catch(e){toast('Error: '+e)}}
    return;
  }
  const it=byId(editing);
  if(isSess(it))patch(it,{content:v.trim()});
  else if(it.type==='image')patch(it,{note:v.trim()});
  else{if(!v.trim()){toast('La nota está vacía');return}patch(it,{content:v})}
  closeEditor();refreshStill();toast('Cambios guardados');
}
$('#eclose').onclick=closeEditor;$('#ecancel').onclick=closeEditor;$('#esave').onclick=saveEditor;
$('#editor').addEventListener('click',e=>{if(e.target.id==='editor')closeEditor()});
$('#newNote').onclick=()=>openEditor('new');

/* Reglas de redacción en español (local, sin internet) */
const DICT={tambien:'también',despues:'después',ademas:'además',aqui:'aquí',asi:'así',facil:'fácil',dificil:'difícil',rapido:'rápido',codigo:'código',informacion:'información',configuracion:'configuración',reunion:'reunión',sesion:'sesión',numero:'número',pagina:'página',telefono:'teléfono',ultimo:'último',analisis:'análisis',solucion:'solución',version:'versión',aplicacion:'aplicación',busqueda:'búsqueda',dia:'día',menu:'menú',contraseña:'contraseña',tmb:'también',xq:'porque',pq:'porque',q:'que'};
const wordRe=w=>new RegExp('(?<!\\p{L})'+w+'(?!\\p{L})','giu');
function matchCase(src,rep){return src[0]===src[0].toUpperCase()&&src[0]!==src[0].toLowerCase()?cap(rep):rep}
function getRules(t){
  const r=[];
  if(/ {2,}/.test(t))r.push({l:'Hay espacios dobles',f:s=>s.replace(/ {2,}/g,' ')});
  if(/[ \t]+[,.;:?!]/.test(t))r.push({l:'Sobra un espacio antes de un signo de puntuación',f:s=>s.replace(/[ \t]+([,.;:?!])/g,'$1')});
  if(/[,;](?=\p{L})/u.test(t))r.push({l:'Falta un espacio después de coma',f:s=>s.replace(/([,;])(?=\p{L})/gu,'$1 ')});
  const rep=t.match(/(?<!\p{L})(\p{L}+)\s+\1(?!\p{L})/iu);
  if(rep)r.push({l:`Palabra repetida: “${rep[1]} ${rep[1]}”`,f:s=>s.replace(/(?<!\p{L})(\p{L}+)\s+\1(?!\p{L})/giu,'$1')});
  if(/(^|[.!?]\s+)\p{Ll}/u.test(t))r.push({l:'Una oración empieza en minúscula',f:s=>s.replace(/(^|[.!?]\s+)(\p{Ll})/gu,(m,a,b)=>a+b.toUpperCase())});
  if(/(^|[.!?]\s+|\n)[^.!?¿¡\n]+\?/.test(t))r.push({l:'Falta el signo de apertura ¿',f:s=>s.replace(/(^|[.!?]\s+|\n)([^.!?¿¡\n]+)\?/g,(m,a,b)=>a+'¿'+b.trimStart()+'?')});
  if(/(^|[.!?]\s+|\n)[^.!?¿¡\n]+!/.test(t))r.push({l:'Falta el signo de apertura ¡',f:s=>s.replace(/(^|[.!?]\s+|\n)([^.!?¿¡\n]+)!/g,(m,a,b)=>a+'¡'+b.trimStart()+'!')});
  for(const w in DICT){if(w===DICT[w])continue;const m=t.match(wordRe(w));if(m)r.push({l:`“${m[0]}” → “${matchCase(m[0],DICT[w])}”`,f:s=>s.replace(wordRe(w),x=>matchCase(x,DICT[w]))})}
  const tr=t.trim();
  if(tr.length>25&&/\s/.test(tr)&&/\p{L}$/u.test(tr))r.push({l:'Falta el punto final',f:s=>s.trimEnd()+'.'});
  return r;
}
const looksLikeCode=t=>/[{};=]|^\s*(select|insert|update|get-|set-|\$|<|import|def |function)/im.test(t);
function runCheck(){
  const ta=$('#etext'), box=$('#sugs'), t=ta.value; box.classList.add('show');
  if(!t.trim()){box.innerHTML='<div class="sugok">No hay texto para revisar.</div>';return}
  if(looksLikeCode(t)){box.innerHTML='<div class="sugok">Parece código o un comando: se omite la revisión de redacción.</div>';return}
  const rules=getRules(t);
  if(!rules.length){box.innerHTML='<div class="sugok">Todo se ve bien ✓</div>';return}
  box.innerHTML=rules.map((r,k)=>`<div class="sug" style="animation-delay:${k*30}ms"><span>${esc(r.l)}</span><button data-k="${k}">Aplicar</button></div>`).join('')
    +`<div style="padding:6px 12px 4px;text-align:right"><button class="sugall" id="applyAll">Aplicar todo</button></div>`;
  box.onclick=e=>{
    if(e.target.id==='applyAll'){let s=ta.value;for(let i=0;i<6;i++){const rs=getRules(s);if(!rs.length)break;rs.forEach(r=>s=r.f(s))}ta.value=s;runCheck();return}
    const b=e.target.closest('button[data-k]');if(b){ta.value=rules[+b.dataset.k].f(ta.value);runCheck()}
  };
}
$('#check').onclick=runCheck;

/* ================= BÚSQUEDA ================= */
let sRes=[],sIdx=0;
function openSearch(){$('#search').classList.add('open');$('#q').value='';runSearch();$('#q').focus();setTimeout(()=>$('#q').focus(),40)}
function closeSearch(){$('#search').classList.remove('open')}
/* Normaliza sin tildes ni mayúsculas, conservando la posición de cada letra */
function fold(t){const out=[],map=[];for(let k=0;k<t.length;k++){const c=t[k].normalize('NFD').replace(/\p{M}/gu,'').toLowerCase();for(const ch of c){out.push(ch);map.push(k)}}return{s:out.join(''),map}}
function terms(q){return [...new Set(fold(q).s.split(/\s+/).filter(Boolean))].sort((a,b)=>b.length-a.length)}
function findRanges(text,ts){
  const f=fold(text), r=[];
  for(const t of ts){let p=f.s.indexOf(t);while(p>-1){r.push([f.map[p],f.map[p+t.length-1]+1]);p=f.s.indexOf(t,p+t.length)}}
  r.sort((a,b)=>a[0]-b[0]); const m=[];
  for(const x of r){if(m.length&&x[0]<=m[m.length-1][1])m[m.length-1][1]=Math.max(m[m.length-1][1],x[1]);else m.push([...x])}
  return m;
}
function markup(text,ranges,from=0,to=text.length){
  let o='',p=from;
  for(const [a,b] of ranges){if(b<=from||a>=to)continue;const A=Math.max(a,from),B=Math.min(b,to);o+=esc(text.slice(p,A))+`<mark class="hl">${esc(text.slice(A,B))}</mark>`;p=B}
  return o+esc(text.slice(p,to));
}
function snippet(text,ranges){
  const t=text.replace(/\s+/g,' ');
  if(t!==text)ranges=findRanges(t,sTerms);
  if(!ranges.length)return esc(t.slice(0,160))+(t.length>160?'…':'');
  const a=Math.max(0,ranges[0][0]-40), b=Math.min(t.length,a+170);
  return (a>0?'…':'')+markup(t,ranges,a,b)+(b<t.length?'…':'');
}
let sTerms=[];
function searchText(i){return i.type==='image'?(i.note||''):isSess(i)?(i.content+(i.transcript?' — '+i.transcript:'')+(i.acta?' — '+actaText(i):'')):i.content}
function runSearch(){
  const q=$('#q').value.trim(); sTerms=q?terms(q):[];
  const hits=[];
  for(const i of live()){
    if(!sTerms.length){hits.push({i,r:[],n:0});continue}
    /* El texto normalizado se guarda en el elemento: no se recalcula en cada tecla */
    const txt=searchText(i), f=i._f??=fold(txt).s, src=i.source?appName(i.source).toLowerCase():'';
    if(!sTerms.every(t=>f.includes(t)||src.includes(t)))continue;
    const r=findRanges(txt,sTerms); hits.push({i,r,n:r.length});
  }
  hits.sort((a,b)=>b.i.ts-a.i.ts);
  const total=hits.reduce((s,h)=>s+h.n,0);
  sRes=hits.slice(0,60); sIdx=0;
  const L=$('#ssum');
  L.innerHTML=!sTerms.length?`<span class="lupa">${I.search}</span>Escribe para buscar en ${live().length} elementos`
    :hits.length?`<span class="lupa go">${I.search}</span><b>${total}</b> ${total===1?'coincidencia':'coincidencias'} en <b>${hits.length}</b> ${hits.length===1?'elemento':'elementos'}${hits.length>60?' · mostrando 60':''}`
    :`<span class="lupa go">${I.search}</span>Sin coincidencias para “${esc(q)}”`;
  paintResults();
}
function paintResults(){
  $('#results').innerHTML=sRes.length?sRes.map((h,k)=>{
    const i=h.i, body=i.type==='image'?(i.note?snippet(i.note,h.r):'Imagen')
      :isSess(i)?esc(KIND[i.skind]||'Grabación')+' · '+fmtDur(i.durationMs)+' · '+(searchText(i)?snippet(searchText(i),h.r):'sin título')
      :snippet(i.content,h.r);
    return `<div class="res${k===sIdx?' on':''}" data-id="${i.id}">${I[i.type]}<div class="rmain"><span class="rt">${body}</span>
      <span class="rmeta"><span>${fmtShort(i.ts)}</span>${i.source?`<span>${esc(appName(i.source))}</span>`:''}${i.pinned?'<span class="rtag">Fijado</span>':''}${h.n?`<span class="rbadge">${I.search}${h.n}</span>`:''}</span></div></div>`;
  }).join(''):(sTerms.length?'':'<div class="none">Aún no hay nada guardado</div>');
  const on=$('#results .res.on'); if(on)on.scrollIntoView({block:'nearest'});
}
function jumpTo(id){
  const it=byId(id);if(!it)return;closeSearch();
  state.zoom='hour';state.cursor=it.ts;const h=new Date(it.ts);h.setMinutes(0,0,0);state.sel=h.getTime();render();
  const ts=sTerms.slice();
  setTimeout(()=>{const c=document.querySelector(`.card[data-id="${id}"]`);if(!c)return;
    c.classList.remove('pop');c.classList.add('flash');setTimeout(()=>c.classList.remove('flash'),1300);
    const p=c.querySelector(it.type==='text'?'.body p':'.body .note');
    if(p&&ts.length){const txt=searchText(it);p.innerHTML=markup(txt,findRanges(txt,ts))}
    c.scrollIntoView({block:'nearest',behavior:'smooth'});},500);
}
$('#q').addEventListener('input',runSearch);
$('#results').addEventListener('click',e=>{const r=e.target.closest('.res');if(!r)return;const it=byId(r.dataset.id);if(isSess(it)){closeSearch();openViewer(it.id)}else jumpTo(r.dataset.id)});
$('#search').addEventListener('click',e=>{if(e.target.id==='search')closeSearch()});
$('#openS').onclick=openSearch;

document.addEventListener('keydown',e=>{
  if(e.target.closest&&e.target.closest('#recPop'))return;
  if($('#bit').classList.contains('open')&&!$('#tset').classList.contains('open')){
    if(e.key==='Escape'){if(bitView!=null){bitView=null;paintBit()}else if(bitRepOpen){bitRepOpen=false;paintBit()}else closeBit()}
    else if(bitView!=null&&(e.key==='ArrowLeft'||e.key==='ArrowRight')){e.preventDefault();stepBit(e.key==='ArrowLeft'?-1:1)}
    return;
  }
  if($('#viewer').classList.contains('open')||$('#tset').classList.contains('open')){if(e.key==='Escape'){closeViewer();closeTset()}return}
  const edOpen=$('#editor').classList.contains('open'), sOpen=$('#search').classList.contains('open');
  if(edOpen){
    if(e.key==='Escape')closeEditor();
    if((e.ctrlKey||e.metaKey)&&e.key==='Enter'){e.preventDefault();saveEditor()}
    return;
  }
  if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='k'){e.preventDefault();openSearch();return}
  if(!sOpen){
    if(e.key==='Escape')invoke('hide_window');
    else if(e.key.length===1&&!e.ctrlKey&&!e.altKey&&!e.metaKey&&/\S/.test(e.key)){openSearch();$('#q').value=e.key;runSearch();e.preventDefault()}
    return;
  }
  if(e.key==='Escape')closeSearch();
  else if(e.key==='ArrowDown'){e.preventDefault();sIdx=Math.min(sIdx+1,sRes.length-1);paintResults()}
  else if(e.key==='ArrowUp'){e.preventDefault();sIdx=Math.max(sIdx-1,0);paintResults()}
  else if(e.key==='Enter'&&sRes[sIdx]){const it=sRes[sIdx].i;if(isSess(it)){closeSearch();openViewer(it.id)}else{closeSearch();pasteItem(it)}}
});

/* ================= CONTROLES DE VENTANA ================= */
function paintToggles(){
  $('#bPause').innerHTML=paused?I.play:I.pause;
  $('#bPause').title=paused?'Reanudar la captura':'Pausar la captura del portapapeles';
  $('#bPause').classList.toggle('on',paused);
  $('#bKeep').classList.toggle('on',keepOpen);
  document.body.classList.toggle('is-paused',paused);
}
$('#bPause').onclick=()=>{invoke('set_paused',{value:!paused})};
$('#bKeep').onclick=()=>{keepOpen=!keepOpen;invoke('set_keep_open',{value:keepOpen});paintToggles();toast(keepOpen?'La ventana se mantiene abierta':'La ventana se oculta al cambiar de app')};
$('#bHide').onclick=()=>invoke('hide_window');
listen('paused',ev=>{paused=!!ev.payload;paintToggles();toast(paused?'Captura en pausa':'Captura activa')});
listen('shown',()=>{closeSearch();closeCtx();loadSystemColors();if(!$('#editor').classList.contains('open')){goHome();render()}setTimeout(()=>layoutAuto(false),120)});

/* ================= TOAST ================= */
let tt,tfn=null;
function toast(m,label,fn){
  const t=$('#toast');$('#tmsg').textContent=m;tfn=fn||null;
  t.classList.toggle('act',!!fn);if(label)$('#tact').textContent=label;
  t.classList.add('show');clearTimeout(tt);tt=setTimeout(()=>t.classList.remove('show'),fn?4500:1800);
}
$('#tact').onclick=()=>{if(tfn){tfn();tfn=null}$('#toast').classList.remove('show')};

/* ================= GRABAR REUNIONES Y CAPACITACIONES ================= */
let rec={active:false}, recKind='reunion';
try{recKind=localStorage.getItem('lienzo-rec-tipo')||'reunion'}catch(e){}
function paintKind(){document.querySelectorAll('#recKind button').forEach(b=>b.classList.toggle('on',b.dataset.k===recKind));moveInd('#recKind',document.querySelector(`#recKind button[data-k="${recKind}"]`))}
function openRecPop(anchor){
  if(rec.active){toast('Ya estás grabando');return}
  const pop=$('#recPop'), r=anchor.getBoundingClientRect();
  pop.classList.add('open');
  pop.style.top=(r.bottom+8)+'px';pop.style.left=Math.max(8,Math.min(innerWidth-pop.offsetWidth-8,r.right-pop.offsetWidth))+'px';
  paintKind();$('#recTitle').value='';setTimeout(()=>$('#recTitle').focus(),60);
}
function closeRecPop(){$('#recPop').classList.remove('open')}
$('#recKind').addEventListener('click',e=>{const b=e.target.closest('button');if(!b)return;recKind=b.dataset.k;try{localStorage.setItem('lienzo-rec-tipo',recKind)}catch(x){}paintKind()});
async function startRec(){
  const mic=true, system=true; /* siempre tu micrófono y el audio del PC */
  $('#recGo').disabled=true;
  try{await invoke('rec_start',{kind:recKind,title:$('#recTitle').value,mic,system});closeRecPop();toast((KIND[recKind]||'Grabación')+': grabando')}
  catch(e){toast(''+e)}
  finally{$('#recGo').disabled=false}
}
$('#recGo').onclick=startRec;
$('#recTitle').addEventListener('keydown',e=>{if(e.key==='Enter'){e.preventDefault();startRec()}if(e.key==='Escape'){e.stopPropagation();closeRecPop()}});
document.addEventListener('click',e=>{
  const b=e.target.closest('[data-rec]');
  if(b){if($('#recPop').classList.contains('open'))closeRecPop();else openRecPop(b);return}
  if(!e.target.closest('#recPop'))closeRecPop();
});
$('#recPause').onclick=()=>invoke('rec_pause',{paused:!rec.paused});
/* El mismo indicador de tu voz sirve para silenciarla: solo se graba el audio del PC */
$('#recMute').onclick=async()=>{const m=!rec.muted;try{await invoke('rec_mute',{muted:m});toast(m?'Micrófono silenciado: solo se graba el PC':'Micrófono activo')}catch(e){toast(''+e)}};
$('#recStop').onclick=async()=>{
  try{
    const s=await invoke('rec_stop'); if(!s)return;
    const su=await invoke('transcribe_setup'), m=su.models.find(x=>x.id===su.model);
    if(su.auto&&!(m&&m.installed))toast('Grabación guardada. Para transcribirla descarga un modelo','Configurar',openTset);
    else toast('Grabación guardada · '+fmtDur(s.durationMs));
  }catch(e){toast('No se pudo guardar: '+e)}
};
function paintRec(){
  const on=!!rec.active, bar=$('#recBar');
  bar.hidden=!on; document.body.classList.toggle('recording',on);
  document.querySelectorAll('[data-rec]').forEach(b=>b.classList.toggle('on',on));
  if(!on)return;
  bar.classList.toggle('paused',!!rec.paused);
  $('#recLabel').textContent=(rec.title||KIND[rec.kind]||'Grabación');
  $('#recPause').innerHTML=rec.paused?I.play:I.pause; $('#recPause').title=rec.paused?'Reanudar':'Pausar';
  $('#recTime').textContent=fmtDur(rec.elapsedMs||0);
  $('#recMeterMic').closest('.mt').hidden=!rec.mic; $('#recMeterSys').closest('.mt').hidden=!rec.system;
  const mb=$('#recMute'), mu=!!rec.muted;
  mb.classList.toggle('off',mu); mb.querySelector('.mic-ic').innerHTML=mu?I.micOff:I.mic;
  mb.title=mu?'Micrófono silenciado · clic para activarlo':'Silenciar tu micrófono';
}
/* Medidor con caída suave: el pico llega 10 veces por segundo */
const lvl={mic:0,sys:0};
function meter(el,k,v){lvl[k]=Math.max(v,lvl[k]*.7);el.style.transform='scaleX('+Math.min(1,Math.sqrt(lvl[k])*1.4).toFixed(3)+')'}
listen('rec-state',ev=>{rec=ev.payload||{active:false};paintRec()});
listen('rec-tick',ev=>{const t=ev.payload;rec.elapsedMs=t.elapsedMs;rec.paused=t.paused;$('#recTime').textContent=fmtDur(t.elapsedMs);meter($('#recMeterMic'),'mic',t.mic);meter($('#recMeterSys'),'sys',t.system)});
listen('rec-error',ev=>toast(''+ev.payload));
function upsertSession(r){
  const s=fromSession(r), old=byId(s.id);
  if(old){const pr=old.progress;Object.assign(old,s);if(s.status==='transcribiendo')old.progress=pr;delete old._f;refreshStill();if(viewing===s.id)openViewer(s.id);return}
  items.push(s); const [a,b]=range(); if(!(s.ts>=a&&s.ts<b))goHome();
  fresh=s.id; render();
}
listen('session-updated',ev=>upsertSession(ev.payload));

/* ================= TRANSCRIPCIÓN ================= */
let tsetup=null, dl=null;
async function loadSetup(){try{tsetup=await invoke('transcribe_setup')}catch(e){tsetup=null}return tsetup}
async function startTranscribe(it){
  const su=await loadSetup(); const m=su&&su.models.find(x=>x.id===su.model);
  if(!m||!m.installed){toast('Primero descarga un modelo de transcripción');openTset();return}
  try{await invoke('transcribe_session',{id:it.id})}catch(e){toast(''+e)}
}
listen('transcribe-progress',ev=>{
  const {id,pct}=ev.payload, it=byId(id); if(!it)return;
  it.progress=pct;
  if(it.status!=='transcribiendo'){it.status='transcribiendo';refreshStill()}
  const c=layer.querySelector(`.card[data-id="${id}"]`);
  if(c){const b=c.querySelector('.tbar i');if(b)b.style.transform='scaleX('+pct/100+')';const t=c.querySelector('.tpct');if(t)t.textContent=pct+'%'}
  if(viewing===id)paintViewerStatus(it);
});

/* ---- Ajustes: modelo, idioma y transcripción automática ---- */
function openTset(){$('#tset').classList.add('open');loadSetup().then(paintTset);paintGem()}
function closeTset(){$('#tset').classList.remove('open')}
function paintTset(){
  const su=tsetup; if(!su)return;
  $('#tmodels').innerHTML=su.models.map(m=>{
    const sel=m.id===su.model, busy=dl&&dl.id===m.id;
    const act=busy?`<span class="tbar"><i style="transform:scaleX(${dl.total?dl.done/dl.total:0})"></i></span><button class="sact" data-m="cancel" title="Cancelar">${I.x}</button>`
      :m.installed?(sel?`<span class="tok">${I.check}</span>`:`<button class="sact txt" data-m="use">Usar</button>`)+`<button class="sact" data-m="delete" title="Eliminar del equipo">${I.trash}</button>`
      :`<button class="sact txt" data-m="download"${dl?' disabled':''}>Descargar</button>`;
    return `<div class="tm${sel?' on':''}" data-id="${m.id}"><div class="tmi"><b>${esc(m.label)}</b><small>${esc(m.hint)} · ${m.sizeMb} MB</small></div><div class="tma">${act}</div></div>`;
  }).join('');
  document.querySelectorAll('#tlang button').forEach(b=>b.classList.toggle('on',b.dataset.v===su.language));
  document.querySelectorAll('#tauto button').forEach(b=>b.classList.toggle('on',b.dataset.v===(su.auto?'1':'0')));
  moveInd('#tlang',document.querySelector(`#tlang button[data-v="${su.language}"]`));
  moveInd('#tauto',document.querySelector(`#tauto button[data-v="${su.auto?'1':'0'}"]`));
}
async function savePrefs(ch){
  Object.assign(tsetup,ch); paintTset();
  try{await invoke('set_transcribe_prefs',{model:tsetup.model,language:tsetup.language,auto:tsetup.auto})}catch(e){toast(''+e)}
}
$('#tlang').addEventListener('click',e=>{const b=e.target.closest('button');if(b&&tsetup)savePrefs({language:b.dataset.v})});
$('#tauto').addEventListener('click',e=>{const b=e.target.closest('button');if(b&&tsetup)savePrefs({auto:b.dataset.v==='1'})});
$('#tmodels').addEventListener('click',async e=>{
  const row=e.target.closest('.tm'); if(!row||!tsetup)return;
  const id=row.dataset.id, m=tsetup.models.find(x=>x.id===id), a=e.target.closest('[data-m]')?.dataset.m;
  if(a==='download'){
    dl={id,done:0,total:m.sizeMb*1048576}; paintTset();
    try{await invoke('download_model',{id});dl=null;await loadSetup();await savePrefs({model:id});toast('Modelo '+m.label+' listo')}
    catch(err){dl=null;paintTset();toast(''+err)}
  }
  else if(a==='cancel')invoke('cancel_download');
  else if(a==='delete'){try{await invoke('delete_model',{id});await loadSetup();paintTset();toast('Modelo eliminado del equipo')}catch(err){toast(''+err)}}
  else if((a==='use'||!a)&&m.installed&&id!==tsetup.model)savePrefs({model:id});
});
listen('model-progress',ev=>{const p=ev.payload;if(dl&&dl.id===p.id){dl.done=p.done;dl.total=p.total;const b=document.querySelector(`.tm[data-id="${p.id}"] .tbar i`);if(b)b.style.transform='scaleX('+(p.total?p.done/p.total:0)+')'}});
$('#tclose').onclick=closeTset;
$('#tset').addEventListener('click',e=>{if(e.target.id==='tset')closeTset()});

/* ---- Visor: transcripción con el audio ---- */
let viewing=null, vblocks=[], vtab='t', gemOk=false, actaBusy=null;
const clock=ms=>fmtDur(ms);
async function openViewer(id){
  const it=byId(id); if(!it)return;
  if(viewing!==id)vtab='t';
  viewing=id; $('#viewer').classList.add('open');
  $('#vtitle').textContent=sessTitle(it);
  $('#vmeta').textContent='· '+(KIND[it.skind]||'Grabación')+' · '+fmtShort(it.ts)+(it.durationMs?' · '+fmtDur(it.durationMs):'');
  const au=$('#vaudio'); au.hidden=it.status==='grabando'||!it.audioPath; if(!au.hidden&&au.dataset.id!==id){au.src=fileSrc(it.audioPath);au.dataset.id=id}
  const done=it.status==='transcrita'; if(!done)vtab='t';
  $('#vtabs').hidden=!done;
  document.querySelectorAll('#vtabs button').forEach(b=>b.classList.toggle('on',b.dataset.v===vtab));
  moveInd('#vtabs',document.querySelector(`#vtabs button[data-v="${vtab}"]`));
  if(vtab==='a'){
    $('#vredo').hidden=!gemOk||actaBusy===id; $('#vredo').textContent=it.acta?'Volver a generar':'Generar acta';
    $('#vcopy').disabled=!it.acta; $('#vcopy').title='Copiar el acta'; $('#vexport').disabled=false;
    paintActa(it); return;
  }
  $('#vredo').hidden=!(done||it.status==='grabada');
  $('#vredo').textContent=done?'Volver a transcribir':'Transcribir';
  $('#vcopy').disabled=!it.transcript; $('#vcopy').title='Copiar la transcripción';
  $('#vexport').disabled=!done;
  await paintViewer(it);
}
function closeViewer(){$('#viewer').classList.remove('open');const au=$('#vaudio');au.pause();viewing=null}
function paintViewerStatus(it){
  const box=$('#vbody .vstate'); if(!box)return;
  if(it.status==='transcribiendo')box.innerHTML=`<span class="tbar"><i style="transform:scaleX(${(it.progress||0)/100})"></i></span><span>Transcribiendo · ${it.progress||0}%</span>`;
}
async function paintViewer(it){
  const body=$('#vbody'); vblocks=[];
  if(it.status!=='transcrita'){
    const msg={grabando:'Se está grabando. La transcripción estará aquí cuando termine.',en_cola:'En cola para transcribir…',transcribiendo:'',error:'Esta grabación no se guardó bien.'}[it.status]
      ??(it.error?'No se pudo transcribir: '+it.error:'Aún no está transcrita.');
    body.innerHTML=`<div class="vempty"><div class="vstate">${esc(msg)}</div></div>`; paintViewerStatus(it); return;
  }
  let t=null; try{t=await invoke('get_transcript',{id:it.id})}catch(e){}
  if(viewing!==it.id)return;
  if(!t||!t.segments.length){body.innerHTML=`<div class="vempty">${esc(it.transcript||'Sin texto')}</div>`;return}
  const both=t.segments.some(s=>s.who==='yo')&&t.segments.some(s=>s.who==='otros');
  /* Párrafos: cambia quien habla o pasan 45 s */
  for(const s of t.segments){const b=vblocks[vblocks.length-1];if(b&&b.who===s.who&&s.t0-b.t0<45000){b.text+=' '+s.text;b.t1=s.t1}else vblocks.push({...s})}
  body.innerHTML=vblocks.map((b,k)=>`<div class="vblk${both?' w-'+b.who:''}" data-k="${k}"><button class="vts" data-t="${b.t0}">${clock(b.t0)}</button><div>${both?`<span class="vwho">${b.who==='yo'?'Tú':'Participantes'}</span>`:''}<p>${esc(b.text)}</p></div></div>`).join('');
}
$('#vbody').addEventListener('click',e=>{const b=e.target.closest('.vts');if(!b)return;const au=$('#vaudio');au.currentTime=+b.dataset.t/1000;au.play().catch(()=>{})});
$('#vaudio').addEventListener('timeupdate',e=>{
  const ms=e.target.currentTime*1000; let on=-1;
  vblocks.forEach((b,k)=>{if(ms>=b.t0)on=k});
  document.querySelectorAll('#vbody .vblk').forEach((el,k)=>el.classList.toggle('on',k===on));
});
$('#vclose').onclick=closeViewer;
$('#viewer').addEventListener('click',e=>{if(e.target.id==='viewer')closeViewer()});
$('#vcopy').onclick=async()=>{
  const it=byId(viewing), a=vtab==='a', text=it&&(a?actaText(it):it.transcript); if(!text)return;
  try{await invoke('copy_text',{text});toast(a?'Acta copiada':'Transcripción copiada')}catch(e){toast(''+e)}
};
/* Exportar: acta (si hay) + transcripción, en Documentos\Lienzo */
$('#vexport').onclick=e=>{
  const it=byId(viewing); if(!it)return; ctxActions={it,el:null};
  const r=e.currentTarget.getBoundingClientRect();
  openCtx(r.left,r.bottom+6,`<div class="lbl">Exportar${it.acta?' acta y transcripción':' transcripción'}</div>`+mi('exp:docx','text','Word (.docx)')+mi('exp:md','pencil','Markdown (.md)'));
};
async function exportSess(it,format){
  try{await invoke('export_session',{id:it.id,format,when:new Date(it.ts).toLocaleDateString('es-CO',{day:'numeric',month:'long',year:'numeric'})+' · '+new Date(it.ts).toLocaleTimeString('es-CO',{hour:'2-digit',minute:'2-digit'})});toast('Guardado en Documentos\\Lienzo')}
  catch(e){toast(''+e)}
}
$('#vfolder').onclick=()=>{const it=byId(viewing);if(it)openFolder(it)};
$('#vset').onclick=openTset;
$('#vredo').onclick=()=>{const it=byId(viewing);if(!it)return;if(vtab==='a')genActa(it);else startTranscribe(it)};
$('#vtabs').addEventListener('click',e=>{const b=e.target.closest('button');if(!b||b.dataset.v===vtab)return;vtab=b.dataset.v;openViewer(viewing)});

/* ---- Acta con Gemini ---- */
const parseActa=it=>{try{return it.acta?JSON.parse(it.acta):null}catch(e){return null}};
const minToMs=m=>{const p=String(m||'').trim().split(':').map(Number);if(p.length<2||p.some(isNaN))return null;return p.reduce((a,b)=>a*60+b,0)*1000};
const arr=v=>Array.isArray(v)?v:[];
function actaText(it){
  const A=parseActa(it); if(!A)return '';
  const out=[sessTitle(it),fmtShort(it.ts),'','Resumen',A.resumen||''];
  const sec=(t,lines)=>{if(lines.length)out.push('',t,...lines)};
  const who=t=>[t.responsable,t.fecha].filter(Boolean).join(' · ');
  sec('Decisiones',arr(A.decisiones).map(d=>'- '+d));
  sec('Tareas',arr(A.tareas).map(t=>'- [ ] '+t.tarea+(who(t)?' ('+who(t)+')':'')));
  sec('Conceptos',arr(A.conceptos).map(c=>'- '+c.concepto+': '+c.explicacion));
  sec('Pasos',arr(A.pasos).map((p,k)=>(k+1)+'. '+p));
  sec('Temas',arr(A.temas).map(t=>'['+t.minuto+'] '+t.tema));
  return out.join('\n');
}
function paintActa(it){
  const body=$('#vbody');
  if(actaBusy===it.id){body.innerHTML='<div class="vempty"><div class="vstate">Gemini está redactando el acta…</div></div>';return}
  const A=parseActa(it);
  if(!A){
    body.innerHTML=gemOk
      ?`<div class="vempty">Gemini puede resumir esta ${esc((KIND[it.skind]||'grabación').toLowerCase())} en ${it.skind==='capacitacion'?'conceptos, pasos y temas':'decisiones, tareas con responsable y temas'}.<small>Se envía solo el texto de la transcripción. Con la clave gratuita, Google puede usarlo para mejorar sus productos: evítalo en reuniones confidenciales.</small></div>`
      :`<div class="vempty">Para crear el acta conecta tu clave gratuita de Gemini.<br><button class="ghost" data-x="tset">Configurar la clave</button></div>`;
    return;
  }
  const list=(t,items,f)=>`<section><h4>${t}</h4>${items.length?f(items):'<p class="anone">Nada registrado</p>'}</section>`;
  const ts=m=>{const ms=minToMs(m);return ms==null?'':`<button class="vts" data-t="${ms}">${esc(m)}</button>`};
  let h=`<section><h4>Resumen</h4><p>${esc(A.resumen||'')}</p></section>`;
  if(it.skind==='capacitacion'){
    h+=list('Conceptos',arr(A.conceptos),x=>x.map(c=>`<div class="aconc"><b>${esc(c.concepto)}</b><small>${esc(c.explicacion)}</small></div>`).join(''));
    h+=list('Pasos',arr(A.pasos),x=>`<ol>${x.map(p=>`<li>${esc(p)}</li>`).join('')}</ol>`);
  }else{
    h+=list('Decisiones',arr(A.decisiones),x=>`<ul>${x.map(d=>`<li>${esc(d)}</li>`).join('')}</ul>`);
    h+=list('Tareas',arr(A.tareas),x=>x.map(t=>{const m=[t.responsable,t.fecha].filter(Boolean).map(esc).join(' · ');return `<div class="atask"><span class="abox"></span><div><p>${esc(t.tarea)}</p>${m?`<small>${m}</small>`:''}</div></div>`}).join(''));
  }
  h+=list('Temas',arr(A.temas),x=>x.map(t=>`<div class="atema">${ts(t.minuto)}<p>${esc(t.tema)}</p></div>`).join(''));
  body.innerHTML=`<div class="acta">${h}</div>`;
}
async function genActa(it){
  if(actaBusy)return;
  actaBusy=it.id; if(viewing===it.id)openViewer(it.id);
  try{const a=await invoke('generate_acta',{id:it.id});it.acta=a;delete it._f;toast('Acta lista')}
  catch(e){toast(''+e);if(/clave/i.test(''+e))refreshGem()}
  finally{actaBusy=null;if(viewing===it.id)openViewer(it.id);refreshStill()}
}
$('#vbody').addEventListener('click',e=>{if(e.target.closest('[data-x="tset"]'))openTset()});

/* ---- Clave de Gemini (se guarda en el Administrador de credenciales de Windows) ---- */
async function refreshGem(){try{gemOk=await invoke('gemini_status')}catch(e){gemOk=false}paintGem();if(viewing)openViewer(viewing)}
function paintGem(){
  const box=$('#tgem'); if(!box)return;
  box.innerHTML=gemOk
    ?`<div class="tm on"><div class="tmi"><b>Clave conectada</b><small>Guardada en el Administrador de credenciales de Windows</small></div><div class="tma"><button class="sact" data-g="delete" title="Quitar la clave">${I.trash}</button></div></div>`
    :`<div class="gkey"><input id="gemKey" type="password" placeholder="Pega tu clave de Gemini" autocomplete="off" spellcheck="false"><button class="ghost" data-g="save">Guardar</button></div><div class="ghint">Consíguela gratis en aistudio.google.com/apikey</div>`;
}
async function saveGem(){
  const inp=$('#gemKey'), b=$('#tgem [data-g="save"]'); if(!inp||!inp.value.trim())return;
  b.disabled=true;
  try{await invoke('set_gemini_key',{key:inp.value});toast('Clave de Gemini conectada');await refreshGem()}
  catch(e){toast(''+e);b.disabled=false}
}
$('#tgem').addEventListener('click',async e=>{
  const a=e.target.closest('[data-g]')?.dataset.g;
  if(a==='save')saveGem();
  if(a==='delete'){await invoke('delete_gemini_key');toast('Clave quitada del equipo');refreshGem()}
});
$('#tgem').addEventListener('keydown',e=>{if(e.key==='Enter'&&e.target.id==='gemKey'){e.preventDefault();saveGem()}});

/* ================= BITÁCORA ================= */
/* Lo que hiciste en las apps elegidas: una captura por clic, con el nombre de lo que tocaste */
let bitOn=false, bitDay=startOfDay(Date.now()), bitRows=[], bitView=null, bitSet=false, bsetup=null, bitRep=null, bitRepOpen=false, bitRepBusy=false;
const dayEnd=t=>{const d=new Date(t);return new Date(d.getFullYear(),d.getMonth(),d.getDate()+1).getTime()};
const hms=t=>new Date(t).toLocaleTimeString('es-CO',{hour:'2-digit',minute:'2-digit',second:'2-digit'});
function paintBitBtn(){document.querySelectorAll('[data-bit]').forEach(b=>{b.classList.toggle('live',bitOn);b.title=bitOn?'Bitácora activa (Alt+Shift+B)':'Bitácora (Alt+Shift+B)'})}
async function openBit(settings){
  $('#bit').classList.add('open'); bitView=null;
  try{bsetup=await invoke('bitacora_setup')}catch(e){bsetup=null}
  bitOn=!!(bsetup&&bsetup.on); paintBitBtn();
  bitSet=!!settings||!(bsetup&&bsetup.apps.length);
  await loadBit();
}
function closeBit(){$('#bit').classList.remove('open');bitView=null;bitRepOpen=false}
async function loadBit(){
  try{bitRows=await invoke('list_bitacora',{from:bitDay,to:dayEnd(bitDay)})}catch(e){bitRows=[]}
  let j=null; try{j=await invoke('get_informe',{day:bitDay})}catch(e){}
  try{bitRep=j?JSON.parse(j):null}catch(e){bitRep=null}
  paintBit();
}
function paintBit(){
  const t=startOfDay(Date.now()), d=new Date(bitDay);
  $('#bDay').textContent=bitDay===t?'Hoy':bitDay===t-D?'Ayer':cap(d.toLocaleDateString('es-CO',{weekday:'long',day:'numeric',month:'long'}));
  $('#bNext').disabled=bitDay>=t;
  $('#bToggle').innerHTML=bitOn?I.pause:I.play; $('#bToggle').title=bitOn?'Pausar la bitácora':'Activar la bitácora';
  $('#bToggle').classList.toggle('on',bitOn); $('#bSet').classList.toggle('on',bitSet);
  $('#bFoot').hidden=bitSet;
  $('#bCount').textContent=bitRows.length+(bitRows.length===1?' clic registrado':' clics registrados');
  $('#bDel').hidden=!bitRows.length||bitRepOpen;
  $('#bRep').hidden=!bitRows.length; $('#bRep').disabled=bitRepBusy;
  $('#bRep').textContent=bitRepBusy?'Redactando…':bitRepOpen?(bitRep?'Volver a generar':'Generar informe'):bitRep?'Ver informe':'Informe del día';
  const body=$('#bBody');
  if(bitSet){paintBitSettings();return}
  if(bitRepOpen){paintReport();return}
  if(bitView!=null&&bitRows[bitView]){paintBitView();return}
  if(!bitRows.length){
    body.innerHTML=`<div class="vempty">${bitOn?'Nada registrado este día.<br>Cada clic en las apps elegidas aparecerá aquí.':'La bitácora está en pausa.<br>Actívala con ▶ o con Alt+Shift+B.'}</div>`;return;
  }
  /* Grupos: clics seguidos en la misma app y ventana */
  const groups=[];
  bitRows.forEach((r,k)=>{const g=groups[groups.length-1];if(g&&g.app===r.app&&g.title===r.title)g.k.push(k);else groups.push({app:r.app,title:r.title,k:[k]})});
  body.innerHTML=groups.map(g=>`<div class="bgrp"><div class="bgh"><span class="bght"><b>${new Date(bitRows[g.k[0]].ts).toLocaleTimeString('es-CO',{hour:'2-digit',minute:'2-digit'})}</b> · ${esc(appName(g.app))}${g.title?' · '+esc(g.title):''}</span>${g.k.length>1?`<button class="sact txt" data-tut="${g.k[0]}-${g.k[g.k.length-1]}" title="Convertir estos clics en una guía paso a paso">Tutorial</button>`:''}</div><div class="bshots">${g.k.map(k=>{
    const r=bitRows[k];
    return `<button class="bshot" data-k="${k}"><div class="bimg">${r.image?`<img src="${fileSrc(r.image)}" alt="" loading="lazy" draggable="false">`:''}${r.x!=null?`<span class="bdot" style="left:${r.x*100}%;top:${r.y*100}%"></span>`:''}</div><div class="bmeta"><span>${hms(r.ts)}</span><small>${esc(r.target||'Clic')}</small></div></button>`;
  }).join('')}</div></div>`).join('');
}
function paintBitView(){
  const r=bitRows[bitView];
  $('#bBody').innerHTML=`<div class="bview"><div class="bnav"><button class="ib sm" data-b="back" title="Volver (Esc)">${I.left}</button><div class="binfo"><b>${hms(r.ts)}</b> · ${esc(appName(r.app))}${r.title?' · '+esc(r.title):''}${r.target?' · clic en “'+esc(r.target)+'”':''}</div><span style="font-size:11.5px;color:var(--ink3)">${bitView+1} / ${bitRows.length}</span><button class="ib sm" data-b="prev" title="Anterior (←)"${bitView?'':' disabled'}>${I.left}</button><button class="ib sm" data-b="next" title="Siguiente (→)"${bitView<bitRows.length-1?'':' disabled'}>${I.right}</button></div>
    <div class="bbig">${r.image?`<img src="${fileSrc(r.image)}" alt="">`:'<div class="vempty">Sin captura</div>'}${r.x!=null&&r.image?`<span class="bdot" style="left:${r.x*100}%;top:${r.y*100}%"></span>`:''}</div>
    ${r.ocr?`<details><summary>Texto de la pantalla</summary><pre>${esc(r.ocr)}</pre></details>`:''}</div>`;
}
function stepBit(d){const n=bitView+d;if(n>=0&&n<bitRows.length){bitView=n;paintBitView()}}
function paintBitSettings(){
  if(!bsetup)bsetup={apps:[],running:[],title:'',days:15,ocr:true};
  const su=bsetup;
  const all=[...new Set([...su.apps,...su.running.map(a=>a.exe.toLowerCase())])].sort((a,b)=>appName(a).localeCompare(appName(b),'es'));
  bsetup.sel=bsetup.sel||new Set(su.apps);
  $('#bBody').innerHTML=`<div class="tbody">
    <div class="tlbl">Apps que se registran</div>
    <div class="bchips">${all.length?all.map(a=>`<button class="bchip${bsetup.sel.has(a)?' on':''}" data-app="${esc(a)}">${esc(appName(a))}</button>`).join(''):'<small class="tnote">Abre las apps que quieras registrar y vuelve aquí.</small>'}</div>
    <small class="tnote" style="margin-top:0">Aparecen las apps abiertas ahora. Solo se registran los clics dentro de las que elijas.</small>
    <div class="tlbl">Solo ventanas cuyo título contenga (opcional)</div>
    <input class="bin" id="bTitle" placeholder="Ej.: Portal de facturación" autocomplete="off" spellcheck="false" value="${esc(su.title)}">
    <div class="tlbl">Guardar por</div>
    <div class="seg" id="bDays"><span class="ind"></span>${[7,15,30].map(n=>`<button data-v="${n}" class="${su.days===n?'on':''}">${n} días</button>`).join('')}</div>
    <small class="tnote">Todo queda en este equipo. No se registra lo que escribes ni los campos de contraseña, y nunca se capturan los gestores de contraseñas.${su.ocr?'':' <b>El OCR de Windows no está disponible:</b> agrega el idioma español en Configuración → Hora e idioma → Idioma para leer el texto de las capturas.'}</small>
    <div class="bsave"><button class="sbtn solid" id="bSave">${bitOn?'Guardar':'Guardar y activar'}</button></div>
  </div>`;
  moveInd('#bDays',document.querySelector('#bDays button.on'));
}
async function saveBit(){
  const days=+(document.querySelector('#bDays button.on')?.dataset.v||15), apps=[...bsetup.sel];
  if(!apps.length){toast('Elige al menos una app');return}
  try{await invoke('set_bitacora',{on:true,apps,title:$('#bTitle').value,days});bsetup.apps=apps;bsetup.days=days;bsetup.title=$('#bTitle').value;bitSet=false;toast('Bitácora activa: registra tus clics en '+apps.map(appName).join(', '));loadBit()}
  catch(e){toast(''+e)}
}
$('#bBody').addEventListener('click',e=>{
  const chip=e.target.closest('.bchip'); if(chip){const a=chip.dataset.app;bsetup.sel.has(a)?bsetup.sel.delete(a):bsetup.sel.add(a);chip.classList.toggle('on');return}
  const day=e.target.closest('#bDays button'); if(day){document.querySelectorAll('#bDays button').forEach(b=>b.classList.toggle('on',b===day));moveInd('#bDays',day);return}
  if(e.target.closest('#bSave')){saveBit();return}
  const shot=e.target.closest('.bshot'); if(shot){bitView=+shot.dataset.k;paintBit();return}
  const tut=e.target.closest('[data-tut]'); if(tut){const [a,z]=tut.dataset.tut.split('-').map(Number),r=tut.getBoundingClientRect();ctxActions={tut:bitRows.slice(a,z+1).map(x=>x.id)};openCtx(r.left,r.bottom+6,'<div class="lbl">Tutorial en Word</div>'+mi('tut:plain','image','Con las capturas')+(gemOk?mi('tut:ai','spell','Redactado con Gemini'):''));return}
  const at=e.target.closest('[data-at]'); if(at){const k=bitRows.findIndex(r=>{const d=new Date(r.ts);return [d.getHours(),d.getMinutes(),d.getSeconds()].map(two).join(':')>=at.dataset.at});if(k>=0){bitRepOpen=false;bitView=k;paintBit()}return}
  const b=e.target.closest('[data-b]')?.dataset.b;
  if(b==='back'){bitView=null;paintBit()} else if(b==='prev')stepBit(-1); else if(b==='next')stepBit(1);
});
$('#bPrev').onclick=()=>{bitDay=startOfDay(bitDay-D/2);bitView=null;bitRepOpen=false;loadBit()};
$('#bNext').onclick=()=>{bitDay=dayEnd(bitDay);bitView=null;bitRepOpen=false;loadBit()};

/* ---- Informe del día (Gemini, solo texto y solo cuando lo pides) ---- */
function paintReport(){
  const body=$('#bBody'), A=bitRep;
  if(bitRepBusy){body.innerHTML='<div class="vempty"><div class="vstate">Gemini está revisando tu día…</div></div>';return}
  if(!A){
    body.innerHTML=gemOk
      ?`<div class="vempty">Gemini puede resumir lo que hiciste este día y señalar acciones que conviene revisar.<small>Se envía solo texto: la hora, la app, la ventana, lo que tocaste y parte del texto que se veía, nunca las capturas. Con la clave gratuita, Google puede usarlo para mejorar sus productos: evítalo si hay información confidencial.</small></div>`
      :`<div class="vempty">Para el informe conecta tu clave gratuita de Gemini.<br><button class="ghost" data-x="tset">Configurar la clave</button></div>`;
    return;
  }
  const list=(t,items,f)=>`<section><h4>${t}</h4>${items.length?f(items):'<p class="anone">Nada para señalar</p>'}</section>`;
  body.innerHTML=`<div class="acta brep"><div class="bnav" style="display:flex;align-items:center;gap:8px"><button class="ib sm" data-b="repback" title="Volver (Esc)">${I.left}</button><span class="sp" style="flex:1"></span><button class="ib sm" data-b="repcopy" title="Copiar el informe">${I.copy}</button></div>
    <section><h4>Resumen</h4><p>${esc(A.resumen||'')}</p></section>
    ${list('Tu día',arr(A.bloques),x=>x.map(b=>`<div class="bblk"><span>${esc(b.desde)} – ${esc(b.hasta)}</span><p><b>${esc(appName(b.app))}</b> · ${esc(b.actividad)}</p></div>`).join(''))}
    ${list('Para revisar',arr(A.atencion),x=>x.map(a=>`<div class="atask" data-at="${esc(a.hora)}" title="Ver la captura de ese momento"><span class="vts">${esc(a.hora)}</span><div><p>${esc(a.detalle)}</p></div></div>`).join(''))}
  </div>`;
}
function reportText(){
  const A=bitRep; if(!A)return '';
  return ['Bitácora · '+$('#bDay').textContent,'','Resumen',A.resumen||'','','Tu día',...arr(A.bloques).map(b=>`${b.desde}–${b.hasta} · ${appName(b.app)}: ${b.actividad}`),'','Para revisar',...arr(A.atencion).map(a=>`${a.hora} · ${a.detalle}`)].join('\n');
}
async function genReport(){
  if(bitRepBusy)return; bitRepBusy=true; paintBit();
  const label=new Date(bitDay).toLocaleDateString('es-CO',{weekday:'long',day:'numeric',month:'long',year:'numeric'});
  try{const j=await invoke('generate_informe',{day:bitDay,to:dayEnd(bitDay),tz:new Date(bitDay).getTimezoneOffset(),label});bitRep=JSON.parse(j);toast('Informe listo')}
  catch(e){toast(''+e);if(/clave/i.test(''+e))refreshGem()}
  finally{bitRepBusy=false;paintBit()}
}
$('#bRep').onclick=()=>{
  if(!bitRepOpen){bitRepOpen=true;bitView=null;if(!bitRep&&gemOk)genReport();else paintBit();return}
  if(gemOk)genReport();else openTset();
};
$('#bBody').addEventListener('click',async e=>{
  if(e.target.closest('[data-x="tset"]')){openTset();return}
  const b=e.target.closest('[data-b]')?.dataset.b;
  if(b==='repback'){bitRepOpen=false;paintBit()}
  if(b==='repcopy'){try{await invoke('copy_text',{text:reportText()});toast('Informe copiado')}catch(err){toast(''+err)}}
});
async function makeTutorial(ids,ai){
  toast(ai?'Gemini está redactando el tutorial…':'Armando el tutorial…');
  try{await invoke('bitacora_tutorial',{ids,title:'',ai});toast('Tutorial guardado en Documentos\\Lienzo')}
  catch(e){toast(''+e)}
}
$('#bSet').onclick=()=>{bitSet=!bitSet;if(bsetup)bsetup.sel=null;paintBit()};
$('#bClose').onclick=closeBit;
$('#bit').addEventListener('click',e=>{if(e.target.id==='bit')closeBit()});
$('#bToggle').onclick=async()=>{
  if(!bsetup||!bsetup.apps.length){bitSet=true;paintBit();return}
  try{await invoke('set_bitacora',{on:!bitOn,apps:bsetup.apps,title:bsetup.title,days:bsetup.days})}catch(e){toast(''+e)}
};
$('#bDel').onclick=()=>{
  const n=bitRows.length, day=$('#bDay').textContent.toLowerCase();
  toast(`¿Borrar los ${n} clics de ${day}?`,'Borrar',async()=>{try{await invoke('delete_bitacora',{from:bitDay,to:dayEnd(bitDay)});toast('Día borrado');loadBit()}catch(e){toast(''+e)}});
};
document.addEventListener('click',e=>{if(e.target.closest('[data-bit]'))openBit()});
listen('bitacora-state',ev=>{bitOn=!!ev.payload;if(bsetup)bsetup.on=bitOn;paintBitBtn();if($('#bit').classList.contains('open'))paintBit();toast(bitOn?'Bitácora activa':'Bitácora en pausa')});
listen('bitacora-setup',()=>openBit(true));
listen('bitacora-added',ev=>{
  const r=ev.payload; if(!$('#bit').classList.contains('open')||bitSet||r.ts<bitDay||r.ts>=dayEnd(bitDay))return;
  bitRows.push(r); if(bitView==null)paintBit(); else $('#bCount').textContent=bitRows.length+' clics registrados';
});

/* ================= ACTUALIZACIONES ================= */
/* Revisa GitHub Releases al abrir y cada 6 horas; si hay versión nueva aparece el botón arriba */
let installing=false;
async function checkUpdate(manual){
  try{
    const u=await invoke('check_update');
    const b=$('#bUpdate');
    if(u){b.hidden=false;b.title=`Lienzo ${u.version} disponible (tienes la ${u.current})`+(u.notes?'\n\n'+u.notes:'');$('#updVer').textContent=u.version;if(manual)toast(`Lienzo ${u.version} disponible`,'Actualizar',installUpdate)}
    else{b.hidden=true;if(manual)toast('Ya tienes la última versión')}
  }catch(e){if(manual)toast('No se pudo revisar: '+e)}
}
async function installUpdate(){
  if(installing)return; installing=true;
  $('#bUpdate').classList.add('busy');toast('Descargando actualización… Lienzo se reiniciará solo');
  try{await invoke('install_update')}catch(e){installing=false;$('#bUpdate').classList.remove('busy');toast('No se pudo actualizar: '+e)}
}
$('#bUpdate').onclick=installUpdate;
listen('check-update',()=>checkUpdate(true));
listen('thumbs-ready',async()=>{try{const rows=await invoke('list_items');rows.forEach(r=>{const it=byId(r.id);if(it&&r.thumb)it.thumb=fileSrc(r.thumb)});refreshStill()}catch(e){}});

/* ================= INICIO ================= */
function render(){renderTimeline();renderCanvas();renderSide()}
window.addEventListener('resize',()=>{const was=compact;updateCompact();if(was!==compact)return;renderTimeline();relayout()});
/* ================= MENÚ DE CLIC DERECHO ================= */
const TONES=[['','Sin tono'],['gris','Gris'],['grafito','Grafito'],['azul','Azul'],['verde','Verde'],['ambar','Ámbar'],['rosa','Rosa'],['lila','Lila']];
let ctxActions={};
function mi(id,icon,label,on,cls){return `<button data-c="${id}" class="${on?'on ':''}${cls||''}">${icon?I[icon]:''}<span>${label}</span><span class="ck">${I.check}</span></button>`}
function sortMenu(){
  const free=sortMode==='free';
  return `<div class="lbl">Organizar tablero</div>`+Object.entries(SORTS).map(([k,v])=>mi('sort:'+k,k==='free'?'hand':'sort',v.l,sortMode===k)).join('')
    +(free?`<hr>`+mi('resetfree','sort','Acomodar todo de nuevo'):'');
}
function toneRow(it){return `<div class="lbl">Tono</div><div class="tones">`+TONES.map(([k,l])=>`<span data-tone="${k}" data-c="tone:${k}" class="${k?'':'nt'}${(it.tone||'')===k?' on':''}" title="${l}"></span>`).join('')+`</div>`}
function openCtx(x,y,html){
  const m=$('#ctx'); m.innerHTML=html; m.classList.add('open');
  const w=m.offsetWidth,h=m.offsetHeight;
  m.style.left=Math.max(6,Math.min(x,innerWidth-w-6))+'px'; m.style.top=Math.max(6,Math.min(y,innerHeight-h-6))+'px';
  m.style.transformOrigin=(x>innerWidth-w-6?'right ':'left ')+(y>innerHeight-h-6?'bottom':'top');
}
function closeCtx(){$('#ctx').classList.remove('open')}
document.addEventListener('contextmenu',e=>{
  if(e.target.closest('textarea,input'))return;
  e.preventDefault(); closeCtx(); $('#opPop').classList.remove('open');
  const card=e.target.closest('.card'), pin=e.target.closest('#sideList .pin');
  let html='';
  if(card&&isSess(byId(card.dataset.id))){
    const it=byId(card.dataset.id); ctxActions={it,el:card};
    html=mi('view','text','Abrir')+mi('edit','pencil','Renombrar')+(it.status==='transcrita'?mi('acta','spell','Acta'):'')+(it.status==='grabada'||it.status==='transcrita'?mi('transcribe','spell',it.status==='transcrita'?'Volver a transcribir':'Transcribir'):'')+mi('folder','folder','Abrir la carpeta del audio')+'<hr>'+toneRow(it)+'<hr>'+mi('del','trash','Mover a la papelera',false,'danger')+(compact?'':'<hr>'+sortMenu());
  }else if(card){
    const it=byId(card.dataset.id); ctxActions={it,el:card};
    html=mi('paste','pasteIn','Pegar en la app anterior')+mi('copy','copy','Copiar')+mi('edit','pencil','Editar')
      +mi('pin','pin',it.pinned?'Quitar de fijados':'Fijar')+`<hr>`+toneRow(it)+`<hr>`+mi('del','trash','Mover a la papelera',false,'danger')
      +(compact?'':`<hr>`+sortMenu());
  }else if(pin){
    const it=byId(pin.dataset.id); ctxActions={it,el:null};
    html=state.tab==='trash'
      ?mi('restore','restore','Restaurar')+mi('kill','x','Eliminar definitivamente',false,'danger')
      :mi('paste','pasteIn','Pegar en la app anterior')+mi('copy','copy','Copiar')+mi('edit','pencil','Editar')+`<hr>`+toneRow(it)+`<hr>`+mi('pin','pin','Quitar de fijados');
  }else if(e.target.closest('#board')){
    ctxActions={};
    html=mi('new','plus','Nueva nota')+mi('tset','sliders','Ajustes de transcripción')+(compact?'':`<hr>`+sortMenu());
  }else return;
  openCtx(e.clientX,e.clientY,html);
});
$('#ctx').addEventListener('click',async e=>{
  const b=e.target.closest('[data-c]'); if(!b)return;
  const [c,arg]=b.dataset.c.split(':'), {it,el}=ctxActions;
  if(c==='tone'){
    patch(it,{tone:arg});
    document.querySelectorAll(`[data-id="${it.id}"]`).forEach(x=>{if(arg)x.dataset.tone=arg;else delete x.dataset.tone});
    $('#ctx').querySelectorAll('.tones span').forEach(s=>s.classList.toggle('on',s.dataset.tone===arg));
    return;
  }
  closeCtx();
  if(c==='sort')setSort(arg);
  else if(c==='resetfree')resetFree();
  else if(c==='new')openEditor('new');
  else if(c==='paste')pasteItem(it);
  else if(c==='copy')copyItem(it);
  else if(c==='edit')openEditor(it.id);
  else if(c==='pin'){pinIt(it,!it.pinned);document.querySelectorAll(`.card[data-id="${it.id}"]`).forEach(x=>x.classList.toggle('pinned',it.pinned));toast(it.pinned?'Fijado':'Quitado de fijados')}
  else if(c==='del')trashItem(it,el);
  else if(c==='restore'){patch(it,{deletedAt:null});fresh=it.id;refreshStill();toast('Restaurado')}
  else if(c==='kill')killItem(it);
  else if(c==='folder')openFolder(it);
  else if(c==='view')openViewer(it.id);
  else if(c==='acta'){viewing=it.id;vtab='a';openViewer(it.id)}
  else if(c==='exp')exportSess(it,arg);
  else if(c==='tut')makeTutorial(ctxActions.tut,arg==='ai');
  else if(c==='transcribe')startTranscribe(it);
  else if(c==='tset')openTset();
});
document.addEventListener('pointerdown',e=>{if(!e.target.closest('#ctx'))closeCtx()},true);
document.addEventListener('keydown',e=>{if(e.key==='Escape'&&$('#ctx').classList.contains('open')){closeCtx();e.stopImmediatePropagation()}},true);
window.addEventListener('blur',closeCtx);
$('#board').addEventListener('scroll',closeCtx);
(async()=>{
  try{
    const [rows,st,ss,rs]=await Promise.all([invoke('list_items'),invoke('status'),invoke('list_sessions'),invoke('rec_status')]);
    items=rows.map(fromDb).concat(ss.map(fromSession)); rec=rs; paintRec(); refreshGem();
    invoke('bitacora_setup').then(s=>{bitOn=s.on;bsetup=s;paintBitBtn()}).catch(()=>{}); paused=st.paused; keepOpen=st.keepOpen;
    let mig=null; try{mig=localStorage.getItem('lienzo-tablero-v2')}catch(e){}
    if(!mig){items.filter(i=>i.x!=null).forEach(i=>patch(i,{x:null,y:null}));try{localStorage.setItem('lienzo-tablero-v2','1')}catch(e){}}
    if(st.hotkeys&&st.hotkeys.length)$('#hint').textContent=`Ábrelo con ${st.hotkeys.join(' o ')} · `+$('#hint').textContent;
    if(st.version)$('.brand').title='Lienzo '+st.version+(st.portable?' · portable':'');
    setTimeout(()=>checkUpdate(false),8000); setInterval(()=>checkUpdate(false),6*H);
  }catch(e){toast('Modo vista previa (sin Tauri)')}
  await loadSystemColors();
  try{if(localStorage.getItem('lienzo-compacto')==='1')invoke('set_compact',{value:true})}catch(e){}
  paintToggles(); updateCompact(); render();
})();
