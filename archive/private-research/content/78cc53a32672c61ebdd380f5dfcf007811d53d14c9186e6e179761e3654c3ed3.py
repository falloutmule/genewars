from pathlib import Path
import sys,subprocess,hashlib,json,shutil
P=Path.cwd();sys.path.insert(0,str(P/'scripts'))
from cargo import environment
W=P/'private/review/family8-departure'
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
def run(args):
 r=subprocess.run(args,cwd=P,env=environment(),capture_output=True,timeout=240);assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'));print(r.stdout.decode(errors='replace'))
def main():
 for name in ['angles.bin','bounce.bin','ethereal.anb']:(W/name).write_bytes((P/'private/review/family8-state4'/name).read_bytes())
 sources=[p for crate in ['assets','core','runtime']for p in(P/f'rust/crates/genewars-{crate}/src').rglob('*.rs')];pins=[pin(p)for p in sources];rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe');runtime=W/'draft-runtime-src'
 shutil.copytree(P/'rust/crates/genewars-runtime/src',runtime,dirs_exist_ok=True);target=runtime/'family8_motion.rs';target.write_bytes(target.read_bytes()+b'\n'+(W/'candidate.rs').read_bytes())
 for crate in ['assets','core','runtime']:
  lib=runtime/'lib.rs' if crate=='runtime' else P/f'rust/crates/genewars-{crate}/src/lib.rs'
  args=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(lib),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
  run(args)
 args=[rustc,'--edition=2021','-O','-C','overflow-checks=yes',str(P/'research/variants/family8-departure-review.rs'),'-L',f'dependency={W}','-o',str(W/'review.exe')]
 for dep in ['assets','core','runtime']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
 run(args);run([str(W/'review.exe'),str(W)]);got=(W/'actual.bin').read_bytes();want=(W/'expected.bin').read_bytes();assert got==want,dict(byte=next((i for i,(a,b)in enumerate(zip(got,want))if a!=b),None));assert pins==[pin(p)for p in sources]
 source=json.loads((W/'source.json').read_text());result=dict(status='passed-private-state9-candidate',callbacks=source['callbacks'],compared_bytes=len(got),literal_instructions=source['literal_instructions'],source_pins=pins,listing_and_input_pins=source['pins'],candidate=pin(W/'candidate.rs'),original_execution=False,gui_execution=False)
 (W/'draft-validation.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k]for k in ['status','callbacks','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()
