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
for funds in [0,1000]:
 d=work/f'funds-{funds}';d.mkdir();(d/'border.bin').write_bytes(border)
 r=subprocess.run([str(exe),str(P),'5428',str(funds),str(d)],cwd=P,env=environment(),capture_output=True,timeout=120)
 (d/'run.log').write_bytes(r.stdout+r.stderr)
 assert r.returncode==0,r.stderr.decode(errors='replace')
 text=r.stdout.decode()
 assert 'sitearrival=530'in text and 'constructionframe=532 stage=1 action=13 anim=13'in text and 'constructionframe=533 stage=3'in text
 actor=(d/'action13.bin').read_bytes();bp=(d/'blueprint.bin').read_bytes()
 assert len(actor)==200 and actor[0x55]==9 and actor[0x59]==13 and actor[0x19]==13 and actor[0x70]&128 and bp[0x15]==2 and bp[0x16]==0
 snapshots={'action13':sha(actor),'blueprint':sha(bp)}
 if funds:
  assert 'paymentcost=688 fundsafter=312'in text
  tail=(d/'action14-after-tail.bin').read_bytes()
  assert tail[0x55]==9 and tail[0x59]==14 and tail[0x19]==14 and tail[0x70]&128
  snapshots['action14_after_tail']=sha(tail)
 else:assert 'InsufficientCancellationRequired'in text and 'cancelled task=0 state=9 mode=255 funds=0'in text
 scenarios.append({'initial_primary_funds':funds,'initial_secondary_funds':0,'site_arrival_frame':530,'terraform_allocation_frame':532,'terraform_completion_frame':533,'payment_frame':int(re.search(r'paymentframe=(\d+)',text)[1]),'funds_after':312 if funds else 0,'resource_key':8,'blueprint_archetype':2,'category':0,'snapshot_sha256':snapshots,'private_log_sha256':sha((d/'run.log').read_bytes())})
report={'status':'passed','mission_sha256':'72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2','seed':5428,'native_priors':'explicit zero actor/creature/effect; jobs all signedcounter-1','funding':'explicit1000 success fixture and fresh0 refusal/cancellation','site':[19839,12927],'scenarios':scenarios,'private_run':work.relative_to(P).as_posix(),'implementation_pins':pins,'harness_sha256':sha(probe.read_bytes()),'validator_sha256':sha(Path(__file__).read_bytes()),'scope':'Production composition: landing189,ordinaryactor callbacks450,actualgroundorder/arrival,constructionassignment/setup,terraformcallbacks0..2/full empty-object-list terrainpass,whole owned construction callbacks0..3 including payment/cancellation and animation14tail. Otherfamilies frozen; source component oracles are separate. No fullstartup/UI/buildingcompletion claim; no speed/readiness/position fabrication.'}
(P/'research/integration/architect-cycle-validation.json').write_text(json.dumps(report,indent=2)+'\n')
print({k:v for k,v in report.items()if k!='implementation_pins'})
