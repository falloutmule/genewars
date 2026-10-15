"""Literal693AE pair slice versus pure planner, with complete plant reservations."""
from pathlib import Path
import importlib.util,struct,json,hashlib,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('source',P/'research/variants/krell-mule-callback-review.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
fixture=P/'private/review/krell-pair-boundaries/25c5283f90b7410c9354ca95be3c474a';before=fixture/'693ae';after=fixture/'694d3';current=(before/'creature-2.bin').read_bytes();peer=(before/'creature-0.bin').read_bytes();plants=(before/'plants.bin').read_bytes();rng=(before/'rng.bin').read_bytes()
backing=(P/'private/review/krell-mule-callbacks-018/frame-651-slot-2-before/backing.bin').read_bytes();work=P/'private/review/mule-pair-init'/uuid.uuid4().hex;work.mkdir(parents=True)
rust=P/'rust/crates/genewars-runtime/src/mule_pair_init.rs';world=P/'rust/crates/genewars-runtime/src/creature_world.rs';text=world.read_text();funcs=[]
for signature in ['fn mule_skill_percent(', 'pub(crate) fn mule_requested_speed(']:
 start=text.index(signature);end=text.index('\n}',start)+2;funcs.append(text[start:end])
h=work/'probe.rs';exe=work/'probe.exe'
h.write_text('mod creature_world{type CreatureRecord=[u8;124];'+''.join(funcs)+'}'+f'#[path="{rust.as_posix()}"]mod pair;'+r"""use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut output=Vec::new();for c in b.chunks_exact(248){let q=pair::plan_pair_init(c[..124].try_into().unwrap(),c[124..].try_into().unwrap());if std::env::args().any(|a|a=="reject"){assert!(q.is_err());output.push(1);continue;}let q=q.unwrap();output.extend(q.current_after);output.extend(q.peer_after);output.push(q.release_plant_reservations.len()as u8);for id in q.release_plant_reservations{output.extend(id.to_le_bytes());}}std::io::stdout().write_all(&output).unwrap();}""")
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(h),'-o',str(exe)],env=environment(),check=True)
inputs=bytearray();expected=bytearray();cases=[];plant_bytes=0
for skill in [None,0,50,100]:
 for null in [False,True]:
  a=bytearray(current);b=bytearray(peer)
  if skill is not None:a[0x76]=skill;b[0x76]=100-skill;a[0x7b]=127;b[0x7b]=255
  if null:a[0x57]=0;b[0x57]=0
  state={0x1542be+248+i:v for i,v in enumerate(a)};state.update({0x1542be+i:v for i,v in enumerate(b)});state.update({0x15d40e+i:v for i,v in enumerate(plants)})
  ebp=0x2fff00;state.update({ebp-0xf72+i:v for i,v in enumerate(struct.pack('<I',0x1542be))});seed,ring,draws=struct.unpack('<HBI',rng);state[0x11872c]=ring;state.update({0x118782+i:v for i,v in enumerate(struct.pack('<I',draws))})
  model=m.Source(backing);model.code[0x694d3]=('RET',[])
  r,_,trace=model.run(seed=seed,state_before=state,entry=0x693ae,registers={'ESI':0x1542be+248,'EBP':ebp},instruction_budget=10000)
  aa=bytes(model.load(0x1542be+248+i,1)for i in range(124));bb=bytes(model.load(0x1542be+i,1)for i in range(124));pp=bytes(model.load(0x15d40e+i,1)for i in range(len(plants)))
  ids=[]
  for record in [a,b]:
   if record[0x57]==2:ids.append(struct.unpack_from('<H',record,0x55)[0])
  planned=bytearray(plants)
  for id in ids:planned[id*49+46]&=0xfe
  assert planned==pp;assert r['draws']==0;assert struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))==rng
  if skill is None and not null:assert aa==(after/'creature-2.bin').read_bytes()and bb==(after/'creature-0.bin').read_bytes()and pp==(after/'plants.bin').read_bytes()
  inputs+=a+b;expected+=aa+bb+bytes([len(ids)])+b''.join(struct.pack('<H',id)for id in ids);plant_bytes+=len(pp);cases.append(dict(skill=skill,null_targets=null,instructions=r['steps']))
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:20]
negative=bytearray()
for off,value in [(6,3),(24,0),(94,3),(96,2),(122,5),(87,1)]:
 a=bytearray(current);a[off]=value;negative+=a+peer
bad=bytearray(current);bad[15:17]=peer[15:17];negative+=bad+peer
assert subprocess.run([str(exe),'reject'],input=negative,capture_output=True,check=True).stdout==bytes([1])*7
(work/'inputs.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
r=dict(status='passed',cases=cases,record_and_release_bytes=len(actual),plant_bytes=plant_bytes,negative_cases=7,private_run=str(work.relative_to(P)),pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [rust,world,Path(__file__),*before.glob('*.bin'),*after.glob('*.bin')]},source_pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in model.files},scope='Pair slice693AE..694D3, both124byte records and fullplantarray; zeroRNG. Current-species speed appliedboth, peertraits deliberatelydifferent. Caller qualification/selection and laterstate3 lifecycle excluded.')
(P/'research/variants/mule-pair-init-rust-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=len(cases),record_bytes=len(actual),plant_bytes=plant_bytes)))
