from pathlib import Path
import subprocess,sys,json,hashlib
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
W=P/'private/review/krell-family8-state2-travel';base=P/'private/review/family8-state4'
for name in ['angles.bin','bounce.bin','ethereal.anb']:(W/name).write_bytes((base/name).read_bytes())
(W/'zones.bin').write_bytes((P/'private/review/krell-mule-callbacks-064/frame-1456-slot-2-family8-before/backing.bin').read_bytes()[0x2ff8:0x300b])
s=(P/'research/variants/family8-state1-review.rs').read_text().replace('use genewars_runtime::{family8_motion::*,blueprint::BlueprintRng,actor_activation::TrigTables,physics_tables::PhysicsTables};','use family8_motion::*;use genewars_runtime::{blueprint::BlueprintRng,actor_activation::TrigTables,physics_tables::PhysicsTables};')
s=s.replace('let mut actual=Vec::new();','let pth=p.clone();let mut actual=Vec::new();')
s=s.replace('plan_selector1_state1_travel(&before,&map,frame,1632,&physics,&trig,&animation,rng)','plan_selector1_state2_travel(&before,&map,frame,2000,&physics,&trig,&animation,rng,&fs::read(p.join("zones.bin")).unwrap()).map(|p|{let initial=fs::read(pth.join("zones.bin")).unwrap();let mut expected=initial.clone();if i16::from_le_bytes(p.onward.motion.after[100..102].try_into().unwrap())>0{expected[18]=expected[18].wrapping_add(1);}assert_eq!(p.zones_after,expected);p.onward})')
a=s.index('  for offset in');b=s.index(' }fs::',a);s=s[:a]+s[b:]
runtime=P/'rust/crates/genewars-runtime/src';prefix='extern crate genewars_runtime;extern crate genewars_core;pub use genewars_runtime::*;'+f'#[path="{runtime.as_posix()}/family8_motion.rs"] mod family8_motion;#[path="{runtime.as_posix()}/family8_zone_target.rs"] mod family8_zone_target;'
angle_text=(runtime/'actor_motion.rs').read_text();start=angle_text.index('pub(crate) fn angle(');end=angle_text.index('\n}\n',start)+2
prefix+='mod actor_motion{'+angle_text[start:end]+'}'
h=W/'probe.rs';h.write_text(prefix+s);t=P/'private/build/rust-target/debug';exe=W/'probe.exe'
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','-Awarnings',str(h),'--extern',f'genewars_runtime={t}/libgenewars_runtime.rlib','--extern',f'genewars_core={t}/libgenewars_core.rlib','-L',f'dependency={t}/deps','-o',str(exe)],env=environment(),check=True)
subprocess.run([str(exe),str(W)],check=True);total=0
for x,y in [('actual.bin','expected.bin'),('actual-particles.bin','particles.bin')]:
 a=(W/x).read_bytes();b=(W/y).read_bytes();assert a==b,(x,[(i,x,y)for i,(x,y)in enumerate(zip(a,b))if x!=y][:20]);total+=len(a)
assert (W/'continuation/prefix.bin').read_bytes()[0x2ff8+18]==(W/'zones.bin').read_bytes()[18]+1
r={'status':'passed','cases':214,'compared_bytes':total,'scope':'Isolated state2 travel through arrival; particlefactory explicit port; pure parent/RNG/request proof, not scheduler. Zone afterimage checked each callback; final source occupancy increment matches','production_sha256':hashlib.sha256((runtime/'family8_motion.rs').read_bytes()).hexdigest()};Path(__file__).with_suffix('.json').write_text(json.dumps(r,indent=2));print(json.dumps(r))
