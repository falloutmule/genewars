"""Native afterimage comparison to separately interpreted private CD listings."""
from pathlib import Path
from datetime import datetime,timezone
import sys,subprocess,hashlib,json
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
W=P/'private/review/family8-state4'
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
def run(args):
 r=subprocess.run(args,cwd=P,env=environment(),capture_output=True,timeout=180);assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'));print(r.stdout.decode(errors='replace'))
def main():
 run([sys.executable,'-B',str(Path(__file__).with_name('family8-state4-source.py'))])
 obj=(P/'private/ghidra/validated/memory-.object1.bin').read_bytes();obj3=(P/'private/ghidra/validated/memory-.object3.bin').read_bytes()
 (W/'angles.bin').write_bytes(obj[0xae134-0x10000:0xae134-0x10000+514]);(W/'bounce.bin').write_bytes(obj3[0x4530:0x4540])
 (W/'ethereal.anb').write_bytes((P/'private/integration/rnc-decoded/7660297d85773f47cacc07dcfdf96e8daa118dc52c8a38afc054c1ef9dfa20bc.bin').read_bytes())
 sources=[p for crate in ['assets','core','runtime']for p in(P/f'rust/crates/genewars-{crate}/src').rglob('*.rs')];pins=[pin(p)for p in sources]
 rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe')
 for crate in ['assets','core','runtime']:
  args=[rustc,'--edition=2021','--cap-lints=allow','-O','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(P/f'rust/crates/genewars-{crate}/src/lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
  run(args)
 args=[rustc,'--edition=2021','-O',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','-o',str(W/'review.exe')]
 for dep in ['assets','core','runtime']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
 run(args);run([str(W/'review.exe'),str(W)])
 expected=(W/'expected.bin').read_bytes();actual=(W/'actual.bin').read_bytes();assert actual==expected,dict(byte=next((i for i,(a,b)in enumerate(zip(actual,expected))if a!=b),None),actual_length=len(actual),expected_length=len(expected))
 assert pins==[pin(p)for p in sources],'production changed during review'
 source=json.loads((W/'source.json').read_text())
 result=dict(status='passed-bounded-family8-selector0-state4',utc=datetime.now(timezone.utc).isoformat(),callbacks=source['cases'],compared_bytes=len(actual),literal_instructions=source['literal_instructions'],source_pins=pins,
  listing_and_input_pins=source['pins'],review_pins=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs'))],private_outputs=[pin(W/f)for f in ['input.bin','expected.bin','actual.bin','source.json']],original_execution=False,gui_execution=False,
  limits=['Factory-only initial prior; first callback6001 and6002 tested separately, each with seeds0/5428/65535','Full102-byte parent and live RNG; pointer bytes normalized but preserved by native planner','Child allocation, map link commits, optional UI and audio device are typed ports; audio stop query result not invented','State5 onward and the later shared ground/bounce branch have separate receipts','Water/damage/dirty virtual2C and non-null target references remain bounded atomic rejection conditions'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n');tmp.replace(out)
 print(json.dumps({k:result[k]for k in ['status','callbacks','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()
