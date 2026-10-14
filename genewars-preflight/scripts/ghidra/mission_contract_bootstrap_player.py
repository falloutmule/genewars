"""Explicit-prior player bootstrap composition and literal7F950 no-op proof."""
from pathlib import Path
import importlib.util,hashlib,json,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
w=P/'private/review/bootstrap-player'/uuid.uuid4().hex;w.mkdir(parents=True)
subprocess.run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked'],cwd=P,capture_output=True,check=True)
target=P/'private/build/rust-target/debug';exe=w/'review.exe';cmd=[str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','scripts/ghidra/mission_contract_bootstrap_player_fixture.rs','-L',f'dependency={target}/deps','-o',str(exe)]
for crate in ['assets','core','runtime']:cmd+=['--extern',f'genewars_{crate}={target}/libgenewars_{crate}.rlib']
subprocess.run(cmd,cwd=P,env=environment(),capture_output=True,check=True)
mission=P/'private/ghidra/campaign-resources/152.dat';text=P/'private/reference-run/run-006/drive-c/GAME/DATA/TEXT.DAT';subprocess.run([str(exe),str(mission),str(text),str(w)],check=True)
s=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
f=P/'private/ghidra/activation-rng-refs/instructions-0007f950.txt';q=w/'7f950-nounused-rep.txt';q.write_text(f.read_text().replace('MOVSD.REP ES:EDI,ESI','NOP'))
x=m.Source();x.code.update(m.m.Model([q],b'').code);order=sorted(x.code);x.next=dict(zip(order,order[1:]));rows=[]
for case in range(4):
 data=(w/f'case{case}.bin').read_bytes();records=data[:1152];player=data[1152:2957];actors=data[2957:];assert len(data)==3957
 metadata=bytes([20])+records;state={0x220000+i:b for i,b in enumerate(metadata)};state[0x11872c]=0 if case<2 else 31;state.update({0x118782+i:b for i,b in enumerate(struct.pack('<I',16919))})
 report,_,trace=x.run(seed=10548,counter=0,state_before=state,entry=0x7f950,registers={'EAX':0x220000});assert 0x7f9d5 not in trace and not x.calls
 assert bytes(x.load(0x220000+i,1)for i in range(1153))==metadata;assert report['draws']==0 and report['rng_seed_after']==10548
 fill=0 if case%2==0 else 0xa5
 assert all(records[i*36+o]==fill for i in range(20)for o in [0x18,0x1b]);assert [actors[i*200+6]for i in range(5)]==[3,3,0,0,0]
 rows.append({'case':case,'scratch_fill':fill,'ring':0 if case<2 else 31,'rng_after':{'seed':10548,'count':16919},'actor_blueprint_indices':[actors[0x74],actors[200+0x74]],'source_backfill_instructions':report['steps'],'snapshot_sha256':hashlib.sha256(data).hexdigest()})
r={'status':'passed-explicit-prior-player-composition','whole_startup_proven':False,'cases':rows,'source7F950_sha256':hashlib.sha256(f.read_bytes()).hexdigest(),'production_sha256':hashlib.sha256((P/'rust/crates/genewars-runtime/src/mission_bootstrap.rs').read_bytes()).hexdigest(),'private_run':w.relative_to(P).as_posix(),'proven':'Existing source-verified blueprint andfresh-actor APIs compose with exact serialized seed/count, mask5 andspecialactivation0; generatedcategorycoverage makes actual7F950 no-op.','ports':'Unused7F950 REP instruction replaced withNOP for parser, asserted never reached; no source external calls.','remaining':['Retained campaign label andnative prior provenance remain caller inputs.','Blueprint scratch18/1B are uninitialized source bytes; both zero andA5 fixtures remain explicit.','Landing factory,72AD0 map/entity-shade refresh andMAINPANL/frame integration pending.']}
(P/'research/campaign/bootstrap-player-composition.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))
