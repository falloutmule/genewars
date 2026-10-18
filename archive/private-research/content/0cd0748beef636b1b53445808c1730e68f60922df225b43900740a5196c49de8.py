"""Bounded literal startup-table proof; no original executable execution."""
from pathlib import Path
import importlib.util,hashlib,json,struct,subprocess,uuid
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class Source(m.Source):
 def store(self,a,n,v):
  if 0x884d0<=getattr(self,'pc',0)<0x88880:self.writes.update(range(a,a+n))
  super().store(a,n,v)
listing=P/'private/ghidra/mission-target/instructions-000884d0.txt';obj=P/'private/ghidra/validated/memory-.object3.bin';mission=P/'private/ghidra/campaign-resources/152.dat'
assert hashlib.sha256(listing.read_bytes()).hexdigest()=='8aa978e0005bc3cb47e85f70f7a05c17f62ac92aa903394056f3e571cc0522ab'
assert hashlib.sha256(obj.read_bytes()).hexdigest()=='aaab09272771ee7f3fbe71e9a3dc75add592fcca42fdd8d982e069629ac1ce87'
assert hashlib.sha256(mission.read_bytes()).hexdigest()=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
x=Source();x.code.update(m.m.Model([listing],b'').code);x.code[0x96ee0]=('RET',[])
rows=[];mapped=obj.read_bytes()[0x12d24:0x14d24]
patterns=obj.read_bytes()[0x5060:0x52e0];choices=[None]*4096
for tile in range(160):
 a,b,c,d=[v&7 for v in patterns[tile*4:tile*4+4]]
 for corners,orientation in [((a,b,c,d),0),((b,a,d,c),16),((c,d,a,b),48),((d,c,b,a),32),((b,c,d,a),96),((c,b,a,d),112),((d,a,b,c),80),((a,d,c,b),64)]:
  key=sum(v<<shift for v,shift in zip(corners,[9,6,3,0]))
  if choices[key] is None:choices[key]=(tile,orientation)
expected=bytes(v for choice in choices for v in (choice if choice is not None else (151,0)))
for fallback in [False,True]:
 for retain in [0,1]:
  x.code[0x96cb0]=('MOV',['EAX','0x0' if fallback else '0x220000']);x.code[0x96cb1]=('RET',[]);order=sorted(x.code);x.next=dict(zip(order,order[1:]));x.writes=set()
  state={base+i:0xa5 for base in [0x220000,0x230000] for i in range(0x9000)};state.update({0x115e84+i:b for i,b in enumerate(struct.pack('<I',0x230000))});state[0x11872c]=17;state.update({0x118782+i:b for i,b in enumerate(struct.pack('<I',16855))})
  report,_,_=x.run(counter=0,seed=5428,state_before=state,entry=0x884d0,registers={'EAX':retain})
  actual=bytes(x.load(0x112d24+i,1)for i in range(8192));assert actual==expected,(hashlib.sha256(actual).hexdigest(),hashlib.sha256(expected).hexdigest(),[(i,a,b) for i,(a,b) in enumerate(zip(actual,expected)) if a!=b][:20])
  unexpected=[a for a in x.writes if not(0x112d24<=a<0x114d24 or 0x220000<=a<0x229000 or 0x230000<=a<0x239000 or 0x2f0000<=a<=0x300100)];assert not unexpected,unexpected
  assert (0x96ee0 in x.calls)==(not fallback and retain==0)
  assert report['draws']==0 and report['rng_seed_after']==5428 and report['rng_ring_after']==17
  rows.append({'allocation_fallback':fallback,'retain':retain,'instructions':report['steps'],'draws':0,'table_matches':True})
work=P/'private/review/bootstrap-border'/uuid.uuid4().hex;work.mkdir(parents=True)
production=P/'rust/crates/genewars-runtime/src/terrain_border_table.rs'
h=work/'review.rs';h.write_text(f'#[path="{production.as_posix()}"]mod terrain_border_table;'+r' use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();assert!(terrain_border_table::build(&b[..639]).is_err());assert!(terrain_border_table::build(&[0u8;641]).is_err());std::io::stdout().write_all(&terrain_border_table::build(&b).unwrap()).unwrap();}')
spec=importlib.util.spec_from_file_location('cargo_local',P/'scripts/cargo.py');cargo=importlib.util.module_from_spec(spec);spec.loader.exec_module(cargo)
exe=work/'review.exe';subprocess.run([str(P/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe'),'--edition=2021',str(h),'-o',str(exe)],env=cargo.environment(),check=True)
assert subprocess.run([str(exe)],input=patterns,capture_output=True,check=True).stdout==expected
data=mission.read_bytes();r={'status':'passed-bounded-bootstrap-details','rust_generator_sha256':hashlib.sha256(production.read_bytes()).hexdigest(),'patterns_address':'00105060','patterns_length':640,'patterns_sha256':hashlib.sha256(patterns).hexdigest(),'rust_compared_bytes':8192,'length_rejection_cases':2,'original_execution':False,'whole_startup_proven':False,'source_sha256':hashlib.sha256(listing.read_bytes()).hexdigest(),'object3_sha256':hashlib.sha256(obj.read_bytes()).hexdigest(),'mission_sha256':hashlib.sha256(data).hexdigest(),'border_table_sha256':hashlib.sha256(expected).hexdigest(),'mapped_prior_table_sha256':hashlib.sha256(mapped).hexdigest(),'mapped_prior_nonzero_bytes':sum(v!=0 for v in mapped),'mapped_prior_differs':mapped!=expected,'cases':rows,'884D0_effect_scope':'Only scratch allocation/stack and8192-byte terrain-border table112D24; no runtime context or RNG mutation. Malloc/free are explicit ports.','serialized_fields':{'live_seed_11878a':struct.unpack_from('<H',data,0x52480)[0],'rng_draw_count_118782':struct.unpack_from('<I',data,0x52478)[0],'frame_11877e':struct.unpack_from('<I',data,0x52474)[0],'frame_backup_118786':struct.unpack_from('<I',data,0x5247c)[0],'active_players_11878c':data[0x52482]},'remaining':'Transitive player preparation, native inherited state, MAINPANL entry and first callback timing are not proved by this narrow check.'}
(P/'research/campaign/bootstrap-details.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))
