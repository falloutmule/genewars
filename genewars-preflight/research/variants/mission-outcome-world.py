"""Replay frozen ordinary commands and source-check read-only live-world outcome adapter."""
from pathlib import Path
import sys,hashlib,json,shutil,subprocess,uuid,struct,importlib.util
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
origin=P/'private/review/distributor-stall/0818cb7f7b104aca993a19e9f6eaf366';W=P/'private/review/mission-outcome-world'/uuid.uuid4().hex;W.mkdir(parents=True);pins={}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def run(a):
 r=subprocess.run(a,cwd=P,env=environment(),capture_output=True,text=True,timeout=180);assert r.returncode==0,(r.stdout,r.stderr);return r.stdout
shutil.copytree(origin/'source',W/'source');shutil.copytree(origin/'support',W/'support');shutil.copyfile(origin/'catalog.tsv',W/'catalog.tsv')
for f in (W/'source').rglob('*.rs'):pins[str(f.relative_to(W))]=sha(f)
for relative in ['mission_outcome.rs','actor_world/mission_outcome.rs']:
 p=P/'rust/crates/genewars-runtime/src'/relative;shutil.copyfile(p,W/'source/runtime'/relative);pins['adapter/'+relative]=sha(p)
for name,line in [('lib.rs','pub mod mission_outcome;'),('actor_world.rs','mod mission_outcome;pub use mission_outcome::OutcomeRuntimeInputs;')]:
 p=W/'source/runtime'/name;s=p.read_text();p.write_text(s+'\n'+line+'\n')
s=(P/'scripts/desktop/distributor_stall_probe.rs').read_text();s=s.replace('fs::write(dir.join("jobs.bin"),s.fixture_jobs()).unwrap();','''fs::write(dir.join("jobs.bin"),s.fixture_jobs()).unwrap();
 let before=w.clone();let input=genewars_runtime::actor_world::OutcomeRuntimeInputs{flags:0x3e,mode:0,participating_players:1,selected_player:0,frame:label.parse().unwrap()};let q=w.plan_mission_outcome(input).unwrap();assert_eq!(*w,before);let mut bytes=q.scan.players.concat();bytes.push(q.flags_after);bytes.push(q.scan.winner.unwrap_or(255));bytes.push(match q.outcome{genewars_runtime::mission_outcome::MissionOutcome::Running=>0,genewars_runtime::mission_outcome::MissionOutcome::Won{..}=>1,genewars_runtime::mission_outcome::MissionOutcome::Lost{..}=>2});bytes.push(q.scan.capability_queries.len()as u8);bytes.extend(q.scan.capability_queries);fs::write(dir.join("outcome.bin"),bytes).unwrap();let mut bindings=Vec::new();for id in 0..20{let b=w.actors().bindings(id).unwrap().blueprint;bindings.extend(b.map(|b|[1,b.player_id,b.blueprint_index]).unwrap_or([0;3]));}fs::write(dir.join("bindings.bin"),bindings).unwrap();''')
s=s[:s.index('g.start_selected_building_type(3).unwrap();capture(&g,out,"1489-assigned");')]+'println!("captured live1289/1489 unchanged outcome adapter");}\n';s=s.replace('if step>=1479{capture','if step==1489{capture');(W/'main.rs').write_text(s)
rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe')
for c in ['assets','core','runtime','render']:
 cmd=[rustc,'--edition=2021','--cap-lints=allow','-O','--crate-name',f'genewars_{c}','--crate-type','rlib',str(W/f'source/{c}/lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{c}.rlib')]
 for dep in (['assets','core']if c=='runtime'else['assets']if c=='render'else[]):cmd+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
 run(cmd)
cmd=[rustc,'--edition=2021','--cap-lints=allow','-O',str(W/'main.rs'),'-L',f'dependency={W}','-o',str(W/'probe.exe')]
for c in ['assets','core','runtime','render']:cmd+=['--extern',f'genewars_{c}={W}/libgenewars_{c}.rlib']
run(cmd);print(run([str(W/'probe.exe'),str(P),str(W)]),flush=True)
spec=importlib.util.spec_from_file_location('literal',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);source=m.Source();files=[P/'private/ghidra/font-menu-state/instructions-00072ed0.txt',P/'private/ghidra/construction-command/instructions-0008ab30.txt',P/'private/ghidra/mission-array-refs/instructions-000805e0.txt'];source.code.update(m.m.Model(files,b'').code);keys=sorted(source.code);source.next=dict(zip(keys,keys[1:]));stats=[]
for label in ['1289','1489']:
 d=W/label
 for f in ['backing.bin','players.bin','actors.bin','buildings.bin','effects.bin','rng.bin','jobs.bin']:assert (d/f).read_bytes()==(origin/label/f).read_bytes(),(label,f)
 backing=(d/'backing.bin').read_bytes();players=(d/'players.bin').read_bytes();actors=bytearray((d/'actors.bin').read_bytes());bindings=(d/'bindings.bin').read_bytes()
 for i in range(20):
  valid,owner,bp=bindings[i*3:i*3+3]
  if valid:struct.pack_into('<I',actors,i*200+192,0x118d74+owner*1805+0x16+bp*36)
 state={0x118d60+i:v for i,v in enumerate(backing[:0x355e])};state.update({0x118d74+i:v for i,v in enumerate(players)});state.update({0x16360e+i:v for i,v in enumerate(actors)});state[0x118770]=0x3e;state[0x118968]=0;state[0x11878c]=1
 for i in range(4):state[0x18386a+i*0x6be]=2
 report,_,trace=source.run(counter=int(label),state_before=state,entry=0x72ed0,registers={'EAX':0x118d60});actual=(d/'outcome.bin').read_bytes();expected=bytes(source.load(0x118d74+i,1)for i in range(7220))+bytes([source.load(0x118770,1),255,0,trace.count(0x805e0)])+bytes([0]*trace.count(0x805e0));assert actual==expected;assert 0x805e0 in trace;stats.append(dict(tick=int(label),outcome='running',literal_instructions=report['steps'],actual_capability=True,capture_pins={f.name:sha(f)for f in d.iterdir()if f.is_file()}))
receipt=dict(status='passed',private_run=W.relative_to(P).as_posix(),original_capture=origin.relative_to(P).as_posix(),cases=stats,production_pins=pins,source_pins={f.relative_to(P).as_posix():sha(f)for f in source.files+files+[P/'research/variants/initial-creature-source.py',P/'research/variants/actor-environment-terrain-target.py',P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin',P/'scripts/desktop/distributor_stall_probe.rs',Path(__file__)]},scope='Unmodified frozen ordinary-command worlds at1289/1489; explicit development flags3E/mode0/active1, actual805E0 with captured actor blueprint bindings; read-only adapter comparison, not mission acceptance')
(W/'result.json').write_text(json.dumps(receipt,indent=2));Path(__file__).with_suffix('.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(dict(status='passed',private_run=receipt['private_run'])))
