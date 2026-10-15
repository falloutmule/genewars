from pathlib import Path
import importlib.util,struct,json,hashlib,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('source',P/'research/variants/krell-mule-callback-review.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
fixture=P/'private/review/krell-mule-callbacks-058/frame-1274-slot-1-blocked-before';base=(fixture/'creatures.bin').read_bytes()[124:248];backing=(fixture/'backing.bin').read_bytes();work=P/'private/review/krell-mule-1274-pure'/uuid.uuid4().hex;work.mkdir(parents=True)
rust=P/'rust/crates/genewars-runtime/src/creature_world.rs';text=rust.read_text();funcs=[]
for sig in ['fn motion_word(', 'fn motion_put(', 'fn draw(', 'pub fn plan_mule_retry_wait(']:
 start=text.index(sig);end=text.index('{',start);depth=1;i=end+1
 while depth:depth+=(text[i]=='{')-(text[i]=='}');i+=1
 funcs.append(text[start:i])
h=work/'probe.rs';exe=work/'probe.exe';target=P/'private/build/rust-target/debug'
h.write_text('extern crate genewars_runtime;pub use genewars_runtime::blueprint;use genewars_runtime::creature_world::{CreatureError,InitialMuleChoice,CreatureRngPurpose,CreatureRngDraw};type CreatureRecord=[u8;124];'+''.join(funcs)+r'''use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for c in b.chunks_exact(131){let rng=blueprint::BlueprintRng{seed:u16::from_le_bytes(c[124..126].try_into().unwrap()),ring:c[126],draw_count:u32::from_le_bytes(c[127..131].try_into().unwrap())};let q=plan_mule_retry_wait(c[..124].try_into().unwrap(),rng).unwrap();out.extend(q.after);out.extend(q.rng_after.seed.to_le_bytes());out.push(q.rng_after.ring);out.extend(q.rng_after.draw_count.to_le_bytes());}std::io::stdout().write_all(&out).unwrap();}''')
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','-Awarnings',str(h),'--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-L',f'dependency={target}/deps','-o',str(exe)],env=environment(),check=True)
inputs=bytearray();expected=bytearray();cases=[]
for state in [1,8]:
 for seed in [0,5428,30553,65535]:
  a=bytearray(base);a[94]=state;a[87]=2 if state==1 else 0;a[93]=0;a[96]&=~2;struct.pack_into('<H',a,73,0x150)
  mem={0x1542be+124+i:v for i,v in enumerate(a)};m.put(mem,0x1542be+124+35,4,0x111fec);m.put(mem,0x11872c,1,7);m.put(mem,0x118782,4,23)
  plant=struct.unpack_from('<h',a,85)[0];mem[0x15d40e+plant*49+46]=0
  model=m.Source(backing);model.code[0x5b100]=('XOR',['EAX','EAX']);model.code[0x2e0000]=('RET',[]);model.code[0x5e3b1]=('RET',[]);model.next[0x5b100]=0x2e0000
  r,_,_=model.run(entry=0x5e1d8,state_before=mem,registers={'ECX':0x1542be+124,'ESI':0},seed=seed)
  after=bytearray(model.load(0x1542be+124+i,1)for i in range(124));after[35:39]=a[35:39];rr=struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4));inputs+=a+struct.pack('<HBI',seed,7,23);expected+=after+rr;cases.append({'state':state,'seed':seed,'instructions':r['steps']})
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout;assert actual==expected,[(i//131,i%131,x,y)for i,(x,y)in enumerate(zip(actual,expected))if x!=y][:10]
r={'status':'passed','cases':cases,'compared_bytes':len(actual),'scope':'Extracted exact productionpurefunction versusliteral5E1D8..5E3B1;5B100false explicitcallerport;plantbit04explicitlyclear;syntheticidle/plantvariations,notfullcallback','production_sha256':hashlib.sha256(rust.read_bytes()).hexdigest(),'extracted_sha256':hashlib.sha256(''.join(funcs).encode()).hexdigest()};Path(__file__).with_suffix('.json').write_text(json.dumps(r,indent=2));print(json.dumps({'status':'passed','cases':len(cases),'bytes':len(actual)}))
