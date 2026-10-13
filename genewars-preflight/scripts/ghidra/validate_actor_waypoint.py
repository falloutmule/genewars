"""Actual landing-created actor passes, source order, arrival and late rollback.

This is production composition evidence; scalar source oracles are separate.
The explicit zero/count-zero/frozen-family fixture does not claim full startup.
"""
from pathlib import Path
import sys,subprocess,json,re,hashlib,uuid
p=Path(__file__).resolve().parents[2];sys.path.insert(0,str(p/'scripts'));from cargo import environment
w=p/'private/ghidra/actor-motion'/uuid.uuid4().hex;w.mkdir(parents=True);sha=lambda b:hashlib.sha256(b).hexdigest()
r=subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked','--message-format=json'],cwd=p,capture_output=True,check=True);art={}
for line in r.stdout.splitlines():
 x=json.loads(line)
 if x.get('reason')=='compiler-artifact' and 'lib'in x['target']['kind']:art[x['target']['name']]=next(f for f in x['filenames'] if f.endswith('.rlib'))
h=p/'scripts/ghidra/actor_waypoint_probe.rs';exe=w/'actor-waypoint-gate.exe';cmd=[str(p/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(h),'-L',f'dependency={p}/private/build/rust-target/debug/deps','-o',str(exe)]
for n in ['genewars_assets','genewars_runtime','genewars_core']:cmd+=['--extern',f'{n}={art[n]}']
subprocess.run(cmd,cwd=p,env=environment(),check=True);
r=subprocess.run([str(exe),str(p),str(w)],cwd=p,env=environment(),capture_output=True,timeout=60);assert r.returncode==0,r.stderr.decode();(w/'waypoint-gate.log').write_bytes(r.stdout+r.stderr);assert 'blocked candidate=[24634, 9875]' in r.stdout.decode();assert 'frame557 id0 pos=[24634, 9875] state1 speed0' in r.stdout.decode();assert 'waterSome(' in r.stdout.decode();assert 'actual blocked-point search requested=' in r.stdout.decode();assert 'late second actor after water factory rollback passed' in r.stdout.decode();assert sha((p/'private/ghidra/player-creation/instructions-0006fb60.txt').read_bytes())=='b373e98528fde4ef720bf0b529edbca889efb97bec713ac4d7fe184c868fb736'
src=(p/'research/variants/initial-creature-source.py').read_text().replace('P=Path(__file__).resolve().parents[2]','P=Path('+repr(str(p))+')').replace('   if self.pc==0xdeadbeef:break','   if self.pc in [0xdeadbeef,0x6df30]:break')
# Preserve the model import location while leaving its source unchanged.
scope={'__file__':str(p/'research/variants/initial-creature-source.py'),'__name__':'actor_prefix_private'};exec(compile(src,'actor-prefix-private','exec'),scope);model=scope['Source']();d=w/'blocked-prefix';state={}
for i,b in enumerate((d/'map.bin').read_bytes()):state[0x11c2be+i]=b
for actor in range(20):
 a=bytearray((d/f'actor{actor}.bin').read_bytes());a[35:39]=(0x112194).to_bytes(4,'little');a[192:196]=(0x118d74+0x16+(0 if actor==0 else 2)*36).to_bytes(4,'little');state.update({0x16360e+actor*200+i:b for i,b in enumerate(a)})
for player in range(4):state.update({0x118d74+player*1805+i:b for i,b in enumerate((d/f'player{player}.bin').read_bytes())})
report,_,trace=model.run(counter=311,state_before=state,entry=0x6fb60,registers={'EAX':0x16360e},instruction_budget=1000000)
a=bytes(model.load(0x16360e+i,1)for i in range(200));(d/'source-prefix.bin').write_bytes(a);print(report);print('flags',a[73:75].hex(),'waypoint',a[69:73].hex(),'desiredyaw',a[75:77].hex(),'source_steps',len(trace))

actual=(d/'production-prefix.bin').read_bytes();source=bytearray((d/'source-prefix.bin').read_bytes());source[35:39]=actual[35:39];source[192:196]=actual[192:196];assert source==actual
assert report['draws']==0 and report['map_changed_bytes']==0
reference=bytes(source)
report_ff,_,_=model.run(counter=311,state_before=state,entry=0x6fb60,registers={'EAX':0x16360e},instruction_budget=1000000,stack_fill=255)
source=bytearray(model.load(0x16360e+i,1)for i in range(200));source[35:39]=actual[35:39];source[192:196]=actual[192:196];assert source==reference
result={'status':'PASS','compared_bytes':200,'literal_instruction_steps':report['steps'],'stack_fill_cases':[0,255],'output_sha256':sha(actual),'source_sha256':sha((p/'rust/crates/genewars-runtime/src/actor_world/actor_updates.rs').read_bytes()),'listing_sha256':sha((p/'private/ghidra/player-creation/instructions-0006fb60.txt').read_bytes()),'scope':'Actual seed5428 landing-created actor; accepted blocked destination[24634,9875], first successful boundary walk/6F980 steering; native200B pointers35/C0 normalized. Actual live-effects route reaches requested destination and state1 at557, including code30 water factory; exhausted retry branches remain unsupported.'}
(p/'research/ghidra/actor-waypoint-evidence.json').write_text(json.dumps(result,indent=2)+'\n');print(result)
