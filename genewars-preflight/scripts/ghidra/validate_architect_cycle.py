"""Production composition of source-compared architect construction slices."""
from pathlib import Path
import hashlib, importlib.util, json, re, subprocess, sys, uuid
P=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(P/'scripts'))
from cargo import environment
sha=lambda b:hashlib.sha256(b).hexdigest()
work=P/'private/ghidra/architect-cycle'/uuid.uuid4().hex
work.mkdir(parents=True)
spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
source=m.Source();source.mem={}
border=bytes(source.load(0x112d24+i,1)for i in range(8192))
assert sha(border)=='9f1dcbc35c350d6027f98be0f5c8b43b42ca52b7604459c0c42be3aa88913d47'
pins={f.relative_to(P).as_posix():sha(f.read_bytes())for f in (P/'rust/crates/genewars-runtime/src').rglob('*.rs')}
subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,capture_output=True,check=True)
assert all(sha((P/f).read_bytes())==h for f,h in pins.items()),'runtime changed during build'
target=P/'private/build/rust-target/debug';exe=work/'cycle.exe';probe=P/'scripts/ghidra/architect_cycle_probe.rs'
cmd=[str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(probe),'-L',f'dependency={target}/deps','-o',str(exe)]
for crate in ['runtime','assets','core']:cmd+=['--extern',f'genewars_{crate}={target}/libgenewars_{crate}.rlib']
r=subprocess.run(cmd,cwd=P,env=environment(),capture_output=True)
assert r.returncode==0,r.stderr.decode(errors='replace')
scenarios=[]
for funds_arg in [0,1000,-1]:
 funds=2000 if funds_arg<0 else funds_arg
 d=work/f'funds-{funds}';d.mkdir();(d/'border.bin').write_bytes(border)
 r=subprocess.run([str(exe),str(P),'5428',str(funds_arg),str(d)],cwd=P,env=environment(),capture_output=True,timeout=120)
 (d/'run.log').write_bytes(r.stdout+r.stderr)
 assert r.returncode==0,r.stderr.decode(errors='replace')
 text=r.stdout.decode()
 assert 'mixedscheduler=2 latefailure=rollback manualmatch=passed'in text
 assert 'pendingcallback=531'in text
 assert 'sitearrival=530'in text and 'constructionframe=532 stage=1 action=13 anim=13'in text and 'constructionframe=533 stage=3'in text
 actor=(d/'action13.bin').read_bytes();bp=(d/'blueprint.bin').read_bytes()
 assert len(actor)==200 and actor[0x55]==9 and actor[0x59]==13 and actor[0x19]==13 and actor[0x70]&128 and bp[0x15]==2 and bp[0x16]==0
 snapshots={'action13':sha(actor),'blueprint':sha(bp)}
 if funds:
  assert f'paymentcost=688 fundsafter={funds-688}'in text
  if funds_arg<0:assert 'sourcefunds=2000'in text
  tail=(d/'action14-after-tail.bin').read_bytes()
  assert tail[0x55]==9 and tail[0x59]==14 and tail[0x19]==14 and tail[0x70]&128
  snapshots['action14_after_tail']=sha(tail)
  stage4=(d/'stage4-after-tail.bin').read_bytes();assert stage4[0x59]==15 and int.from_bytes(stage4[0x88:0x8a],'little')==5
  snapshots['stage4_after_tail']=sha(stage4)
  assert 'stage4frame='in text and 'workcomplete_next='in text
  complete_actor=(d/'work-completed-actor.bin').read_bytes();complete_building=(d/'completed-building.bin').read_bytes();assert int.from_bytes(complete_building[0x2b:0x2d],'little')==120 and int.from_bytes(complete_building[0x31:0x33],'little')==1
  snapshots['work_completed_actor']=sha(complete_actor);snapshots['completed_building']=sha(complete_building)
 else:assert 'InsufficientCancellationRequired'in text and 'cancelled task=0 state=9 mode=255 funds=0'in text
 scenarios.append({'initial_primary_funds':funds,'initial_secondary_funds':0,'site_arrival_frame':530,'terraform_allocation_frame':532,'terraform_completion_frame':533,'payment_frame':int(re.search(r'paymentframe=(\d+)',text)[1]),'funds_after':funds-688 if funds else 0,'funding_source':'literal mission initializer'if funds_arg<0 else 'explicit fixture','resource_key':8,'blueprint_archetype':2,'category':0,'snapshot_sha256':snapshots,'private_log_sha256':sha((d/'run.log').read_bytes())})
report={'status':'passed','mission_sha256':'72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2','seed':5428,'native_priors':'explicit zero actor/creature/effect; jobs all signedcounter-1','funding':'Source mission initializer2000, explicit1000 success fixture and fresh0 refusal/cancellation','site':[19839,12927],'scenarios':scenarios,'private_run':work.relative_to(P).as_posix(),'implementation_pins':pins,'harness_sha256':sha(probe.read_bytes()),'validator_sha256':sha(Path(__file__).read_bytes()),'scope':'Production composition: landing189,ordinaryactor callbacks450,actualgroundorder/arrival,constructionassignment/wholependingcallbacksetupwithlateresourcerollback,terraformcallbacks0..2/full empty-object-list terrainpass,whole owned construction callbacks0..5 including payment/cancellation, animation14tail, building/effect allocation and progress1, work63/120, incompletebuilding callbacks/effectretirement untilcompletion beforefirstcompletedbuildingcallback. Otherfamilies frozen; source component oracles are separate. No fullstartup/UI/buildingcompletion claim; no speed/readiness/position fabrication.'}
(P/'research/integration/architect-cycle-validation.json').write_text(json.dumps(report,indent=2)+'\n')
print({k:v for k,v in report.items()if k!='implementation_pins'})
