"""Literal72E50 carried-player/isolated-actor teardown on captured ordinary worlds."""
from pathlib import Path
import hashlib,json,struct,subprocess,uuid,sys,shutil,random
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
origin=P/'private/review/mission-outcome-world/632fb3f7961d4661938f9858ccfb0314';W=P/'private/review/result-carry'/uuid.uuid4().hex;W.mkdir(parents=True)
original=P/'research/variants/initial-creature-source.py';text=original.read_text().replace('(0x11873d,1,0),','');needle="    elif target==0x982c0:";assert text.count(needle)==1;text=text.replace(needle,"""    elif target==0xa96b9:
     snapshot=bytes(self.load(self.r['EDX']+i,1)for i in range(self.r['EBX']))
     for i,b in enumerate(snapshot):self.store(self.r['EAX']+i,1,b)
    elif target==0x982c0:""");scope={'__file__':str(original),'__name__':'carry_source'};exec(compile(text,'carry_source','exec'),scope);s=scope['Source']();m=scope['m'];files=[P/'private/ghidra'/x for x in ['terrain-render-refs/instructions-00072e50.txt','mission-bootstrap-mode/instructions-00080e90.txt','mission-array-refs/instructions-00080d40.txt','mission-array-refs/instructions-00080360.txt','mission-result-refs/building-reset.txt','mission-result-refs/carry-range.txt']];s.code.update(m.Model(files,b'').code);s.code[0x25150]=('RET',[]);keys=sorted(s.code);s.next=dict(zip(keys,keys[1:]));stats=[]
for label in ['1289','1489']:
 d=W/label;d.mkdir();records=random.Random(int(label)).randbytes(600)
 for name in ['players.bin','actors.bin','bindings.bin','map.bin','backing.bin']:shutil.copyfile(origin/label/name,d/name)
 (d/'records.bin').write_bytes(records);backing=(d/'backing.bin').read_bytes();players=(d/'players.bin').read_bytes();prior_actors=(d/'actors.bin').read_bytes();actors=bytearray(prior_actors);bindings=(d/'bindings.bin').read_bytes()
 for i in range(20):
  valid,owner,bp=bindings[i*3:i*3+3]
  if valid:struct.pack_into('<I',actors,i*200+192,0x118d74+owner*1805+0x16+bp*36)
 state={0x118d60+i:v for i,v in enumerate(backing[:0x355e])};state.update({0x118d74+i:v for i,v in enumerate(players)});state.update({0x16360e+i:v for i,v in enumerate(actors)});state.update({0x11c2be+i:v for i,v in enumerate((d/'map.bin').read_bytes())});state.update({0x184d7c+i:v for i,v in enumerate(records)});state[0x118770]=0x7e;state[0x118968]=2;state[0x11873d]=0
 for addr,val in [(0x118740,0x16360e),(0x185ec4,0)]:state.update({addr+i:v for i,v in enumerate(struct.pack('<I',val))})
 report,_,trace=s.run(counter=int(label),state_before=state,entry=0x72e50,registers={'EAX':0x118d60});pa=bytes(s.load(0x118d74+i,1)for i in range(7220));aa=bytearray(s.load(0x16360e+i,1)for i in range(4000))
 for i in range(20):aa[i*200+192:i*200+196]=prior_actors[i*200+192:i*200+196]
 expected=pa+aa+bytes(s.load(0x11c2be+i,1)for i in range(229376))+bytes(s.load(0x184d7c+i,1)for i in range(600))+bytes([s.load(0x118770,1)])+struct.pack('<I',s.load(0x11877e,4))+bytes([int(0x25150 in trace),0,1]);(d/'expected.bin').write_bytes(expected)
 state[0x11873d]=1;other_report,_,other_trace=s.run(counter=int(label),state_before=state,entry=0x72e50,registers={'EAX':0x118d60});assert 0x25150 not in other_trace
 assert bytes(s.load(0x118d74+i,1)for i in range(7220))==pa
 assert bytes(s.load(0x11c2be+i,1)for i in range(229376))==expected[11220:11220+229376]
 assert trace.count(0x80d40)==2 and trace.count(0x8acc0)==2;stats.append(dict(tick=int(label),instructions=report['steps'],selected_players_tested=[0,1],actor_removals=2,unlinks=2,selection_service_requests=trace.count(0x25150)))
production=P/'rust/crates/genewars-runtime/src/result_carry.rs';shutil.copyfile(production,W/'runtime.rs');shutil.copyfile(Path(__file__).with_suffix('.rs'),W/'probe.rs')
def run(a):
 r=subprocess.run(a,cwd=P,env=environment(),capture_output=True,text=True,timeout=120);assert r.returncode==0,(r.stdout,r.stderr)
run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(W/'probe.rs'),'-o',str(W/'probe.exe')]);run([str(W/'probe.exe'),str(W)])
for label in ['1289','1489']:
 a=(W/label/'actual.bin').read_bytes();e=(W/label/'expected.bin').read_bytes();assert a==e,(label,[(i,x,y)for i,(x,y)in enumerate(zip(a,e))if x!=y][:12])
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
receipt=dict(status='passed',private_run=W.relative_to(P).as_posix(),source_capture=origin.relative_to(P).as_posix(),cases=stats,source_pins={f.relative_to(P).as_posix():sha(f)for f in s.files+files+[original,P/'research/variants/actor-environment-terrain-target.py',P/'private/ghidra/mission-result-refs/memmove.txt',P/'private/ghidra/mission-tail-readers/instructions-000982c0.txt',P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin',production,Path(__file__),Path(__file__).with_suffix('.rs')]},capture_pins={str(f.relative_to(W)):sha(f)for label in ['1289','1489']for f in (W/label).glob('*.bin')},scope='Actual72E50/80E90/80D40/80360/42B50/8ACC0 compared fullplayers/actors/map/600recordbytes/flags/frame; source memset/memmove primitives and selection25150 are explicit services; only isolated actor links admitted; no nextworld load or accepted transition')
(W/'result.json').write_text(json.dumps(receipt,indent=2));Path(__file__).with_suffix('.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(dict(status='passed',private_run=receipt['private_run'])))
