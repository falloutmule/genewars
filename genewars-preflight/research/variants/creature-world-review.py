"""Native storage comparison only. Initial AI closure remains separate."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,sys
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
W=P/'private/review/creature-world'
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
def main():
 W.mkdir(parents=True,exist_ok=True);source=P/'rust/crates/genewars-runtime/src/creature_world.rs';before=pin(source);mission=P/'private/integration/game-runtime/terrain/CONQ001.DAT';data=mission.read_bytes();(W/'mission.bin').write_bytes(data)
 rustc=P/'private/tools/rust/cargo/bin/rustc.exe'
 for crate in ['assets','core','runtime']:
  args=[str(rustc),'--edition=2021','--cap-lints=allow','-O','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(P/f'rust/crates/genewars-{crate}/src/lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for d in ['assets','core']:args+=['--extern',f'genewars_{d}={W}/libgenewars_{d}.rlib']
  r=subprocess.run(args,env=environment(),capture_output=True,timeout=120);assert r.returncode==0,r.stderr.decode(errors='replace')
 for test in [False,True]:
  args=[str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','-o',str(W/('tests.exe'if test else'review.exe'))]
  if test:args+=['--test']
  for d in ['assets','core','runtime']:args+=['--extern',f'genewars_{d}={W}/libgenewars_{d}.rlib']
  for command in [args,[str(W/('tests.exe'if test else'review.exe')),str(W)]]:
   r=subprocess.run(command,env=environment(),capture_output=True,timeout=120);assert r.returncode==0,r.stderr.decode(errors='replace')+r.stdout.decode(errors='replace')
 expected=bytearray()
 for i in range(300):
  raw=data[0x3b55e+i*116:0x3b55e+(i+1)*116];gap=bytes((i*37+j*19)&255 for j in range(31,39));native=bytearray(raw[:31]+gap+raw[31:])
  if raw[6]:native[31:35]=bytes(4)
  expected+=native+bytes([int(bool(raw[6]))])
 actual=(W/'pool.bin').read_bytes();assert actual==expected;assert before==pin(source)
 result=dict(status='passed-native-creature-storage-review',utc=datetime.now(timezone.utc).isoformat(),scope='Exact first-mission native124B family0 copy and typedMule binding only; no AI/tick closure',records=300,compared_bytes=len(actual),active_records=1,source=before,mission=pin(mission),review_sources=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs'))],instruction_pins=[pin(P/'private/ghidra'/f)for f in ['mission-tail-readers/instructions-0005a850.txt','mission-tail-readers/instructions-0006d490.txt','mission-tail-init/instructions-0005a1c0.txt']],output=pin(W/'pool.bin'),tests=2,original_execution=False,gui_execution=False,limits=['Explicit already-constructed priors; vtable bytes preserved and never dereferenced','Only active type1/state<26/selector4 admitted; otheractivebindings reject','Rawprior gap preserved except activeanimationpointerplaceholder; sourceartifact remains unchanged','No map, RNG or scheduling implemented by this storage API'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:result[k]for k in ['status','records','compared_bytes','tests']}))
if __name__=='__main__':main()
