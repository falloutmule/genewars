"""Independent native first-choice sweep plus literal-listing whole callbacks."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,struct,subprocess,sys
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
W=P/'private/review/initial-creature-native'
def module(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
environment_model=module('environment_model',Path(__file__).with_name('actor-environment-contract.py'))
source_model=module('source_model',Path(__file__).with_name('initial-creature-source.py'))
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
def signed(v):return (v+32768)%65536-32768
def expected(data,seed,sine,cosine):
 raw=data[0x3b55e:0x3b55e+116];after=bytearray(raw[:31]+bytes(8)+raw[31:]);draws=[];points=[]
 def draw():
  nonlocal seed
  seed=seed*58925%65536;draws.append(seed);return seed//256
 # These are source-visible even though neutral/full-health scorer outcomes
 # are irrelevant to this admitted first-choice branch.
 draw();draw();wait=draw()%10<4
 after[0x3b:0x3d]=struct.pack('<h',512);after[0x66:0x68]=struct.pack('<h',9083);after[0x72:0x76]=after[7:11];after[0x60]|=8
 if wait:timer=19
 else:
  for _ in range(16):
   angle=draw()*8;radius=draw()*2048//256;index=(angle-512)%2048
   delta=[cosine[index]*radius*2//65536,sine[index]*radius*2//65536]
   point=[(signed(32768+delta[0])//2)&32767,(signed(31744+delta[1])//2)&32767]
   cell=(point[1]//256%128)*128+point[0]//256%128
   accepted=environment_model.terrain_class(data[0x355e:0x3b55e],point)!=0 and environment_model.initial_blocker(data,cell)<10
   points.append((point,accepted))
   if accepted:break
   after[0x49]|=64
  else:return bytes([0]),None,draws,points
  after[11:15]=struct.pack('<hh',*point);after[0x49]|=20;after[0x54]=20;after[0x60]|=128;after[0x3f:0x41]=struct.pack('<h',577);timer=32766
 after[0x6c:0x6e]=struct.pack('<h',timer);after[0x1c]=0
 payload=bytes([1])+after+struct.pack('<HB I H B',seed,(31+len(draws))%32,(len(draws)-1)%2**32,len(draws),len(points))
 for point,accepted in points:payload+=struct.pack('<hhB',*point,accepted)
 return payload,after,draws,points
def main():
 W.mkdir(parents=True,exist_ok=True);mission=P/'private/integration/game-runtime/terrain/CONQ001.DAT';data=mission.read_bytes();(W/'mission.bin').write_bytes(data)
 paths=[p for crate in ['assets','core','runtime']for p in (P/f'rust/crates/genewars-{crate}/src').rglob('*.rs')];before=[pin(p)for p in paths]
 rustc=P/'private/tools/rust/cargo/bin/rustc.exe'
 for crate in ['assets','core','runtime']:
  args=[str(rustc),'--edition=2021','--cap-lints=allow','-O','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(P/f'rust/crates/genewars-{crate}/src/lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for d in ['assets','core']:args+=['--extern',f'genewars_{d}={W}/libgenewars_{d}.rlib']
  r=subprocess.run(args,env=environment(),capture_output=True,timeout=120);assert r.returncode==0,r.stderr.decode(errors='replace')
 args=[str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','-o',str(W/'review.exe')]
 for d in ['assets','core','runtime']:args+=['--extern',f'genewars_{d}={W}/libgenewars_{d}.rlib']
 for cmd in [args,[str(W/'review.exe'),str(W)]]:
  r=subprocess.run(cmd,env=environment(),capture_output=True,timeout=180);assert r.returncode==0,r.stderr.decode(errors='replace')
 mapped=(P/'private/ghidra/validated/memory-.object3.bin').read_bytes();sine=struct.unpack_from('<2048i',mapped,0x107ee8-0x100000);cosine=struct.unpack_from('<2048i',mapped,0x1086e8-0x100000)
 actual=(W/'all-seeds.bin').read_bytes();at=0;waits=errors=draw_count=0;lengths=[]
 for seed in range(65536):
  wanted,after,draws,points=expected(data,seed,sine,cosine);got=actual[at:at+len(wanted)];assert got==wanted,('seed',seed,got.hex(),wanted.hex());at+=len(wanted);lengths.append(len(wanted));waits+=not bool(points);errors+=after is None;draw_count+=len(draws)
 assert at==len(actual)
 selected=list(range(64))+[5428,65535];owned=(W/'owned-seeds.bin').read_bytes();at=0;literal=[];model=source_model.Source()
 for seed in selected:
  wanted,after,draws,points=expected(data,seed,sine,cosine);assert owned[at:at+len(wanted)]==wanted,('owned',seed);at+=len(wanted)
  result,raw,trace=model.run(seed=seed);normalized=bytearray(raw);normalized[31:39]=bytes(8)
  assert normalized==after,('listing-afterimage',seed,[(i,a,b)for i,(a,b)in enumerate(zip(normalized,after))if a!=b]);assert result['draws']==len(draws) and result['rng_seed_after']==draws[-1] and result['rng_ring_after']==(31+len(draws))%32 and result['rng_draw_count_after']==(len(draws)-1)%2**32;assert result['map_changed_bytes']==0
  # These three original helpers read two extra high bytes from a packed
  # point, then narrow/sign-extend before any branch or output. Their low16
  # results are independent of those high bytes; verify both poison patterns.
  assert all(pc in [0x736a6,0x7371b,0x73986]for pc,address in model.uninitialized_stack_reads)
  for poison in [0xa5,0xff]:
   poisoned,raw_poisoned,_=model.run(seed=seed,stack_fill=poison);canon=bytearray(raw_poisoned);canon[31:39]=bytes(8)
   assert canon==after and poisoned['rng_seed_after']==result['rng_seed_after'] and poisoned['draws']==result['draws'] and poisoned['map_changed_bytes']==0
  literal.append(dict(seed=seed,instructions=result['steps'],draws=result['draws'],rng_seed_after=result['rng_seed_after'],canonical_after_sha256=hashlib.sha256(normalized).hexdigest()))
 assert at==len(owned);assert before==[pin(p)for p in paths]
 report=dict(status='passed-independent-native-initial-mule-choice',utc=datetime.now(timezone.utc).isoformat(),scope='Exact initial owner4/type1/selector4/state8/zero-timer numeric callback with neutral-neighborhood and ground-context gates; no latermovement/scheduler claim',source_pins=before,mission=pin(mission),instruction_pins=[pin(p)for p in model.files],review_sources=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs')),pin(Path(__file__).with_name('initial-creature-source.py'))],seed_cases=65536,waiting_cases=waits,backward_search_rejections=errors,draws_checked=draw_count,owned_world_cases=len(selected),literal_whole_callback_cases=len(literal),dead_high_stack_lane_poison_cases=2*len(selected),literal_instructions=sum(r['instructions']for r in literal),native_output_bytes=len(actual)+len(owned),outputs=[pin(W/'all-seeds.bin'),pin(W/'owned-seeds.bin')],source_callbacks=literal,negative_cases=['wrongframe0preserveswholeworld','repeatedfirstcallbackpreserveswholeworld','16rejectedpointsrequirebackwardsearch','alteredinitialrecordrejectsbeforequery'],original_execution=False,gui_execution=False,limits=['Live RNG is caller-owned and shared; seed5428 is one explicit fixture, not inferred landingseed','83140 captured as pending queue request; admission and rendering notexecuted by sourceoracle','Complete ground/climate/currentcell/neutralneighborhood gates are checked against owned state','Latercallbackmovement, damage/reproduction/enemyAI and backwardsearch remain unsupported','Private pointer bytes normalized only forafterimage comparison; typedresource/vtable handles stay separate','Three original point helpers read dead high stack lanes; low16 narrowing before observable use checked with two poison patterns'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(report,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:report[k]for k in ['status','seed_cases','owned_world_cases','literal_whole_callback_cases','draws_checked','native_output_bytes']}))
if __name__=='__main__':main()
