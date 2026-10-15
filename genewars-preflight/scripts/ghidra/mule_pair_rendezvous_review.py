"""Literal693AE pair slice versus pure planner, with complete plant reservations."""
from pathlib import Path
import importlib.util,struct,json,hashlib,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('source',P/'research/variants/krell-mule-callback-review.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
fixture=P/'private/review/krell-rendezvous-boundaries/6582ce34187d4230a4135633e70ae64a';before=fixture/'69a17';after=fixture/'69b8d';current=(before/'creature-2.bin').read_bytes();peer=(before/'creature-0.bin').read_bytes();plants=(before/'plants.bin').read_bytes();rng=(before/'rng.bin').read_bytes()
backing=(P/'private/review/krell-mule-callbacks-018/frame-651-slot-2-before/backing.bin').read_bytes();work=P/'private/review/mule-pair-rendezvous'/uuid.uuid4().hex;work.mkdir(parents=True)
rust=P/'rust/crates/genewars-runtime/src/mule_pair_rendezvous.rs';world=P/'rust/crates/genewars-runtime/src/creature_world.rs';text=world.read_text();funcs=[]
for signature in ['fn mule_skill_percent(', 'pub(crate) fn mule_requested_speed(']:
 start=text.index(signature);end=text.index('\n}',start)+2;funcs.append(text[start:end])
h=work/'probe.rs';exe=work/'probe.exe'
h.write_text('mod creature_world{type CreatureRecord=[u8;124];'+''.join(funcs)+'}'+f'#[path="{rust.as_posix()}"]mod pair;'+r"""use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut output=Vec::new();for c in b.chunks_exact(250){let q=pair::plan_rendezvous(c[..124].try_into().unwrap(),c[124..248].try_into().unwrap(),u16::from_le_bytes(c[248..250].try_into().unwrap()));if std::env::args().any(|a|a=="reject"){assert!(q.is_err());output.push(1);continue;}let q=q.unwrap();output.extend(q.current_after);output.extend(q.peer_after);}std::io::stdout().write_all(&output).unwrap();}""")
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(h),'-o',str(exe)],env=environment(),check=True)
inputs=bytearray();expected=bytearray();cases=[];plant_bytes=0
for cell in [None,0,16383]:
 for skill in [None,0,100]:
  a=bytearray(current);b=bytearray(peer)
  if cell is not None:
   x=(cell%128)*256;y=(cell//128)*256;struct.pack_into('<hh',a,7,x+27,y+50);struct.pack_into('<hh',b,7,x+200,y+80)
  if skill is not None:a[0x76]=skill;b[0x76]=100-skill;a[0x7b]=127;b[0x7b]=255
  saved=((struct.unpack_from('<h',b,9)[0]>>8)*128)+(struct.unpack_from('<h',b,7)[0]>>8)
  state={0x1542be+248+i:v for i,v in enumerate(a)};state.update({0x1542be+i:v for i,v in enumerate(b)});state.update({0x15d40e+i:v for i,v in enumerate(plants)})
  ebp=0x2fff00
  for offset,value in [(0x11a,0x1542be+248+7),(0x122,0x1542be+248+0x4d),(0x11e,0x1542be+248+0x50)]:state.update({ebp+offset+i:v for i,v in enumerate(struct.pack('<I',value))})
  seed,ring,draws=struct.unpack('<HBI',rng);state[0x11872c]=ring;state.update({0x118782+i:v for i,v in enumerate(struct.pack('<I',draws))})
  model=m.Source(backing);model.code[0x69b8d]=('RET',[])
  r,_,trace=model.run(seed=seed,state_before=state,entry=0x69a17,registers={'ESI':0x1542be+248,'EDI':0x1542be,'EBP':ebp},instruction_budget=10000)
  aa=bytes(model.load(0x1542be+248+i,1)for i in range(124));bb=bytes(model.load(0x1542be+i,1)for i in range(124));pp=bytes(model.load(0x15d40e+i,1)for i in range(len(plants)))
  assert pp==plants;assert r['draws']==0;assert struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))==rng
  if cell is None and skill is None:assert aa==(after/'creature-2.bin').read_bytes()and bb==(after/'creature-0.bin').read_bytes()
  inputs+=a+b+struct.pack('<H',saved);expected+=aa+bb;plant_bytes+=len(pp);cases.append(dict(cell=cell,skill=skill,instructions=r['steps']))
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:20]
negative=bytearray();saved=(struct.unpack_from('<h',peer,9)[0]>>8)*128+(struct.unpack_from('<h',peer,7)[0]>>8)
for off,value in [(6,3),(24,0),(94,1),(97,0x60),(122,5),(87,2)]:
 a=bytearray(current);a[off]=value;negative+=a+peer+struct.pack('<H',saved)
negative+=current+peer+struct.pack('<H',(saved+1)%16384)
assert subprocess.run([str(exe),'reject'],input=negative,capture_output=True,check=True).stdout==bytes([1])*7
(work/'inputs.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
r=dict(status='passed',cases=cases,record_bytes=len(actual),unchanged_plant_bytes=plant_bytes,negative_cases=7,private_run=str(work.relative_to(P)),pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [rust,world,Path(__file__),*before.glob('*.bin'),*after.glob('*.bin')]},source_pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in model.files},scope='Rendezvous69A17..69B8D bothrecords; ownspeedhalf independently both,0RNG,plantsunchanged. Callerproves sourcebranch/savedcomparisoncell; no lifecycle claim.')
(P/'research/variants/mule-pair-rendezvous-rust-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=len(cases),record_bytes=len(actual),plant_bytes=plant_bytes)))
