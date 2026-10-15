"""Literal114B0 empty-building fallback versus native complete afterimages."""
from pathlib import Path
from datetime import datetime,timezone
import importlib.util,struct,hashlib,json,sys,subprocess,random
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
W=P/'private/review/family8-target'
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
def run(args):
 r=subprocess.run(args,cwd=P,env=environment(),capture_output=True,timeout=180);assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'));print(r.stdout.decode(errors='replace'))
def main():
 W.mkdir(parents=True,exist_ok=True)
 s=importlib.util.spec_from_file_location('family8_source',Path(__file__).with_name('family8-state4-source.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m);model=m.Model()
 files=[P/'private/ghidra/countdown-expiry'/f'instructions-000{x}.txt'for x in ['114b0','13e60']]+[P/'private/ghidra/actor-activation/instructions-00073e00.txt']
 model.files+=files;model.code.update(m.m.m.Model(files,b'').code);order=sorted(model.code);model.next=dict(zip(order,order[1:]));model.next[0x97830]=0x97831
 mission=P/'private/integration/game-runtime/terrain/CONQ001.DAT';map_data=mission.read_bytes()[0x355e:0x3b55e];(W/'map.bin').write_bytes(map_data)
 before_input=P/'private/review/family8-onward/child-after6143.bin';template=before_input.read_bytes();rand=random.Random(11400);buildings=bytearray(rand.randbytes(64*67))
 for i in range(64):buildings[i*67+6]=0
 (W/'buildings.bin').write_bytes(buildings);inputs=bytearray();expected=bytearray();steps=0;count=0
 for owner in range(4):
  for point in [[0,0],[32767,-32768],[20736,13568],[-1,-1]]:
   for seed in [0,1,5428,65535]:
    before=bytearray(template);before[0x18]=owner;before[0x57]=0;before[0x5b]=0;struct.pack_into('<H',before,0x59,0x7fff)
    rng=(seed,count%32,0xffffffff if count%2 else 0);inputs+=before+struct.pack('<hhHBI',*point,*rng)
    state={0x1645ae+owner*64*67+i:b for i,b in enumerate(buildings)};state.update({0x118d74+owner*1805+9+i:b for i,b in enumerate(struct.pack('<hh',*point))})
    after,nrng,n,_,trace=model.step(before,map_data,6143,rng,entry=0x114b0,extra_state=state)
    expected+=after+struct.pack('<HBI',*nrng)+after[0xb:0xf]+after[0x60:0x64];steps+=n;count+=1
    if count==1:
     for fill in [85,255]:
      other,rr,_,_,_=model.step(before,map_data,6143,rng,fill,entry=0x114b0,extra_state=state);assert(other,rr)==(after,nrng)
 (W/'input.bin').write_bytes(inputs);(W/'expected.bin').write_bytes(expected)
 sources=[p for crate in ['assets','core','runtime']for p in(P/f'rust/crates/genewars-{crate}/src').rglob('*.rs')];pins=[pin(p)for p in sources]
 rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe')
 for crate in ['assets','core','runtime']:
  args=[rustc,'--edition=2021','--cap-lints=allow','-O','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(P/f'rust/crates/genewars-{crate}/src/lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
  run(args)
 args=[rustc,'--edition=2021','-O',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','-o',str(W/'review.exe')]
 for dep in ['assets','core','runtime']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
 run(args);run([str(W/'review.exe'),str(W)]);got=(W/'actual.bin').read_bytes();assert got==expected,dict(byte=next((i for i,(a,b)in enumerate(zip(got,expected))if a!=b),None))
 assert pins==[pin(p)for p in sources],'source changed during review'
 paths=model.files+[Path(__file__),Path(__file__).with_suffix('.rs'),Path(__file__).with_name('family8-state4-source.py'),Path(__file__).with_name('initial-creature-source.py'),mission,before_input,m.ANB,P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin']
 result=dict(status='passed-bounded-family8-initial-target',utc=datetime.now(timezone.utc).isoformat(),cases=count,compared_bytes=len(got),literal_instructions=steps,source_pins=pins,input_and_listing_pins=[pin(p)for p in paths],outputs=[pin(W/f)for f in ['input.bin','expected.bin','actual.bin','buildings.bin']],original_execution=False,gui_execution=False,
  limits=['State1 initial helper inactive-building fallback source matrix; native checks also cover active incomplete records being skipped and malformed pool length rejection. Eligible active targets are covered by separate active-building proofs','Player target is explicit raw player+9 input, not inferred from camera','Source one73E00 draw/threaded seed/ring/count; pointer/link/opaque nullindex preserved','No world commit or subsequent state1 callback claimed'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:result[k]for k in ['status','cases','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()
