"""Compare captured owned Rust callbacks against full literal source sequence."""
from pathlib import Path
import importlib.util,contextlib,io,json,pickle,hashlib,struct,sys,uuid
P=Path(__file__).resolve().parents[2];capture=P/sys.argv[1];sp=importlib.util.spec_from_file_location('terminal',Path(__file__).with_name('krell_owned_mule_terminal_callbacks.py'));q=importlib.util.module_from_spec(sp)
with contextlib.redirect_stdout(io.StringIO()):sp.loader.exec_module(q)
rows=[];fail=None;total=0;w=P/'private/review/owned-mule-world-comparison'/uuid.uuid4().hex;w.mkdir(parents=True)
for frame in range(1,19):
 if not(capture/f'frame-{frame}').exists():fail={'frame':frame,'reason':'missing Rust callback export'};break
 memory=pickle.loads((q.w/f'{frame}-after.pickle').read_bytes());diff={}
 for name,address,stride in [('map',0x11c2be,0),('creatures',0x1542be,124),('buildings',0x1645ae,67),('players',0x118d74,0)]:
  actual=(capture/f'frame-{frame}'/(name+'.bin')).read_bytes();expected=bytearray(memory.get(address+i,0)for i in range(len(actual)))
  if stride:
   for off in range(0,len(actual),stride):expected[off+31:off+39]=actual[off+31:off+39]
  ds=[{'offset':hex(i),'rust':a,'source':b}for i,(a,b)in enumerate(zip(actual,expected))if a!=b];diff[name]={'count':len(ds),'first':ds[:30]};total+=len(actual)
 actual=(capture/f'frame-{frame}'/'rng.bin').read_bytes();expected=bytes(memory.get(x,0)for x in [0x11878a,0x11878b,0x11872c,*range(0x118782,0x118786)]);diff['rng']={'rust':actual.hex(),'source':expected.hex(),'count':0 if actual==expected else 1};total+=7
 rows.append({'frame':frame,'differences':diff})
 if any(x['count']for x in diff.values()):fail={'frame':frame,'reason':'afterimage mismatch'};break
files=[P/'rust/crates/genewars-runtime/src'/x for x in ['owned_mule_callback.rs','owned_mule_ai.rs','owned_mule_common_choice.rs','actor_world/owned_mule_updates.rs','creature_world.rs','actor_world/creature_updates.rs']]
r={'status':'passed'if fail is None else ('blocked' if fail['reason']=='missing Rust callback export' else 'error'),'frames':len(rows),'compared_bytes':total,'failure':fail,'rows':rows,'capture':capture.relative_to(P).as_posix(),'source_fixture':q.w.relative_to(P).as_posix(),'private_run':w.relative_to(P).as_posix(),'scope':'Full owned callback sequence plus real opcode11 handler on synthetic terminal/ejection fixture, not authentic Krell initialization or native acceptance. Pointer31:39 normalized only.','pins':{p.relative_to(P).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in files+[Path(__file__),Path(__file__).with_name('krell_owned_mule_terminal_callbacks.py')]+list(capture.rglob('*.bin'))}}
(w/'report.json').write_text(json.dumps(r,indent=2)+'\n');(P/'research/campaign/owned-mule-world-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({k:v for k,v in r.items()if k not in ['pins','rows']}));assert not fail,rows[-1]if rows else fail
