from pathlib import Path
import importlib.util,struct,json,hashlib,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('source',P/'research/variants/krell-mule-callback-review.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
fixture=P/'private/review/krell-mule-callbacks-058/frame-1274-slot-1-blocked-before';base=(fixture/'creatures.bin').read_bytes()[124:248];backing=(fixture/'backing.bin').read_bytes();work=P/'private/review/krell-mule-1279-pure'/uuid.uuid4().hex;work.mkdir(parents=True)
rust=P/'rust/crates/genewars-runtime/src/creature_world.rs';text=rust.read_text();funcs=[]
for sig in ['fn motion_word(', 'fn motion_put(', 'fn draw(', 'pub fn plan_mule_feeding_query_prefix(', 'fn plan_mule_plant_feed_inner(']:
 start=text.index(sig);end=text.index('{',start);depth=1;i=end+1
 while depth:depth+=(text[i]=='{')-(text[i]=='}');i+=1
 funcs.append(text[start:i])
h=work/'probe.rs';exe=work/'probe.exe';target=P/'private/build/rust-target/debug'
h.write_text('extern crate genewars_runtime;pub use genewars_runtime::blueprint;use genewars_runtime::creature_world::{CreatureError,InitialMuleChoice,CreatureRngPurpose,CreatureRngDraw};type CreatureRecord=[u8;124];'+''.join(funcs)+r'''use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for c in b.chunks_exact(131){let rng=blueprint::BlueprintRng{seed:u16::from_le_bytes(c[124..126].try_into().unwrap()),ring:c[126],draw_count:u32::from_le_bytes(c[127..131].try_into().unwrap())};let q=plan_mule_feeding_query_prefix(c[..124].try_into().unwrap(),8,rng,true).unwrap();out.extend(q.after);out.extend(q.rng_after.seed.to_le_bytes());out.push(q.rng_after.ring);out.extend(q.rng_after.draw_count.to_le_bytes());}std::io::stdout().write_all(&out).unwrap();}''')
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','-Awarnings',str(h),'--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-L',f'dependency={target}/deps','-o',str(exe)],env=environment(),check=True)
q=P/'private/review/krell-pair-boundaries/349b2055a53f43a78a4768b9b4198a38'
a=bytearray((q/'688b8/creature-0.bin').read_bytes());expected=bytearray((q/'693ae/creature-0.bin').read_bytes());expected[31:39]=a[31:39];a[0:6]=expected[0:6] # explicit owned retirement unlink port before pure numeric plan
inputs=a+(q/'688b8/rng.bin').read_bytes();expected+=(q/'693ae/rng.bin').read_bytes()
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i,x,y)for i,(x,y)in enumerate(zip(actual,expected))if x!=y]
r={'status':'passed','cases':1,'compared_bytes':len(actual),'scope':'Actual1279 literal688B8 to693AE currentrecord/RNG versus extracted production feeding prefix; nutrition8, retiredtrue; globalplant retirement/peer cleanup and linkbytes0..5 are explicit caller obligations','source_boundary':str(q),'production_sha256':hashlib.sha256(rust.read_bytes()).hexdigest(),'extracted_sha256':hashlib.sha256(''.join(funcs).encode()).hexdigest()};Path(__file__).with_suffix('.json').write_text(json.dumps(r,indent=2));print(json.dumps(r))
