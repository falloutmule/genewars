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
h=p/'scripts/ghidra/actor_owned_probe.rs';exe=w/'actor-owned-gate.exe';cmd=[str(p/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(h),'-L',f'dependency={p}/private/build/rust-target/debug/deps','-o',str(exe)]
for n in ['genewars_assets','genewars_runtime','genewars_core']:cmd+=['--extern',f'{n}={art[n]}']
subprocess.run(cmd,cwd=p,env=environment(),check=True);scenarios=[]
for seed in [5428,65535]:
 r=subprocess.run([str(exe),str(p),str(seed)],cwd=p,env=environment(),capture_output=True,timeout=60);log=w/f'owned-seed{seed}-gate.log';log.write_bytes(r.stdout+r.stderr)
 assert r.returncode==0,r.stderr.decode();s=r.stdout.decode();assert 'late resource failure rollback passed'in s and 'rejected'not in s
 rows=[(int(f),int(i),int(st),sm)for f,i,st,sm in re.findall(r'frame(\d+) id(\d+) pos=.*? state(\d+) speed\d+ flags[0-9a-f]+ smoke([^\r\n]+)',s)]
 assert len(rows)==522 and [(f,i)for f,i,st,sm in rows]==[(f,i)for f in range(190,451)for i in [0,1]]
 previous={};arrivals=[]
 for f,i,st,sm in rows:
  if previous.get(i)==8 and st==1:arrivals.append({'frame':f,'actor_id':i})
  previous[i]=st
 assert len(arrivals)==3 and [a['actor_id']for a in arrivals].count(0)==2
 assert 'reorder=Applied { requested: [20736, 13568], resolved: [20736, 13568] }'in s
 assert 'frame450 id0 pos=[20736, 13568] state1 speed0'in s
 scenarios.append({'seed':seed,'actor_callbacks':len(rows),'footstep_factories':sum(sm!='None'for f,i,st,sm in rows),'arrivals':arrivals,'ordered_frames':[190,450],'late_second_actor_rollback':True,'private_log_sha256':sha(log.read_bytes())})
files=['rust/crates/genewars-runtime/src/actor_motion.rs','rust/crates/genewars-runtime/src/actor_world/actor_updates.rs','rust/crates/genewars-runtime/src/creature_world.rs']
report={'status':'PASS','kind':'Production composition evidence; independent source component oracles are separate','mission_sha256':'72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2','inputs':{'native_priors':'explicit zero actor/creature/effect records','players':'four fresh campaign records','ring':0,'scratch36':'explicit zero','blueprint_draws':64,'mask':5,'reveal_radius':4096,'family4_prepasses':189,'runtime_mode':0,'override_180d80':0,'runtime_count_11c2bc':0,'other_families':'frozen; no full startup parity claim'},'scenarios':scenarios,'reorder':{'frame':310,'actor_id':0,'requested':[20736,13568]},'harness_sha256':sha(h.read_bytes()),'production_source_sha256':{f:sha((p/f).read_bytes())for f in files}}
(p/'research/ghidra/actor-owned-evidence.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'status':'PASS','scenarios':scenarios}))
