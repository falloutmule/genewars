"""Independent neutral-food acquisition replay; no original or GUI execution."""
from pathlib import Path
from datetime import datetime,timezone
import subprocess,sys,json,hashlib
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
W=P/'private/review/creature-motion';S=Path(__file__).with_name('creature-hungry-source.py')
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
def run(cmd):
 r=subprocess.run(cmd,cwd=P,env=environment(),capture_output=True,timeout=180);assert r.returncode==0,r.stderr.decode(errors='replace');return r.stdout.decode(errors='replace')
def main():
 cases=[(f,5428)for f in [365,366,375,385,415,447,508,509,689,704,705,1311,1312,5677,5678]]+[(f,0)for f in [441,508,509,631,640,641]]+[(f,65535)for f in [511,607,608,609,974,975]]
 for frame,seed in cases:run([sys.executable,'-B',str(S),str(frame)]+([f'--seed={seed}']if seed!=5428 else[]))
 rustc=P/'private/tools/rust/cargo/bin/rustc.exe'
 paths=[p for name in ['assets','core','runtime']for p in(P/f'rust/crates/genewars-{name}/src').rglob('*.rs')];pins=[pin(p)for p in paths]
 for name in ['assets','core','runtime']:
  args=[str(rustc),'--edition=2021','--cap-lints=allow','-O','--crate-name',f'genewars_{name}','--crate-type','rlib',str(P/f'rust/crates/genewars-{name}/src/lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{name}.rlib')]
  if name=='runtime':
   for d in ['assets','core']:args+=['--extern',f'genewars_{d}={W}/libgenewars_{d}.rlib']
  run(args)
 exe=W/'hungry-review.exe';run([str(rustc),'--edition=2021',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','--extern',f'genewars_runtime={W}/libgenewars_runtime.rlib','-o',str(exe)])
 print(run([str(exe),str(P/'private/integration/combined-before-mule365/acquisition-fixture.bin'),str(P/'private/integration/combined-before-mule415/feed-fixture.bin')]).strip())
 historical=[]
 import struct
 for frame,seed in cases:
  prefix=f'combined-seed{seed}-before-mule'if seed!=5428 else'combined-before-mule';w=P/f'private/integration/{prefix}{frame}'
  a=bytearray((w/f'source-after{frame}.bin').read_bytes());b=bytearray((w/f'native-after{frame}.bin').read_bytes());a[31:39]=b[31:39]=bytes(8);assert a==b,(frame,'record')
  assert (w/f'map-after{frame}.bin').read_bytes()==(w/f'source-map-after{frame}.bin').read_bytes(),(frame,'map')
  compared=229500;full=w/f'backing-after{frame}.bin'
  if full.exists():
   expected=bytearray((w/'backing.bin').read_bytes());expected[:0x355e]=(w/f'source-prefix-after{frame}.bin').read_bytes();expected[0x355e:0x3b55e]=(w/f'source-map-after{frame}.bin').read_bytes();expected[0x43d4e:0x48f4e]=(w/f'source-plants-after{frame}.bin').read_bytes();assert full.read_bytes()==expected,(frame,'full backing');compared=124+len(expected)
  trace=json.loads((w/f'source-trace{frame}.json').read_text());r=trace['report'];assert struct.unpack('<HBI',(w/f'rng-after{frame}.bin').read_bytes())==(r['rng_seed_after'],r['rng_ring_after'],r['rng_draw_count_after'])
  historical.append(dict(frame=frame,startup_seed=seed,compared_bytes=compared,source_instructions=r['steps'],ui_cleanup_deferred=trace['deferred_ui_calls'],portrait_check_deferred=trace.get('deferred_portrait_checks',0),inputs=[pin(w/f)for f in ['backing.bin','creatures.bin','actors.bin','effects.bin','players.bin','rng.bin',f'native-after{frame}.bin',f'map-after{frame}.bin',f'rng-after{frame}.bin']]+([pin(full)]if full.exists()else[]),reference_trace=pin(w/f'source-trace{frame}.json')))
 assert pins==[pin(p)for p in paths]
 result=dict(status='passed-source-food-acquisition-and-feeding',utc=datetime.now(timezone.utc).isoformat(),current_native_pure_plan_bytes=248,current_native_rejection_cases=4,historical_callback_comparisons=historical,historical_compared_bytes=sum(r['compared_bytes']for r in historical),stack_fill_cases=[0,85,255],source_pins=pins,review_sources=[pin(p)for p in [Path(__file__),Path(__file__).with_suffix('.rs'),S,Path(__file__).with_name('initial-creature-source.py')]],limits=['Current compiled acquisition/feeding plans are compared against independently interpreted source before/after images. Full owned callback comparisons use parent-generated private historical captures; this script does not freshly replay the whole inter-family scheduler.','Render queue83140, module UI cleanup2C200 and optional portrait check5E068 remain explicit pending requests; source417E0 is gated off by the explicitly bound mission-tail flags38. No original or GUI execution.','Source/native pointer gap31..39 is normalized for comparison; full backing comparisons do not assert it is a complete save file.','Immediate neutral plant death is admitted only for retirement delay<4; larger delays, foreign neighborhoods, energy death and source water-damage branches remain explicit boundaries. Full-reserve completion, hunger saturation and target release are compared across all three seeded contexts.'])
 out=Path(__file__).with_suffix('.json');temp=out.with_suffix('.json.tmp');temp.write_text(json.dumps(result,indent=2)+'\n');temp.replace(out);print('food acquisition source/native review passed')
if __name__=='__main__':main()
