"""Source-to-production mortality comparison, from actual before9052."""
from pathlib import Path
from datetime import datetime,timezone
import sys,subprocess,hashlib,json,shutil,struct
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
W=P/'private/review/mule-mortality'
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
def run(args):
 r=subprocess.run(args,cwd=P,env=environment(),capture_output=True,timeout=240);assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'));return r.stdout.decode(errors='replace')
def main():
 start=P/'private/review/family8-trajectory/bb1c98b625784fc49cefacf05b8133ff/frame9052/boundary'
 run([sys.executable,'-B',str(P/'research/variants/creature-hungry-source.py'),'9052','--snapshot='+str(start.relative_to(P))])
 run([sys.executable,'-B',str(Path(__file__).with_name('mule-mortality-source.py'))])
 (W/'map.bin').write_bytes((start/'backing.bin').read_bytes()[0x355e:0x3b55e]);inputs=bytearray();expected=bytearray();steps=0;source_pins={}
 for frame in range(9052,9075):
  fixture=start if frame==9052 else W/('frame'+str(frame));before=(fixture/'creatures.bin').read_bytes()[:124];after=bytearray((fixture/('source-after'+str(frame)+'.bin')).read_bytes());after[31:39]=before[31:39];inputs+=struct.pack('<I',frame)+before;expected+=after
  report=json.loads((fixture/('source-trace'+str(frame)+'.json')).read_text());steps+=report['report']['steps'];source_pins.update({p['path']:p['sha256']for p in report['listing_pins']})
 (W/'input.bin').write_bytes(inputs);(W/'expected.bin').write_bytes(expected)
 for name in ['angles.bin','bounce.bin']:(W/name).write_bytes((P/'private/review/creature-motion'/name).read_bytes())
 (W/'mule.anb').write_bytes((P/'private/reference-run/run-006/drive-c/GAME/DATA/CREATURE/MULE.ANB').read_bytes())
 sources=[p for crate in ['assets','core','runtime']for p in(P/f'rust/crates/genewars-{crate}/src').rglob('*.rs')];pins=[pin(p)for p in sources];rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe')
 for crate in ['assets','core','runtime']:
  args=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(P/f'rust/crates/genewars-{crate}/src/lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
  run(args)
 args=[rustc,'--edition=2021','-O','-C','overflow-checks=yes',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','-o',str(W/'review.exe')]
 for dep in ['assets','core','runtime']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
 run(args);print(run([str(W/'review.exe'),str(W)]));actual=(W/'actual.bin').read_bytes();assert actual==expected,dict(byte=next((i for i,(a,b)in enumerate(zip(actual,expected))if a!=b),None));assert pins==[pin(p)for p in sources]
 result=dict(status='passed-bounded-neutral-mule-mortality',utc=datetime.now(timezone.utc).isoformat(),callbacks=23,compared_bytes=len(actual),literal_instructions=steps,source_pins=pins,listing_pins=source_pins,review_pins=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs')),pin(Path(__file__).with_name('mule-mortality-source.py')),pin(P/'research/variants/creature-hungry-source.py'),pin(P/'research/variants/initial-creature-source.py')],inputs=[pin(x)for x in list(start.glob('*.bin')) if not x.name.startswith('source-')],outputs=[pin(W/f)for f in ['input.bin','expected.bin','actual.bin','map.bin','angles.bin','bounce.bin','mule.anb']],original_execution=False,gui_execution=False,
  limits=['Actual before9052 is fromcorrectedtrajectorybb1c...; subsequentcallbacks hold otherfamilies fixed. Literal model also checks stackpoison fills55/FF.', 'Naturalowner4selector4 mortality withhealthpositive/null actionreferences; deadstates24/25 stationarydesired-1 only. Genericnegativephysicsdomain unchanged.', 'SourcefirstRNGunchanged; planner hasnoRNGinput/draw. Fullownedworld composition andrenderer are separatechecks.', 'BONES conversion at corpse reserve<=max62/2 remains rejected, as doexternaldeathreferences/otherowners/physicaldamage.'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:result[k]for k in ['status','callbacks','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()
